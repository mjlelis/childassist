use std::fs;

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
    
    // Retorna (Acertou, Palavra Certa que mais se assemelha ou a própria palavra se não errada)
    pub fn verificar_soletracao(&self, palavra: &str) -> (bool, String) {
        let palavra_lower = palavra.to_lowercase();
        
        if self.dicionario.contains(&palavra_lower) {
            (true, palavra_lower)
        } else {
            // Mock de correção (retorna a primeira palavra do dicionário apenas como exemplo)
            let sugestao = self.dicionario.first().cloned().unwrap_or_else(|| "gato".to_string());
            (false, sugestao)
        }
    }
}
