use serde::Deserialize;
use std::collections::HashMap;
use std::fs;

#[derive(Debug, Deserialize, Clone)]
pub struct ConfigIA {
    pub provedor: String,
    pub endpoint: String,
    pub modelo: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Temperaturas {
    pub logica: f32,
    pub correcao: f32,
    pub bate_papo: f32,
}

#[derive(Debug, Deserialize, Clone)]
pub struct BancoDePrompts {
    pub versao: String,
    pub ia: ConfigIA,
    pub temperaturas: Temperaturas,
    pub personas: HashMap<String, String>,
    pub fluxos: HashMap<String, String>,
}

impl BancoDePrompts {
    pub fn carregar(caminho: &str) -> Result<Self, String> {
        let conteudo = fs::read_to_string(caminho)
            .map_err(|e| format!("Erro ao ler {}: {}", caminho, e))?;
        serde_json::from_str(&conteudo)
            .map_err(|e| format!("Erro ao fazer parse do JSON de prompts: {}", e))
    }

    pub fn montar_prompt(&self, chave_fluxo: &str, variaveis: &[(&str, &str)]) -> String {
        let mut texto = self.fluxos.get(chave_fluxo)
            .cloned()
            .unwrap_or_else(|| String::from("Fluxo não encontrado."));

        // Injeta a persona padrão se existir a tag
        if texto.contains("{persona}") {
            let persona = self.personas.get("professor_alfa")
                .cloned()
                .unwrap_or_default();
            texto = texto.replace("{persona}", &persona);
        }

        // Substitui variáveis dinâmicas
        for (chave, valor) in variaveis {
            let tag = format!("{{{}}}", chave);
            texto = texto.replace(&tag, valor);
        }

        texto
    }
}
