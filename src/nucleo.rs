use crate::gerenciador_prompts::BancoDePrompts;
use crate::db_sessao::DbSessao;
use crate::ferramentas::sanitizador::{Sanitizador, StatusEntrada};
use crate::ferramentas::corretor::CorretorMock;
use crate::motor_ia::MotorIA;
use std::sync::Arc;
use serde::Deserialize;

#[derive(Deserialize)]
struct RespostaIntencao {
    intencao: String,
}

#[derive(uniffi::Object)]
pub struct NucleoAlfabetizacao {
    llama: Arc<dyn MotorIA>,
    banco_prompts: Arc<BancoDePrompts>,
    db: Arc<DbSessao>,
    sanitizador: Arc<Sanitizador>,
    corretor: Arc<CorretorMock>,
}

#[uniffi::export]
impl NucleoAlfabetizacao {
    #[uniffi::constructor]
    pub fn new(
        caminho_prompts: String, 
        caminho_db: String,
        caminho_dicionario: String,
        caminho_proibidas: String
    ) -> Result<Arc<Self>, String> {
        let banco_prompts = BancoDePrompts::carregar(&caminho_prompts)?;
        let llama = crate::motor_ia::criar_motor(banco_prompts.ia.clone())?;
        let db = DbSessao::new(&caminho_db)?;
        let sanitizador = Sanitizador::new(&caminho_proibidas);
        let corretor = CorretorMock::new(&caminho_dicionario);
        
        Ok(Arc::new(Self {
            llama: llama.into(), // Converte Box<dyn MotorIA> para Arc<dyn MotorIA>
            banco_prompts: Arc::new(banco_prompts),
            db: Arc::new(db),
            sanitizador: Arc::new(sanitizador),
            corretor: Arc::new(corretor),
        }))
    }

    pub fn processar_entrada(&self, id_crianca: String, texto_digitado: String) -> String {
        // 1. Sanitização Segura
        match self.sanitizador.verificar(&texto_digitado) {
            StatusEntrada::Proibida => return "Oops! Essa palavra não é legal. Vamos falar sobre outra coisa?".to_string(),
            StatusEntrada::Spam => return "Uau, quantas letras! Tente escrever uma palavra inteira.".to_string(),
            StatusEntrada::Valida => {}
        }

        // 2. Classificação de Intenção com Temperatura 0.0
        let intencao = self.classificar_intencao(&texto_digitado);

        // 3. Roteamento Seguro
        match intencao.as_str() {
            "SOLETRAÇÃO" => self.fluxo_soletracao(&id_crianca, &texto_digitado),
            "BATE_PAPO" => self.fluxo_bate_papo(&texto_digitado),
            _ => self.fluxo_bate_papo(&texto_digitado), // Fallback caso o JSON do LLM venha sujo
        }
    }
}

impl NucleoAlfabetizacao {
    fn classificar_intencao(&self, texto: &str) -> String {
        let prompt = self.banco_prompts.montar_prompt(
            "classificador_intencao", 
            &[("fala_crianca", texto)]
        );
        let temp = self.banco_prompts.temperaturas.logica; // 0.0
        
        match self.llama.inferir(&prompt, temp) {
            Ok(resp_json) => {
                if let Ok(parsed) = serde_json::from_str::<RespostaIntencao>(&resp_json) {
                    parsed.intencao
                } else {
                    "BATE_PAPO".to_string() // Fallback seguro sem panic!
                }
            },
            Err(_) => "BATE_PAPO".to_string() // Fallback em caso de erro da inferência
        }
    }

    fn fluxo_soletracao(&self, id_crianca: &str, palavra_digitada: &str) -> String {
        let (acertou, palavra_correta) = self.corretor.verificar_soletracao(palavra_digitada);
        
        if acertou {
            "Muito bem! Você escreveu direitinho!".to_string()
        } else {
            // Persistir falha no banco SQLite local
            let _ = self.db.salvar_erro(id_crianca, &palavra_correta); // Ignora erros de log sem panic!
            
            // Corrige com persona e pedagogia do LLM (temp 0.2)
            let prompt = self.banco_prompts.montar_prompt(
                "corrigir_erro", 
                &[
                    ("palavra_correta", &palavra_correta),
                    ("palavra_errada", palavra_digitada)
                ]
            );
            
            self.llama.inferir(&prompt, self.banco_prompts.temperaturas.correcao)
                .unwrap_or_else(|_| "Acontece! Vamos tentar de novo?".to_string())
        }
    }
    
    fn fluxo_bate_papo(&self, texto: &str) -> String {
        let prompt = self.banco_prompts.montar_prompt(
            "bate_papo_livre", 
            &[("fala_crianca", texto)]
        );
        
        // Conversa solta e criativa (temp 0.7)
        self.llama.inferir(&prompt, self.banco_prompts.temperaturas.bate_papo)
            .unwrap_or_else(|_| "Que legal! Me conte mais!".to_string())
    }
}
