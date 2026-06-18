use crate::config::{AsumConfig, DiffReductionMode, Provider, ProviderConfig};
use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TomlConfig {
    pub general: GeneralConfig,
    pub prompts: Option<PromptsConfig>,
    pub ai_params: AIParamsConfig,
    pub gemini: Option<GeminiConfig>,
    pub ollama: Option<OllamaConfig>,
    pub openai: Option<OpenAIConfig>,
    pub vertexai: Option<VertexAIConfig>,
    pub groq: Option<GroqConfig>,
    pub mistral: Option<MistralConfig>,
    pub github: Option<GithubConfig>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct GeneralConfig {
    pub active_provider: Provider,
    pub max_diff_length: usize,
    pub git_extensions: Option<Vec<String>>,
    pub enable_tree_view: Option<bool>,
    pub diff_reduction_mode: Option<DiffReductionMode>,
    pub max_hunks_per_file: Option<usize>,
    pub fallbacks: Option<Vec<Provider>>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PromptsConfig {
    pub system_prompt: Option<String>,
    pub user_prompt: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AIParamsConfig {
    pub num_predict: i32,
    pub temperature: f64,
    pub top_p: f64,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct GeminiConfig {
    pub api_key: String,
    pub model: String,
    pub url: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct OllamaConfig {
    pub model: String,
    pub url: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct OpenAIConfig {
    pub api_key: String,
    pub model: String,
    pub url: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct VertexAIConfig {
    pub project_id: String,
    pub location: String,
    pub model: String,
    pub access_token: Option<String>,
    pub url: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct GroqConfig {
    pub api_key: String,
    pub model: String,
    pub url: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct MistralConfig {
    pub api_key: String,
    pub model: String,
    pub url: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct GithubConfig {
    pub api_key: String,
    pub model: String,
    pub url: Option<String>,
}

/// Builds a `ProviderConfig` for the given provider from the parsed TOML sections.
fn build_provider_config(provider: Provider, toml_config: &TomlConfig) -> Result<ProviderConfig> {
    match provider {
        Provider::Gemini => {
            let gemini = toml_config
                .gemini
                .as_ref()
                .ok_or_else(|| anyhow!("Missing [gemini] configuration section"))?;
            Ok(ProviderConfig::Gemini {
                api_key: gemini.api_key.clone(),
                model: gemini.model.clone(),
                url: gemini.url.clone(),
            })
        }
        Provider::Ollama => {
            let ollama = toml_config
                .ollama
                .as_ref()
                .ok_or_else(|| anyhow!("Missing [ollama] configuration section"))?;
            Ok(ProviderConfig::Ollama {
                model: ollama.model.clone(),
                url: ollama.url.clone(),
            })
        }
        Provider::OpenAI => {
            let openai = toml_config
                .openai
                .as_ref()
                .ok_or_else(|| anyhow!("Missing [openai] configuration section"))?;
            Ok(ProviderConfig::OpenAI {
                api_key: openai.api_key.clone(),
                model: openai.model.clone(),
                url: openai.url.clone(),
            })
        }
        Provider::VertexAI => {
            let vertex = toml_config
                .vertexai
                .as_ref()
                .ok_or_else(|| anyhow!("Missing [vertexai] configuration section"))?;
            Ok(ProviderConfig::VertexAI {
                project_id: vertex.project_id.clone(),
                location: vertex.location.clone(),
                model: vertex.model.clone(),
                access_token: vertex.access_token.clone(),
                url: vertex.url.clone(),
            })
        }
        Provider::Groq => {
            let groq = toml_config
                .groq
                .as_ref()
                .ok_or_else(|| anyhow!("Missing [groq] configuration section"))?;
            Ok(ProviderConfig::Groq {
                api_key: groq.api_key.clone(),
                model: groq.model.clone(),
                url: groq.url.clone(),
            })
        }
        Provider::Mistral => {
            let mistral = toml_config
                .mistral
                .as_ref()
                .ok_or_else(|| anyhow!("Missing [mistral] configuration section"))?;
            Ok(ProviderConfig::Mistral {
                api_key: mistral.api_key.clone(),
                model: mistral.model.clone(),
                url: mistral.url.clone(),
            })
        }
        Provider::Github => {
            let github = toml_config
                .github
                .as_ref()
                .ok_or_else(|| anyhow!("Missing [github] configuration section"))?;
            Ok(ProviderConfig::Github {
                api_key: github.api_key.clone(),
                model: github.model.clone(),
                url: github.url.clone(),
            })
        }
    }
}

pub fn load_from_toml_impl(path: &Path) -> Result<AsumConfig> {
    let content = fs::read_to_string(path)?;
    let toml_config: TomlConfig = toml::from_str(&content)?;

    let default_extensions = vec![
        "*.java", "*.php", "*.js", "*.jsx", "*.ts", "*.tsx", "*.vue", "*.svelte", "*.scss",
        "*.css", "*.html", "*.rs", "*.py", "*.pyi", "*.go", "*.c", "*.cpp", "*.h", "*.hpp", "*.cs",
        "*.rb", "*.swift", "*.kt", "*.kts", "*.dart", "*.sh", "*.sql", "*.md", "*.yml", "*.yaml",
        "*.toml", "*.json",
    ]
    .into_iter()
    .map(String::from)
    .collect();

    let default_system_prompt = r#"# SYSTEM IDENTITY
You are an expert Git Commit Generator. Your goal is to produce high-quality, professional commit messages following Conventional Commits 1.0.0.

# STRICT RULES
1. MANDATORY HEADER: Every response MUST start with `<type>(<scope>): <description>`.
2. TYPES: Only use: feat, fix, docs, style, refactor, perf, test, build, ci, chore, revert.
3. DESCRIPTION: Use imperative mood, lowercase, no period at the end (per Conventional Commits spec), max 50 chars.
4. BODY (OPTIONAL): Use bullet points ("- ") to explain "what" and "why".
5. OUTPUT: Return ONLY the raw commit message. No preamble, no backticks, no markdown blocks.

# BODY STYLE RULES
- Write in English only.
- Each bullet point MUST be a complete sentence with a clear subject, verb, and object or complement.
- Each bullet point MUST end with a period (.).
- Use active voice. Write "Add retry logic for rate-limited requests." not "Retry logic added."
- Be specific and concise. Avoid vague phrases like "various changes" or "some fixes."
- Header line does NOT end with a period (Conventional Commits spec).

# FEW-SHOT EXAMPLES

Example 1 (Simple Fix):
fix(ui): correct button alignment on mobile

Example 2 (Feature with Body):
feat(auth): implement oauth2 login flow

- Add Google and GitHub provider support to the authentication module.
- Implement secure callback handling to prevent token leakage.
- Encrypt user tokens before persisting them to the database.

Example 3 (Breaking Change):
refactor(api)!: migrate to async/await syntax

- Rewrite all controllers to use non-blocking async handlers.
- Update the database driver to support connection pooling.

BREAKING CHANGE: the synchronous API is no longer supported."#.to_string();

    let default_user_prompt = r#"[INPUT DIFF]
{{diff}}

[OUTPUT]"#
        .to_string();

    let default_enable_tree_view = true;
    let default_diff_reduction_mode = DiffReductionMode::File;
    let default_max_hunks_per_file = 3;

    let provider = build_provider_config(toml_config.general.active_provider, &toml_config)?;

    // Build fallback provider configs from the fallbacks list
    let fallbacks = match &toml_config.general.fallbacks {
        Some(fallback_providers) => {
            let mut configs = Vec::with_capacity(fallback_providers.len());
            for fb_provider in fallback_providers {
                let fb_config = build_provider_config(*fb_provider, &toml_config)?;
                configs.push(fb_config);
            }
            configs
        }
        None => Vec::new(),
    };

    Ok(AsumConfig {
        provider,
        fallbacks,
        max_diff_length: toml_config.general.max_diff_length,
        git_extensions: toml_config
            .general
            .git_extensions
            .unwrap_or(default_extensions),
        enable_tree_view: toml_config
            .general
            .enable_tree_view
            .unwrap_or(default_enable_tree_view),
        diff_reduction_mode: toml_config
            .general
            .diff_reduction_mode
            .unwrap_or(default_diff_reduction_mode),
        max_hunks_per_file: toml_config
            .general
            .max_hunks_per_file
            .unwrap_or(default_max_hunks_per_file),
        system_prompt: toml_config
            .prompts
            .as_ref()
            .and_then(|p| p.system_prompt.clone())
            .unwrap_or(default_system_prompt),
        user_prompt: toml_config
            .prompts
            .as_ref()
            .and_then(|p| p.user_prompt.clone())
            .unwrap_or(default_user_prompt),
        ai_temperature: toml_config.ai_params.temperature,
        ai_top_p: toml_config.ai_params.top_p,
        ai_num_predict: toml_config.ai_params.num_predict,
    })
}
