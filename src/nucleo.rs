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

    pub fn iniciar_interacao(&self, id_crianca: String) -> String {
        // Limpa qualquer desafio pendente de sessões anteriores que fecharam de forma abrupta
        let _ = self.db.limpar_desafio(&id_crianca);
        
        let prompt = self.banco_prompts.montar_prompt("boas_vindas", &[]);
        let resposta = self.llama.inferir(&prompt, self.banco_prompts.temperaturas.bate_papo)
            .unwrap_or_else(|_| "Oi! Eu sou o seu tutor. O que vamos descobrir hoje?".to_string());
        
        let _ = self.db.salvar_mensagem(&id_crianca, "Brinquedo", &resposta);
        resposta
    }

    pub fn processar_entrada(&self, id_crianca: String, texto_digitado: String) -> String {
        let _ = self.db.salvar_mensagem(&id_crianca, "Criança", &texto_digitado);

        // 1. Sanitização Segura
        match self.sanitizador.verificar(&texto_digitado) {
            StatusEntrada::Proibida => return "Oops! Essa palavra não é legal. Vamos falar sobre outra coisa?".to_string(),
            StatusEntrada::Spam => return "Uau, quantas letras! Tente escrever uma palavra inteira.".to_string(),
            StatusEntrada::Valida => {}
        }

        // 2. Classificação de Intenção (Híbrida: Regras Fixas + LLM)
        let tem_desafio = !self.db.obter_desafio(&id_crianca).unwrap_or_default().is_empty();
        
        let intencao = if tem_desafio {
            "TENTATIVA_SOLETRAÇÃO".to_string()
        } else {
            let txt_limpo = texto_digitado.trim().to_lowercase();
            if txt_limpo == "1" || txt_limpo == "soletrar" {
                "QUER_SOLETRAR".to_string()
            } else if txt_limpo == "2" {
                "BATE_PAPO".to_string()
            } else {
                self.classificar_intencao(&id_crianca, &texto_digitado)
            }
        };

        // 3. Roteamento Seguro com Máquina de Estados
        let resposta = match intencao.as_str() {
            "QUER_SOLETRAR" => self.fluxo_iniciar_soletracao(&id_crianca),
            "TENTATIVA_SOLETRAÇÃO" => self.fluxo_avaliar_soletracao(&id_crianca, &texto_digitado),
            "BATE_PAPO" => self.fluxo_bate_papo(&id_crianca, &texto_digitado),
            _ => self.fluxo_bate_papo(&id_crianca, &texto_digitado),
        };

        let _ = self.db.salvar_mensagem(&id_crianca, "Brinquedo", &resposta);
        resposta
    }
}

impl NucleoAlfabetizacao {
    fn classificar_intencao(&self, id_crianca: &str, texto: &str) -> String {
        let contexto = self.db.obter_contexto(id_crianca, 2).unwrap_or_default();
        let prompt = self.banco_prompts.montar_prompt(
            "classificador_intencao", 
            &[
                ("contexto", &contexto),
                ("fala_crianca", texto)
            ]
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

    fn fluxo_iniciar_soletracao(&self, id_crianca: &str) -> String {
        let palavra_sorteada = self.corretor.sortear_palavra();
        let _ = self.db.definir_desafio(id_crianca, &palavra_sorteada);
        
        // Em dispositivos IoT limitados, evitar uso de LLM para templates estritos economiza processamento e evita alucinações
        format!("Oba! Vamos soletrar! Como se escreve a palavra '{}'?", palavra_sorteada)
    }

    fn fluxo_avaliar_soletracao(&self, id_crianca: &str, palavra_digitada: &str) -> String {
        let palavra_esperada = self.db.obter_desafio(id_crianca).unwrap_or_default();
        
        if palavra_esperada.is_empty() {
            // Se caiu aqui sem estado, apenas joga pro bate-papo
            return self.fluxo_bate_papo(id_crianca, palavra_digitada);
        }
        
        let acertou = self.corretor.verificar_desafio(palavra_digitada, &palavra_esperada);
        
        if acertou {
            let _ = self.db.limpar_desafio(id_crianca);
            "Muito bem! Você escreveu direitinho! Quer continuar soletrando ou quer bater papo?".to_string()
        } else {
            let _ = self.db.salvar_erro(id_crianca, &palavra_esperada);
            
            let prompt = self.banco_prompts.montar_prompt(
                "corrigir_erro", 
                &[
                    ("palavra_correta", &palavra_esperada),
                    ("palavra_errada", palavra_digitada)
                ]
            );
            
            self.llama.inferir(&prompt, self.banco_prompts.temperaturas.correcao)
                .unwrap_or_else(|_| "Acontece! Vamos tentar de novo?".to_string())
        }
    }
    
    fn fluxo_bate_papo(&self, id_crianca: &str, texto: &str) -> String {
        let contexto = self.db.obter_contexto(id_crianca, 4).unwrap_or_default();
        let prompt = self.banco_prompts.montar_prompt(
            "bate_papo_livre", 
            &[
                ("contexto", &contexto),
                ("fala_crianca", texto)
            ]
        );
        
        // Conversa solta e criativa (temp 0.7)
        self.llama.inferir(&prompt, self.banco_prompts.temperaturas.bate_papo)
            .unwrap_or_else(|_| "Que legal! Me conte mais!".to_string())
    }
}
