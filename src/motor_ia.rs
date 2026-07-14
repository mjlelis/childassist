use crate::gerenciador_prompts::ConfigIA;

use std::io::BufRead;

pub trait MotorIA: Send + Sync {
    fn inferir(&self, prompt: &str, temperatura: f32, callback: Option<&mut dyn FnMut(&str)>) -> Result<String, String>;
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
    fn inferir(&self, prompt: &str, temperatura: f32, mut callback: Option<&mut dyn FnMut(&str)>) -> Result<String, String> {
        let req_body = OllamaRequest {
            model: &self.config_ia.modelo,
            prompt,
            stream: callback.is_some(),
            options: OllamaOptions { temperature: temperatura },
        };

        let res = self.client.post(&self.config_ia.endpoint)
            .json(&req_body)
            .send()
            .map_err(|e| format!("Erro requisição Ollama: {}", e))?;

        if !res.status().is_success() {
            return Err(format!("Erro Ollama Status: {}", res.status()));
        }

        if let Some(ref mut cb) = callback {
            let mut full_response = String::new();
            let reader = std::io::BufReader::new(res);
            for line_res in reader.lines() {
                if let Ok(line) = line_res {
                    if line.is_empty() { continue; }
                    if let Ok(chunk) = serde_json::from_str::<OllamaResponse>(&line) {
                        cb(&chunk.response);
                        full_response.push_str(&chunk.response);
                    }
                }
            }
            Ok(full_response.trim().to_string())
        } else {
            let ollama_res: OllamaResponse = res.json()
                .map_err(|e| format!("Erro parse Ollama JSON: {}", e))?;
            Ok(ollama_res.response.trim().to_string())
        }
    }
}

// ==========================================
// LLAMA.CPP SERVER ENGINE (Chave rápida para ambiente local)
// ==========================================
pub struct LlamaCppServerEngine {
    config_ia: ConfigIA,
    client: reqwest::blocking::Client,
}

#[derive(serde::Serialize)]
struct LlamaServerRequest<'a> {
    prompt: &'a str,
    temperature: f32,
    stream: bool,
}

#[derive(serde::Deserialize)]
struct LlamaServerResponse {
    content: String,
}

impl LlamaCppServerEngine {
    pub fn new(config: ConfigIA) -> Result<Self, String> {
        Ok(Self {
            config_ia: config,
            client: reqwest::blocking::Client::new(),
        })
    }
}

impl MotorIA for LlamaCppServerEngine {
    fn inferir(&self, prompt: &str, temperatura: f32, mut callback: Option<&mut dyn FnMut(&str)>) -> Result<String, String> {
        let req_body = LlamaServerRequest {
            prompt,
            temperature: temperatura,
            stream: callback.is_some(),
        };

        let res = self.client.post(&self.config_ia.endpoint)
            .json(&req_body)
            .send()
            .map_err(|e| format!("Erro requisição Llama Server: {}", e))?;

        if !res.status().is_success() {
            return Err(format!("Erro Llama Server Status: {}", res.status()));
        }

        if let Some(ref mut cb) = callback {
            let mut full_response = String::new();
            let reader = std::io::BufReader::new(res);
            for line_res in reader.lines() {
                if let Ok(line) = line_res {
                    let l = line.trim();
                    if l.starts_with("data: ") {
                        let json_str = &l[6..];
                        if json_str == "[DONE]" { break; }
                        if let Ok(chunk) = serde_json::from_str::<LlamaServerResponse>(json_str) {
                            cb(&chunk.content);
                            full_response.push_str(&chunk.content);
                        }
                    }
                }
            }
            Ok(full_response.trim().to_string())
        } else {
            let llama_res: LlamaServerResponse = res.json()
                .map_err(|e| format!("Erro parse Llama Server JSON: {}", e))?;
            Ok(llama_res.content.trim().to_string())
        }
    }
}

// ==========================================
// FACTORY
// ==========================================
pub fn criar_motor(config: ConfigIA) -> Result<Box<dyn MotorIA>, String> {
    match config.provedor.as_str() {
        "ollama" => Ok(Box::new(OllamaEngine::new(config)?)),
        "llama_cpp" => Ok(Box::new(LlamaCppServerEngine::new(config)?)),
        _ => Err(format!("Provedor desconhecido: {}", config.provedor)),
    }
}
