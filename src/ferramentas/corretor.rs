use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct CorretorMock {
    dicionario: Vec<String>,
}

impl CorretorMock {
    pub fn new(caminho_dicionario: &str) -> Self {
        let dicionario = fs::read_to_string(caminho_dicionario)
            .and_then(|c| Ok(serde_json::from_str::<Vec<String>>(&c).unwrap_or_default()))
            .unwrap_or_default();
            
        Self { dicionario }
    }
    
    pub fn sortear_palavra(&self) -> String {
        if self.dicionario.is_empty() {
            return "gato".to_string();
        }
        
        let mut seed = 0;
        if let Ok(duration) = SystemTime::now().duration_since(UNIX_EPOCH) {
            seed = duration.as_nanos() as usize;
        }
        
        let index = seed % self.dicionario.len();
        self.dicionario[index].clone()
    }
    
    // Retorna (Acertou, Mensagem/Palavra)
    pub fn verificar_desafio(&self, digitado: &str, esperado: &str) -> bool {
        let digitado_lower = digitado.to_lowercase();
        let esperado_lower = esperado.to_lowercase();
        
        digitado_lower == esperado_lower
    }
}
