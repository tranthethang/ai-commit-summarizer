//! Provider configuration resolvers for ASUM.
//!
//! This module contains the validation and construction logic for resolving
//! individual provider configurations into a common tuple format used to
//! build `AIConfig` instances.

use crate::config::ProviderConfig;

/// Unified output type for all provider config builders.
/// Fields: (model, api_url, api_key, project_id, location, provider_name)
pub type ProviderConfigOutput = (
    String,
    Option<String>,
    Option<String>,
    Option<String>,
    Option<String>,
    &'static str,
);

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

fn build_github_config(
    api_key: &str,
    model: &str,
    url: &Option<String>,
) -> anyhow::Result<ProviderConfigOutput> {
    if model.is_empty() {
        anyhow::bail!("Model is required: add [github] section with 'model' in asum.toml");
    }
    if api_key.is_empty() {
        anyhow::bail!("API key is required: add 'api_key' to [github] section in asum.toml");
    }
    Ok((
        model.to_string(),
        url.clone(),
        Some(api_key.to_string()),
        None,
        None,
        "github",
    ))
}

/// Resolves a `ProviderConfig` into the common tuple used to build an `AIConfig`.
pub fn resolve_provider_config(provider: &ProviderConfig) -> anyhow::Result<ProviderConfigOutput> {
    match provider {
        ProviderConfig::Gemini {
            api_key,
            model,
            url,
        } => build_gemini_config(api_key, model, url),
        ProviderConfig::Ollama { model, url } => build_ollama_config(model, url),
        ProviderConfig::OpenAI {
            api_key,
            model,
            url,
        } => build_openai_config(api_key, model, url),
        ProviderConfig::VertexAI {
            project_id,
            location,
            model,
            access_token,
            url,
        } => build_vertexai_config(project_id, location, model, access_token, url),
        ProviderConfig::Groq {
            api_key,
            model,
            url,
        } => build_groq_config(api_key, model, url),
        ProviderConfig::Mistral {
            api_key,
            model,
            url,
        } => build_mistral_config(api_key, model, url),
        ProviderConfig::Github {
            api_key,
            model,
            url,
        } => build_github_config(api_key, model, url),
    }
}
