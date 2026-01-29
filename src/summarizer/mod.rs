pub mod gemini;
pub mod ollama;

use crate::db::get_config;
use async_trait::async_trait;
use sqlx::SqlitePool;

#[derive(Debug, Clone)]
pub struct AIConfig {
    pub model: String,
    pub temperature: f64,
    pub top_p: f64,
    pub num_predict: i32,
    pub api_url: Option<String>,
    pub api_key: Option<String>,
}

#[async_trait]
pub trait Summarizer: Send + Sync {
    async fn summarize(&self, diff: &str) -> anyhow::Result<String>;
}

pub async fn get_summarizer(pool: &SqlitePool) -> anyhow::Result<Box<dyn Summarizer>> {
    let provider = get_config(pool, "active_provider")
        .await?
        .unwrap_or_else(|| "ollama".to_string());

    let model_key = if provider == "gemini" {
        format!("{}_api_model", provider)
    } else {
        format!("{}_model", provider)
    };

    let model = get_config(pool, &model_key).await?.unwrap_or_default();
    let temperature: f64 = get_config(pool, "ai_temperature")
        .await?
        .unwrap_or_default()
        .parse()
        .unwrap_or(0.1);
    let top_p: f64 = get_config(pool, "ai_top_p")
        .await?
        .unwrap_or_default()
        .parse()
        .unwrap_or(0.9);
    let num_predict: i32 = get_config(pool, "ai_num_predict")
        .await?
        .unwrap_or_default()
        .parse()
        .unwrap_or(250);

    let config = AIConfig {
        model,
        temperature,
        top_p,
        num_predict,
        api_url: get_config(pool, &format!("{}_url", provider)).await?,
        api_key: get_config(pool, &format!("{}_api_key", provider)).await?,
    };

    eprintln!("[INFO] Using provider: {}", provider);
    eprintln!("[INFO] Using model: {}", config.model);
    if let Some(ref key) = config.api_key {
        if !key.is_empty() {
            let masked_key = if key.len() > 8 {
                format!("{}...{}", &key[..4], &key[key.len() - 4..])
            } else {
                "****".to_string()
            };
            eprintln!("[INFO] Using API key: {}", masked_key);
        }
    }

    match provider.as_str() {
        "ollama" => Ok(Box::new(ollama::OllamaProvider::new(config))),
        "gemini" => Ok(Box::new(gemini::GeminiProvider::new(config))),
        _ => Err(anyhow::anyhow!("Unknown provider: {}", provider)),
    }
}
