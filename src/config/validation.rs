use crate::config::Provider;
use crate::config::parser::{
    GeminiConfig, GroqConfig, MistralConfig, OllamaConfig, OpenAIConfig, TomlConfig, VertexAIConfig,
};
use anyhow::Result;
use std::fs;
use std::path::Path;

fn verify_gemini_config(gemini: Option<&GeminiConfig>) -> Result<()> {
    let config = gemini.ok_or_else(|| {
        anyhow::anyhow!("[gemini] section is required when active_provider = \"gemini\"")
    })?;
    if config.model.is_empty() {
        anyhow::bail!("model in [gemini] section cannot be empty");
    }
    if config.api_key.is_empty() {
        anyhow::bail!("api_key in [gemini] section cannot be empty");
    }
    Ok(())
}

fn verify_ollama_config(ollama: Option<&OllamaConfig>) -> Result<()> {
    let config = ollama.ok_or_else(|| {
        anyhow::anyhow!("[ollama] section is required when active_provider = \"ollama\"")
    })?;
    if config.model.is_empty() {
        anyhow::bail!("model in [ollama] section cannot be empty");
    }
    if config.url.is_empty() {
        anyhow::bail!("url in [ollama] section cannot be empty");
    }
    Ok(())
}

fn verify_openai_config(openai: Option<&OpenAIConfig>) -> Result<()> {
    let config = openai.ok_or_else(|| {
        anyhow::anyhow!("[openai] section is required when active_provider = \"openai\"")
    })?;
    if config.model.is_empty() {
        anyhow::bail!("model in [openai] section cannot be empty");
    }
    if config.api_key.is_empty() {
        anyhow::bail!("api_key in [openai] section cannot be empty");
    }
    Ok(())
}

fn verify_vertexai_config(vertex: Option<&VertexAIConfig>) -> Result<()> {
    let config = vertex.ok_or_else(|| {
        anyhow::anyhow!("[vertexai] section is required when active_provider = \"vertexai\"")
    })?;
    if config.model.is_empty() {
        anyhow::bail!("model in [vertexai] section cannot be empty");
    }
    if config.project_id.is_empty() {
        anyhow::bail!("project_id in [vertexai] section cannot be empty");
    }
    if config.location.is_empty() {
        anyhow::bail!("location in [vertexai] section cannot be empty");
    }
    Ok(())
}

fn verify_groq_config(groq: Option<&GroqConfig>) -> Result<()> {
    let config = groq.ok_or_else(|| {
        anyhow::anyhow!("[groq] section is required when active_provider = \"groq\"")
    })?;
    if config.model.is_empty() {
        anyhow::bail!("model in [groq] section cannot be empty");
    }
    if config.api_key.is_empty() {
        anyhow::bail!("api_key in [groq] section cannot be empty");
    }
    Ok(())
}

fn verify_mistral_config(mistral: Option<&MistralConfig>) -> Result<()> {
    let config = mistral.ok_or_else(|| {
        anyhow::anyhow!("[mistral] section is required when active_provider = \"mistral\"")
    })?;
    if config.model.is_empty() {
        anyhow::bail!("model in [mistral] section cannot be empty");
    }
    if config.api_key.is_empty() {
        anyhow::bail!("api_key in [mistral] section cannot be empty");
    }
    Ok(())
}

pub fn verify_toml<P: AsRef<Path>>(path: P) -> Result<()> {
    verify_toml_impl(path.as_ref())
}

fn verify_toml_impl(path: &Path) -> Result<()> {
    let content = fs::read_to_string(path)?;
    let toml_config: TomlConfig = toml::from_str(&content)?;

    match toml_config.general.active_provider {
        Provider::Gemini => verify_gemini_config(toml_config.gemini.as_ref()),
        Provider::Ollama => verify_ollama_config(toml_config.ollama.as_ref()),
        Provider::OpenAI => verify_openai_config(toml_config.openai.as_ref()),
        Provider::VertexAI => verify_vertexai_config(toml_config.vertexai.as_ref()),
        Provider::Groq => verify_groq_config(toml_config.groq.as_ref()),
        Provider::Mistral => verify_mistral_config(toml_config.mistral.as_ref()),
    }
}
