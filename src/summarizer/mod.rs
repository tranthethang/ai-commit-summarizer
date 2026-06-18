//! AI summarizer module for ASUM.
//!
//! This module defines the summarization interface and factory logic
//! for various AI providers like Gemini and Ollama.

pub mod fallback;
pub mod gemini;
pub mod github;
pub mod groq;
pub mod helpers;
pub mod mistral;
pub mod ollama;
pub mod openai;
pub mod provider_config;
pub mod vertexai;

use crate::config::{AsumConfig, ProviderConfig};
use async_trait::async_trait;
use tracing::info;

use provider_config::resolve_provider_config;

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

/// Builds a concrete `Summarizer` from a `ProviderConfig` and shared settings.
fn build_summarizer_for_provider(
    provider_config: &ProviderConfig,
    config: &AsumConfig,
    verbose: bool,
) -> anyhow::Result<(String, Box<dyn Summarizer>)> {
    let (model, api_url, api_key, project_id, location, provider_name) =
        resolve_provider_config(provider_config)?;

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

    let summarizer: Box<dyn Summarizer> = match provider_name {
        "ollama" => Box::new(ollama::OllamaProvider::new(ai_config)),
        "gemini" => Box::new(gemini::GeminiProvider::new(ai_config)),
        "openai" => Box::new(openai::OpenAIProvider::new(ai_config)),
        "vertexai" => Box::new(vertexai::VertexAIProvider::new(ai_config)),
        "groq" => Box::new(groq::GroqProvider::new(ai_config)),
        "mistral" => Box::new(mistral::MistralProvider::new(ai_config)),
        "github" => Box::new(github::GithubProvider::new(ai_config)),
        _ => return Err(anyhow::anyhow!("Unknown provider: {}", provider_name)),
    };

    Ok((provider_name.to_string(), summarizer))
}

pub async fn get_summarizer(
    config: AsumConfig,
    verbose: bool,
) -> anyhow::Result<Box<dyn Summarizer>> {
    let (primary_name, primary_summarizer) =
        build_summarizer_for_provider(&config.provider, &config, verbose)?;

    info!("Using provider: {}", primary_name);

    // If no fallbacks are configured, return the primary summarizer directly
    if config.fallbacks.is_empty() {
        return Ok(primary_summarizer);
    }

    // Build fallback summarizers and wrap in FallbackSummarizer
    let fallback_names: Vec<String> = config
        .fallbacks
        .iter()
        .filter_map(|fb| resolve_provider_config(fb).ok())
        .map(|(_, _, _, _, _, name)| name.to_string())
        .collect();
    info!("Fallback providers: {:?}", fallback_names);

    let mut fallback_summarizers = Vec::with_capacity(config.fallbacks.len());
    for fb_config in &config.fallbacks {
        let (fb_name, fb_summarizer) = build_summarizer_for_provider(fb_config, &config, verbose)?;
        fallback_summarizers.push(fallback::NamedSummarizer {
            name: fb_name,
            summarizer: fb_summarizer,
        });
    }

    let fallback_orchestrator = fallback::FallbackSummarizer::new(
        fallback::NamedSummarizer {
            name: primary_name,
            summarizer: primary_summarizer,
        },
        fallback_summarizers,
    );

    Ok(Box::new(fallback_orchestrator))
}
