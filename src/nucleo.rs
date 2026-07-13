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
        caminho_config: String, 
        dir_prompts: String,
        caminho_db: String,
        caminho_dicionario: String,
        caminho_proibidas: String
    ) -> Result<Arc<Self>, String> {
        let banco_prompts = BancoDePrompts::carregar(&caminho_config, &dir_prompts)?;
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
        let estado_missao = self.db.obter_estado_missao(&id_crianca);
        
        let intencao = if let Some((fase, _tema, opcoes, _palavra, _acertos, _total)) = estado_missao {
            let txt_limpo = texto_digitado.trim().to_lowercase();
            if txt_limpo == "sair" || txt_limpo == "parar" {
                let _ = self.db.limpar_desafio(&id_crianca);
                return "Missão cancelada! Você quer tentar soletrar de novo (1) ou bater papo (2)?".to_string();
            }

            if fase == "ESCOLHENDO_TEMA" {
                let num_opcoes = opcoes.split(',').count();
                if txt_limpo == "3" && num_opcoes == 2 {
                    let _ = self.db.limpar_desafio(&id_crianca);
                    return "Legal! Vamos bater papo! Sobre o que você quer falar?".to_string();
                } else if txt_limpo.contains("bater papo") {
                    let _ = self.db.limpar_desafio(&id_crianca);
                    return "Legal! Vamos bater papo! Sobre o que você quer falar?".to_string();
                }
                "ESCOLHENDO_TEMA".to_string()
            } else if fase == "JOGANDO" {
                "TENTATIVA_SOLETRAÇÃO".to_string()
            } else {
                "BATE_PAPO".to_string()
            }
        } else {
            let txt_limpo = texto_digitado.trim().to_lowercase();
            if txt_limpo == "1" || txt_limpo.contains("soletrar") || txt_limpo == "sim" {
                // 'sim' fora de missão vai direto pro jogo, cortando o loop de repetição cega da IA.
                "QUER_SOLETRAR".to_string()
            } else if txt_limpo == "2" || txt_limpo.contains("bater papo") || txt_limpo.contains("conversar") {
                let _ = self.db.limpar_desafio(&id_crianca); // Abandona a missão silenciosamente
                "BATE_PAPO".to_string()
            } else {
                self.classificar_intencao(&id_crianca, &texto_digitado)
            }
        };

        // 3. Roteamento Seguro com Máquina de Estados
        let resposta = match intencao.as_str() {
            "QUER_SOLETRAR" => self.fluxo_iniciar_escolha_tema(&id_crianca),
            "ESCOLHENDO_TEMA" => self.fluxo_iniciar_soletracao(&id_crianca, &texto_digitado),
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

    fn fluxo_iniciar_escolha_tema(&self, id_crianca: &str) -> String {
        let prompt = self.banco_prompts.montar_prompt("sugerir_temas", &[]);
        let json_resposta = self.llama.inferir(&prompt, 0.7)
            .unwrap_or_else(|_| "[\"Animais\", \"Frutas\", \"Cores\"]".to_string());
            
        // Extrai temas do JSON de forma rústica para Edge IoT
        let clean = json_resposta.replace("[", "").replace("]", "").replace("\"", "");
        let temas_array: Vec<&str> = clean.split(',').map(|s| s.trim()).collect();
        
        let t1 = temas_array.get(0).unwrap_or(&"Animais");
        let t2 = temas_array.get(1).unwrap_or(&"Frutas");
        let t3 = temas_array.get(2).unwrap_or(&"Cores");
        
        // Inicia a fase salvando as 3 opções no banco (separadas por vírgula)
        let string_temas = format!("{},{},{}", t1, t2, t3);
        let _ = self.db.iniciar_escolha_tema(id_crianca, &string_temas);
        
        format!("Legal! Escolha um tema pelo número:\n1 - {}\n2 - {}\n3 - {}", t1, t2, t3)
    }

    fn fluxo_iniciar_soletracao(&self, id_crianca: &str, texto_digitado: &str) -> String {
        // Recupera as opções que foram salvas no banco
        let estado = self.db.obter_estado_missao(id_crianca);
        let mut tema_escolhido = "Geral".to_string();
        
        if let Some((_fase, _tema, opcoes_tema, _palavra, _acertos, _total)) = estado {
            let txt_limpo = texto_digitado.trim();
            let opcoes: Vec<&str> = opcoes_tema.split(',').collect();
            
            tema_escolhido = match txt_limpo {
                "1" => opcoes.get(0).unwrap_or(&"Geral").to_string(),
                "2" => opcoes.get(1).unwrap_or(&"Geral").to_string(),
                "3" => opcoes.get(2).unwrap_or(&"Geral").to_string(),
                _ => txt_limpo.to_string(), // Se a criança digitou o nome inteiro
            };
        }
        
        let prompt_palavra = self.banco_prompts.montar_prompt(
            "gerar_palavra_desafio", 
            &[
                ("tema", &tema_escolhido),
                ("palavra_anterior", "nenhuma")
            ]
        );
        let mut palavra_sorteada = match self.llama.inferir(&prompt_palavra, 0.9) {
            Ok(p) => {
                let limpo = p.trim();
                let partes: Vec<&str> = limpo.split_whitespace().collect();
                let ultima = partes.last().unwrap_or(&"").to_string();
                ultima.chars().filter(|c| c.is_alphabetic()).collect::<String>().to_uppercase()
            },
            Err(_) => String::new(),
        };
        
        if palavra_sorteada.is_empty() || palavra_sorteada.len() > 10 {
            palavra_sorteada = self.corretor.sortear_palavra(); // Fallback seguro
        }

        let _ = self.db.iniciar_missao(id_crianca, &tema_escolhido, &palavra_sorteada, 3);
        
        // Em dispositivos IoT limitados, evitar uso de LLM para templates estritos economiza processamento e evita alucinações
        format!("Missão de '{}' iniciada! Vamos soletrar 3 palavras. Como se escreve a palavra '{}'?", tema_escolhido, palavra_sorteada)
    }

    fn fluxo_avaliar_soletracao(&self, id_crianca: &str, palavra_digitada: &str) -> String {
        let estado = self.db.obter_estado_missao(id_crianca);
        
        if estado.is_none() {
            return self.fluxo_bate_papo(id_crianca, palavra_digitada);
        }
        
        let (_fase, tema, opcoes_tema, palavra_esperada, acertos, total) = estado.unwrap();
        
        let acertou = self.corretor.verificar_desafio(palavra_digitada, &palavra_esperada);
        
        if acertou {
            let novos_acertos = acertos + 1;
            if novos_acertos < total {
                // Sorteia próxima palavra usando o mesmo tema da missão
                let prompt_palavra = self.banco_prompts.montar_prompt(
                    "gerar_palavra_desafio", 
                    &[
                        ("tema", &tema),
                        ("palavra_anterior", &palavra_esperada)
                    ]
                );
                
                let mut nova_palavra = match self.llama.inferir(&prompt_palavra, 0.9) {
                    Ok(p) => {
                        let limpo = p.trim();
                        let partes: Vec<&str> = limpo.split_whitespace().collect();
                        let ultima = partes.last().unwrap_or(&"").to_string();
                        ultima.chars().filter(|c| c.is_alphabetic()).collect::<String>().to_uppercase()
                    },
                    Err(_) => String::new(),
                };
                if nova_palavra.is_empty() || nova_palavra.len() > 10 {
                    nova_palavra = self.corretor.sortear_palavra();
                }

                let _ = self.db.avancar_missao(id_crianca, &nova_palavra);
                
                let prompt_progresso = self.banco_prompts.montar_prompt(
                    "progresso_missao", 
                    &[
                        ("palavra_acertada", &palavra_esperada),
                        ("acertos", &novos_acertos.to_string()),
                        ("total", &total.to_string()),
                        ("nova_palavra", &nova_palavra)
                    ]
                );
                
                self.llama.inferir(&prompt_progresso, self.banco_prompts.temperaturas.bate_papo)
                    .unwrap_or_else(|_| format!("Muito bem! Você acertou {} de {}! Agora escreva '{}'.", novos_acertos, total, nova_palavra))
            } else {
                // Missão concluída!
                let prompt_conclusao = self.banco_prompts.montar_prompt(
                    "conclusao_missao", &[("palavra_acertada", &palavra_esperada)]
                );
                
                let msg_conclusao = self.llama.inferir(&prompt_conclusao, self.banco_prompts.temperaturas.bate_papo)
                    .unwrap_or_else(|_| "Uau! Você completou toda a missão! Parabéns!".to_string());
                
                // Filtra as opções não escolhidas ainda
                let mut opcoes_restantes: Vec<&str> = opcoes_tema.split(',').collect();
                opcoes_restantes.retain(|&x| x.trim().to_lowercase() != tema.trim().to_lowercase());
                
                if opcoes_restantes.len() >= 2 {
                    let t1 = opcoes_restantes[0].trim();
                    let t2 = opcoes_restantes[1].trim();
                    
                    let string_temas = format!("{},{}", t1, t2);
                    let _ = self.db.iniciar_escolha_tema(id_crianca, &string_temas);
                    
                    format!("{}\nVamos para a próxima missão? Escolha:\n1 - {}\n2 - {}\n3 - Bater Papo", msg_conclusao, t1, t2)
                } else if opcoes_restantes.len() == 1 {
                    let t1 = opcoes_restantes[0].trim();
                    
                    let string_temas = t1.to_string();
                    let _ = self.db.iniciar_escolha_tema(id_crianca, &string_temas);
                    
                    format!("{}\nFalta apenas um tema! Escolha:\n1 - {}\n2 - Bater Papo", msg_conclusao, t1)
                } else {
                    let _ = self.db.limpar_desafio(id_crianca);
                    format!("{}\nVocê fechou todos os temas! Que incrível! Digite 1 para nova missão, ou 2 para Bater Papo!", msg_conclusao)
                }
            }
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
