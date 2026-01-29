use crate::db::get_config;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use std::fs;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AsumConfig {
    pub active_provider: String,
    pub max_diff_length: usize,
    pub ai_temperature: f64,
    pub ai_top_p: f64,
    pub ai_num_predict: i32,
    pub ollama_url: Option<String>,
    pub ollama_model: Option<String>,
    pub gemini_api_key: Option<String>,
    pub gemini_model: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct TomlConfig {
    pub general: GeneralConfig,
    pub ai_params: AIParamsConfig,
    pub gemini: Option<GeminiConfig>,
    pub ollama: Option<OllamaConfig>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct GeneralConfig {
    pub active_provider: String,
    pub max_diff_length: usize,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct AIParamsConfig {
    pub num_predict: i32,
    pub temperature: f64,
    pub top_p: f64,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct GeminiConfig {
    pub api_key: String,
    pub model: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct OllamaConfig {
    pub model: String,
    pub url: String,
}

impl AsumConfig {
    pub async fn load(pool: &SqlitePool) -> Result<Self> {
        if std::path::Path::new("asum.toml").exists() {
            match Self::load_from_toml("asum.toml") {
                Ok(config) => {
                    eprintln!("[INFO] Loaded configuration from asum.toml");
                    return Ok(config);
                }
                Err(e) => {
                    eprintln!(
                        "[ERROR] Failed to parse asum.toml: {}. Falling back to database.",
                        e
                    );
                }
            }
        }

        Self::load_from_db(pool).await
    }

    fn load_from_toml(path: &str) -> Result<Self> {
        let content = fs::read_to_string(path)?;
        let toml_config: TomlConfig = toml::from_str(&content)?;

        Ok(AsumConfig {
            active_provider: toml_config.general.active_provider,
            max_diff_length: toml_config.general.max_diff_length,
            ai_temperature: toml_config.ai_params.temperature,
            ai_top_p: toml_config.ai_params.top_p,
            ai_num_predict: toml_config.ai_params.num_predict,
            ollama_url: toml_config.ollama.as_ref().map(|o| o.url.clone()),
            ollama_model: toml_config.ollama.as_ref().map(|o| o.model.clone()),
            gemini_api_key: toml_config.gemini.as_ref().map(|g| g.api_key.clone()),
            gemini_model: toml_config.gemini.as_ref().map(|g| g.model.clone()),
        })
    }

    async fn load_from_db(pool: &SqlitePool) -> Result<Self> {
        let active_provider = get_config(pool, "active_provider")
            .await?
            .unwrap_or_else(|| "ollama".to_string());
        let max_diff_length = get_config(pool, "max_diff_length")
            .await?
            .unwrap_or_default()
            .parse()
            .unwrap_or(1000000);
        let ai_temperature = get_config(pool, "ai_temperature")
            .await?
            .unwrap_or_default()
            .parse()
            .unwrap_or(0.1);
        let ai_top_p = get_config(pool, "ai_top_p")
            .await?
            .unwrap_or_default()
            .parse()
            .unwrap_or(0.9);
        let ai_num_predict = get_config(pool, "ai_num_predict")
            .await?
            .unwrap_or_default()
            .parse()
            .unwrap_or(250);

        let ollama_url = get_config(pool, "ollama_url").await?;
        let ollama_model = get_config(pool, "ollama_model").await?;
        let gemini_api_key = get_config(pool, "gemini_api_key").await?;
        let gemini_model = get_config(pool, "gemini_api_model").await?;

        Ok(AsumConfig {
            active_provider,
            max_diff_length,
            ai_temperature,
            ai_top_p,
            ai_num_predict,
            ollama_url,
            ollama_model,
            gemini_api_key,
            gemini_model,
        })
    }
}

pub fn verify_toml(path: &str) -> Result<()> {
    let content = fs::read_to_string(path)?;
    let _: TomlConfig = toml::from_str(&content)?;
    Ok(())
}
