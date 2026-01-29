use anyhow::{Context, Result, anyhow};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

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
    pub fn load() -> Result<Self> {
        // 1. Check local config
        let local_path = Path::new("asum.toml");
        if local_path.exists() {
            return Self::load_from_toml(local_path)
                .with_context(|| format!("Failed to load local config: {:?}", local_path));
        }

        // 2. Check global config
        let mut global_path =
            home::home_dir().ok_or_else(|| anyhow!("Could not find home directory"))?;
        global_path.push(".asum");
        global_path.push("asum.toml");

        if global_path.exists() {
            return Self::load_from_toml(&global_path)
                .with_context(|| format!("Failed to load global config: {:?}", global_path));
        }

        Err(anyhow!(
            "Configuration file 'asum.toml' not found locally or in ~/.asum/asum.toml"
        ))
    }

    fn load_from_toml<P: AsRef<Path>>(path: P) -> Result<Self> {
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
}

pub fn verify_toml<P: AsRef<Path>>(path: P) -> Result<()> {
    let content = fs::read_to_string(path)?;
    let _: TomlConfig = toml::from_str(&content)?;
    Ok(())
}
