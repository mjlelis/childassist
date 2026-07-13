use regex::Regex;
use std::fs;

#[derive(Debug, PartialEq)]
pub enum StatusEntrada {
    Valida,
    Spam,
    Proibida,
}

pub struct Sanitizador {
    palavras_proibidas: Vec<String>,
    regex_repeticao: Regex,
}

impl Sanitizador {
    pub fn new(caminho_proibidas: &str) -> Self {
        let proibidas = fs::read_to_string(caminho_proibidas)
            .unwrap_or_default()
            .lines()
            .map(|l| l.trim().to_lowercase())
            .filter(|l| !l.is_empty())
            .collect();
            
        Self {
            palavras_proibidas: proibidas,
            regex_repeticao: Regex::new(r"(.)\1{4,}").unwrap(), // Mais de 4 repetições do mesmo caractere
        }
    }

    pub fn verificar(&self, texto: &str) -> StatusEntrada {
        let texto_lower = texto.to_lowercase();
        
        for palavra in &self.palavras_proibidas {
            if texto_lower.contains(palavra) {
                return StatusEntrada::Proibida;
            }
        }
        
        if self.regex_repeticao.is_match(&texto_lower) {
            return StatusEntrada::Spam;
        }
        
        StatusEntrada::Valida
    }
}
