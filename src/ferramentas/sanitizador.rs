use std::fs;

#[derive(Debug, PartialEq)]
pub enum StatusEntrada {
    Valida,
    Spam,
    Proibida,
}

pub struct Sanitizador {
    palavras_proibidas: Vec<String>,
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
        }
    }

    pub fn verificar(&self, texto: &str) -> StatusEntrada {
        let texto_lower = texto.to_lowercase();
        
        for palavra in &self.palavras_proibidas {
            if texto_lower.contains(palavra) {
                return StatusEntrada::Proibida;
            }
        }
        
        let mut last_char = '\0';
        let mut count = 0;
        
        for c in texto_lower.chars() {
            if c == last_char && c.is_alphabetic() {
                count += 1;
                if count >= 4 {
                    return StatusEntrada::Spam;
                }
            } else {
                last_char = c;
                count = 0;
            }
        }
        
        StatusEntrada::Valida
    }
}
