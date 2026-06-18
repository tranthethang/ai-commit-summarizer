//! AI summarizer module for ASUM.
//!
//! This module defines the summarization interface and factory logic
//! for various AI providers like Gemini and Ollama.

pub mod gemini;
pub mod groq;
pub mod helpers;
pub mod mistral;
pub mod ollama;
pub mod openai;
pub mod vertexai;

use crate::config::{AsumConfig, ProviderConfig};
use async_trait::async_trait;
use tracing::info;

pub use helpers::{
    build_gemini_payload, build_http_client, check_response_status, clean_ai_response,
    generate_prompt, log_verbose_prompt, log_verbose_response, parse_gemini_response,
    parse_openai_response,
};

/// Configuration specifically for the AI model execution.
/// This is derived from the main `AsumConfig` but tailored for the providers.
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
    pub project_id: Option<String>,
    pub location: Option<String>,
    pub verbose: bool,
}

/// Trait defining the behavior of an AI commit summarizer.
/// Any new AI provider must implement this trait.
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait Summarizer: Send + Sync {
    /// Takes a git diff and returns a generated commit message.
    async fn summarize(&self, diff: &str) -> anyhow::Result<String>;
}

type ProviderConfigOutput = (
    String,
    Option<String>,
    Option<String>,
    Option<String>,
    Option<String>,
    &'static str,
);

/// Factory function that returns a concrete implementation of a `Summarizer`
/// based on the configuration's `active_provider`.
fn build_gemini_config(
    api_key: &str,
    model: &str,
    url: &Option<String>,
) -> anyhow::Result<ProviderConfigOutput> {
    if model.is_empty() {
        anyhow::bail!("Model is required: add [gemini] section with 'model' in asum.toml");
    }
    if api_key.is_empty() {
        anyhow::bail!("API key is required: add 'api_key' to [gemini] section in asum.toml");
    }
    Ok((
        model.to_string(),
        url.clone(),
        Some(api_key.to_string()),
        None,
        None,
        "gemini",
    ))
}

fn build_ollama_config(model: &str, url: &str) -> anyhow::Result<ProviderConfigOutput> {
    if model.is_empty() {
        anyhow::bail!("Model is required: add [ollama] section with 'model' in asum.toml");
    }
    if url.is_empty() {
        anyhow::bail!("URL is required: add 'url' to [ollama] section in asum.toml");
    }
    Ok((
        model.to_string(),
        Some(url.to_string()),
        None,
        None,
        None,
        "ollama",
    ))
}

fn build_openai_config(
    api_key: &str,
    model: &str,
    url: &Option<String>,
) -> anyhow::Result<ProviderConfigOutput> {
    if model.is_empty() {
        anyhow::bail!("Model is required: add [openai] section with 'model' in asum.toml");
    }
    if api_key.is_empty() {
        anyhow::bail!("API key is required: add 'api_key' to [openai] section in asum.toml");
    }
    Ok((
        model.to_string(),
        url.clone(),
        Some(api_key.to_string()),
        None,
        None,
        "openai",
    ))
}

fn build_vertexai_config(
    project_id: &str,
    location: &str,
    model: &str,
    access_token: &Option<String>,
    url: &Option<String>,
) -> anyhow::Result<ProviderConfigOutput> {
    if model.is_empty() {
        anyhow::bail!("Model is required: add [vertexai] section with 'model' in asum.toml");
    }
    if project_id.is_empty() {
        anyhow::bail!(
            "Project ID is required: add 'project_id' to [vertexai] section in asum.toml"
        );
    }
    if location.is_empty() {
        anyhow::bail!("Location is required: add 'location' to [vertexai] section in asum.toml");
    }
    Ok((
        model.to_string(),
        url.clone(),
        access_token.clone(),
        Some(project_id.to_string()),
        Some(location.to_string()),
        "vertexai",
    ))
}

fn build_groq_config(
    api_key: &str,
    model: &str,
    url: &Option<String>,
) -> anyhow::Result<ProviderConfigOutput> {
    if model.is_empty() {
        anyhow::bail!("Model is required: add [groq] section with 'model' in asum.toml");
    }
    if api_key.is_empty() {
        anyhow::bail!("API key is required: add 'api_key' to [groq] section in asum.toml");
    }
    Ok((
        model.to_string(),
        url.clone(),
        Some(api_key.to_string()),
        None,
        None,
        "groq",
    ))
}

fn build_mistral_config(
    api_key: &str,
    model: &str,
    url: &Option<String>,
) -> anyhow::Result<ProviderConfigOutput> {
    if model.is_empty() {
        anyhow::bail!("Model is required: add [mistral] section with 'model' in asum.toml");
    }
    if api_key.is_empty() {
        anyhow::bail!("API key is required: add 'api_key' to [mistral] section in asum.toml");
    }
    Ok((
        model.to_string(),
        url.clone(),
        Some(api_key.to_string()),
        None,
        None,
        "mistral",
    ))
}

pub async fn get_summarizer(
    config: AsumConfig,
    verbose: bool,
) -> anyhow::Result<Box<dyn Summarizer>> {
    let (model, api_url, api_key, project_id, location, provider_name) = match &config.provider {
        ProviderConfig::Gemini {
            api_key,
            model,
            url,
        } => build_gemini_config(api_key, model, url)?,
        ProviderConfig::Ollama { model, url } => build_ollama_config(model, url)?,
        ProviderConfig::OpenAI {
            api_key,
            model,
            url,
        } => build_openai_config(api_key, model, url)?,
        ProviderConfig::VertexAI {
            project_id,
            location,
            model,
            access_token,
            url,
        } => build_vertexai_config(project_id, location, model, access_token, url)?,
        ProviderConfig::Groq {
            api_key,
            model,
            url,
        } => build_groq_config(api_key, model, url)?,
        ProviderConfig::Mistral {
            api_key,
            model,
            url,
        } => build_mistral_config(api_key, model, url)?,
    };

    let ai_config = AIConfig {
        model,
        temperature: config.ai_temperature,
        top_p: config.ai_top_p,
        num_predict: config.ai_num_predict,
        api_url,
        api_key,
        system_prompt: config.system_prompt.clone(),
        user_prompt: config.user_prompt.clone(),
        project_id,
        location,
        verbose,
    };

    info!("Using provider: {}", provider_name);
    info!("Using model: {}", ai_config.model);
    if let Some(key) = ai_config.api_key.as_ref().filter(|k| !k.is_empty()) {
        let masked_key = if key.len() > 8 {
            format!("{}...{}", &key[..4], &key[key.len() - 4..])
        } else {
            "****".to_string()
        };
        info!("Using API key / Access Token: {}", masked_key);
    }

    match provider_name {
        "ollama" => Ok(Box::new(ollama::OllamaProvider::new(ai_config)) as Box<dyn Summarizer>),
        "gemini" => Ok(Box::new(gemini::GeminiProvider::new(ai_config)) as Box<dyn Summarizer>),
        "openai" => Ok(Box::new(openai::OpenAIProvider::new(ai_config)) as Box<dyn Summarizer>),
        "vertexai" => {
            Ok(Box::new(vertexai::VertexAIProvider::new(ai_config)) as Box<dyn Summarizer>)
        }
        "groq" => Ok(Box::new(groq::GroqProvider::new(ai_config)) as Box<dyn Summarizer>),
        "mistral" => Ok(Box::new(mistral::MistralProvider::new(ai_config)) as Box<dyn Summarizer>),
        _ => Err(anyhow::anyhow!("Unknown provider: {}", provider_name)),
    }
}
