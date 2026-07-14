use serde::Deserialize;
use std::collections::HashMap;
use std::fs;

#[derive(Debug, Deserialize, Clone)]
pub struct ConfigIA {
    pub provedor: String,
    pub endpoint: String,
    pub modelo: String,
    pub ativo: bool,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Temperaturas {
    pub logica: f32,
    pub correcao: f32,
    pub bate_papo: f32,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ConfigGeral {
    pub versao: String,
    pub motores_ia: Vec<ConfigIA>,
    pub temperaturas: Temperaturas,
}

#[derive(Debug, Clone)]
pub struct BancoDePrompts {
    pub ia: ConfigIA,
    pub temperaturas: Temperaturas,
    pub personas: HashMap<String, String>,
    pub fluxos: HashMap<String, String>,
}

impl BancoDePrompts {
    pub fn carregar(caminho_config: &str, dir_prompts: &str) -> Result<Self, String> {
        let config_str = fs::read_to_string(caminho_config)
            .map_err(|e| format!("Erro ao ler config {}: {}", caminho_config, e))?;
        let config: ConfigGeral = serde_json::from_str(&config_str)
            .map_err(|e| format!("Erro ao fazer parse do config.json: {}", e))?;

        let ativos: Vec<ConfigIA> = config.motores_ia.into_iter().filter(|ia| ia.ativo).collect();
        
        if ativos.len() != 1 {
            return Err(format!(
                "Erro Crítico de Configuração: Esperado EXATAMENTE 1 motor de IA com 'ativo: true'. Encontrados: {}. Corrija o arquivo config.json.", 
                ativos.len()
            ));
        }
        
        let ia_ativa = ativos[0].clone();

        let mut personas = HashMap::new();
        let mut fluxos = HashMap::new();

        if let Ok(entradas) = fs::read_dir(dir_prompts) {
            for entrada in entradas.flatten() {
                let path = entrada.path();
                if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("txt") {
                    let nome_arquivo = path.file_stem().unwrap().to_str().unwrap().to_string();
                    let conteudo = fs::read_to_string(&path).unwrap_or_default();
                    
                    if nome_arquivo.starts_with("persona_") {
                        let chave_persona = nome_arquivo.replace("persona_", "");
                        personas.insert(chave_persona, conteudo);
                    } else {
                        fluxos.insert(nome_arquivo, conteudo);
                    }
                }
            }
        }

        Ok(Self {
            ia: ia_ativa,
            temperaturas: config.temperaturas,
            personas,
            fluxos,
        })
    }

    pub fn montar_prompt(&self, chave_fluxo: &str, variaveis: &[(&str, &str)]) -> String {
        let mut texto = self.fluxos.get(chave_fluxo)
            .cloned()
            .unwrap_or_else(|| String::from("Fluxo não encontrado."));

        if texto.contains("{persona}") {
            let persona = self.personas.get("professor_alfa")
                .cloned()
                .unwrap_or_default();
            texto = texto.replace("{persona}", &persona);
        }

        for (chave, valor) in variaveis {
            let tag = format!("{{{}}}", chave);
            texto = texto.replace(&tag, valor);
        }

        texto
    }
}
