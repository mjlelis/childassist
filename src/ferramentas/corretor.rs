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
            return (true, palavra_lower);
        }
        
        let primeira_letra = palavra_lower.chars().next().unwrap_or(' ');
        let mut sugestao = None;
        
        for palavra_dic in &self.dicionario {
            if palavra_dic.starts_with(primeira_letra) {
                let diff = (palavra_dic.len() as isize - palavra_lower.len() as isize).abs();
                if diff <= 2 {
                    sugestao = Some(palavra_dic.clone());
                    break;
                }
            }
        }
        
        if let Some(s) = sugestao {
            (false, s)
        } else {
            // Assume que é um nome próprio se não houver nada remotamente parecido
            (true, palavra_lower)
        }
    }
}
