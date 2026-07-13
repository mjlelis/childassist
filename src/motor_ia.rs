use crate::gerenciador_prompts::ConfigIA;

pub trait MotorIA: Send + Sync {
    fn inferir(&self, prompt: &str, temperatura: f32) -> Result<String, String>;
}

// ==========================================
// OLLAMA ENGINE (Para uso local via HTTP)
// ==========================================
pub struct OllamaEngine {
    config_ia: ConfigIA,
    client: reqwest::blocking::Client,
}

#[derive(serde::Serialize)]
struct OllamaRequest<'a> {
    model: &'a str,
    prompt: &'a str,
    stream: bool,
    options: OllamaOptions,
}

#[derive(serde::Serialize)]
struct OllamaOptions {
    temperature: f32,
}

#[derive(serde::Deserialize)]
struct OllamaResponse {
    response: String,
}

impl OllamaEngine {
    pub fn new(config: ConfigIA) -> Result<Self, String> {
        Ok(Self {
            config_ia: config,
            client: reqwest::blocking::Client::new(),
        })
    }
}

impl MotorIA for OllamaEngine {
    fn inferir(&self, prompt: &str, temperatura: f32) -> Result<String, String> {
        let req_body = OllamaRequest {
            model: &self.config_ia.modelo,
            prompt,
            stream: false,
            options: OllamaOptions { temperature: temperatura },
        };

        let res = self.client.post(&self.config_ia.endpoint)
            .json(&req_body)
            .send()
            .map_err(|e| format!("Erro requisição Ollama: {}", e))?;

        if res.status().is_success() {
            let ollama_res: OllamaResponse = res.json()
                .map_err(|e| format!("Erro parse Ollama JSON: {}", e))?;
            Ok(ollama_res.response.trim().to_string())
        } else {
            Err(format!("Erro Ollama Status: {}", res.status()))
        }
    }
}

// ==========================================
// LLAMA.CPP ENGINE (Para embarcado / offline)
// ==========================================
// Mock de implementação que futuramente integrará com `llama_cpp_rs`.
pub struct LlamaCppEngine {
    _caminho_modelo: String,
}

impl LlamaCppEngine {
    pub fn new(caminho_modelo: String) -> Result<Self, String> {
        Ok(Self {
            _caminho_modelo: caminho_modelo,
        })
    }
}

impl MotorIA for LlamaCppEngine {
    fn inferir(&self, prompt: &str, _temperatura: f32) -> Result<String, String> {
        if prompt.contains("Analise a fala") {
            Ok(r#"{"intencao": "SOLETRAÇÃO"}"#.to_string())
        } else {
            Ok("Resposta direta gerada pelo Llama C++ (Offline).".to_string())
        }
    }
}

// ==========================================
// FACTORY
// ==========================================
pub fn criar_motor(config: ConfigIA) -> Result<Box<dyn MotorIA>, String> {
    match config.provedor.as_str() {
        "ollama" => Ok(Box::new(OllamaEngine::new(config)?)),
        "llama_cpp" => Ok(Box::new(LlamaCppEngine::new(config.modelo)?)),
        _ => Err(format!("Provedor desconhecido: {}", config.provedor)),
    }
}
