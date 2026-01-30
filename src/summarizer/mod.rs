pub mod gemini;
pub mod ollama;

use crate::config::AsumConfig;
use async_trait::async_trait;
use tracing::info;

#[derive(Debug, Clone)]
pub struct AIConfig {
    pub model: String,
    pub temperature: f64,
    pub top_p: f64,
    pub num_predict: i32,
    pub api_url: Option<String>,
    pub api_key: Option<String>,
    pub system_prompt: String,
    pub user_prompt: String,
}

#[async_trait]
pub trait Summarizer: Send + Sync {
    async fn summarize(&self, diff: &str) -> anyhow::Result<String>;
}

pub async fn get_summarizer(config: AsumConfig) -> anyhow::Result<Box<dyn Summarizer>> {
    let provider = config.active_provider.clone();

    let model = match provider.as_str() {
        "gemini" => config.gemini_model.clone().unwrap_or_default(),
        "ollama" => config.ollama_model.clone().unwrap_or_default(),
        _ => "".to_string(),
    };

    let ai_config = AIConfig {
        model,
        temperature: config.ai_temperature,
        top_p: config.ai_top_p,
        num_predict: config.ai_num_predict,
        api_url: config.ollama_url.clone(),
        api_key: config.gemini_api_key.clone(),
        system_prompt: config.system_prompt.clone(),
        user_prompt: config.user_prompt.clone(),
    };

    info!("Using provider: {}", provider);
    info!("Using model: {}", ai_config.model);
    if let Some(key) = ai_config.api_key.as_ref().filter(|k| !k.is_empty()) {
        let masked_key = if key.len() > 8 {
            format!("{}...{}", &key[..4], &key[key.len() - 4..])
        } else {
            "****".to_string()
        };
        info!("Using API key: {}", masked_key);
    }

    match provider.as_str() {
        "ollama" => Ok(Box::new(ollama::OllamaProvider::new(ai_config))),
        "gemini" => Ok(Box::new(gemini::GeminiProvider::new(ai_config))),
        _ => Err(anyhow::anyhow!("Unknown provider: {}", provider)),
    }
}

pub fn generate_prompt(prompt_template: &str, diff: &str) -> String {
    prompt_template.replace("{{diff}}", diff)
}
