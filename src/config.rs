//! Configuration management for ASUM.
//!
//! This module handles loading, parsing, and validating the application settings
//! from local or global TOML configuration files.

use anyhow::{Context, Result, anyhow};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// Supported AI providers.
#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
pub enum Provider {
    #[serde(rename = "gemini")]
    Gemini,
    #[serde(rename = "ollama")]
    Ollama,
    #[serde(rename = "openai")]
    OpenAI,
    #[serde(rename = "vertexai")]
    VertexAI,
}

/// Modes for reducing diff length when it exceeds limits.
#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
pub enum DiffReductionMode {
    #[serde(rename = "file")]
    File,
    #[serde(rename = "hunk")]
    Hunk,
}

/// Provider-specific configurations.
#[derive(Debug, Clone, PartialEq)]
pub enum ProviderConfig {
    Gemini {
        api_key: String,
        model: String,
        url: Option<String>,
    },
    Ollama {
        model: String,
        url: String,
    },
    OpenAI {
        api_key: String,
        model: String,
        url: Option<String>,
    },
    VertexAI {
        project_id: String,
        location: String,
        model: String,
        access_token: Option<String>,
        url: Option<String>,
    },
}

/// Main configuration structure for the application.
/// It holds settings for AI providers, git filters, and prompt templates.
#[derive(Debug, Clone, PartialEq)]
pub struct AsumConfig {
    /// The AI provider config containing model and connection details.
    pub provider: ProviderConfig,
    /// Maximum character length of the git diff to send to the AI.
    pub max_diff_length: usize,
    /// List of file extensions to include in the git diff.
    pub git_extensions: Vec<String>,
    /// Whether to generate and prepend a tree view of staged files to the diff.
    pub enable_tree_view: bool,
    /// Mode to reduce/truncate the diff when it is too large: "file" or "hunk".
    pub diff_reduction_mode: DiffReductionMode,
    /// Max hunks per file in "hunk" reduction mode.
    pub max_hunks_per_file: usize,
    /// System-level instruction for the AI model.
    pub system_prompt: String,
    /// User-level prompt template containing the {{diff}} placeholder.
    pub user_prompt: String,
    /// Controls randomness: lower is more deterministic.
    pub ai_temperature: f64,
    /// Nucleus sampling: limits the model to the most likely tokens.
    pub ai_top_p: f64,
    /// Maximum number of tokens to generate in the response.
    pub ai_num_predict: i32,
}

/// Internal structure representing the raw TOML file layout.
#[derive(Debug, Deserialize, Serialize, Clone)]
struct TomlConfig {
    pub general: GeneralConfig,
    pub prompts: Option<PromptsConfig>,
    pub ai_params: AIParamsConfig,
    pub gemini: Option<GeminiConfig>,
    pub ollama: Option<OllamaConfig>,
    pub openai: Option<OpenAIConfig>,
    pub vertexai: Option<VertexAIConfig>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct GeneralConfig {
    pub active_provider: Provider,
    pub max_diff_length: usize,
    pub git_extensions: Option<Vec<String>>,
    pub enable_tree_view: Option<bool>,
    pub diff_reduction_mode: Option<DiffReductionMode>,
    pub max_hunks_per_file: Option<usize>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct PromptsConfig {
    pub system_prompt: Option<String>,
    pub user_prompt: Option<String>,
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
    pub url: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct OllamaConfig {
    pub model: String,
    pub url: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct OpenAIConfig {
    pub api_key: String,
    pub model: String,
    pub url: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct VertexAIConfig {
    pub project_id: String,
    pub location: String,
    pub model: String,
    pub access_token: Option<String>,
    pub url: Option<String>,
}

impl AsumConfig {
    /// Loads configuration by searching for 'asum.toml' in the current directory,
    /// then falling back to '~/.asum/asum.toml'.
    pub fn load() -> Result<Self> {
        Self::load_with_search(None, None)
    }

    /// Loads configuration with custom search paths (used for unit testing without mutating environment/process state).
    pub fn load_with_search(current_dir: Option<&Path>, home_dir: Option<&Path>) -> Result<Self> {
        // 1. Check local config
        let local_path = current_dir
            .unwrap_or_else(|| Path::new("."))
            .join("asum.toml");
        if local_path.exists() {
            return Self::load_from_toml(&local_path)
                .with_context(|| format!("Failed to load local config: {:?}", local_path));
        }

        // 2. Check global config
        let mut global_path = match home_dir {
            Some(path) => path.to_path_buf(),
            None => home::home_dir().ok_or_else(|| anyhow!("Could not find home directory"))?,
        };
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

    /// Reads and parses a TOML configuration file from the specified path.
    /// Fills in default values for missing optional fields.
    pub fn load_from_toml<P: AsRef<Path>>(path: P) -> Result<Self> {
        Self::load_from_toml_impl(path.as_ref())
    }

    fn load_from_toml_impl(path: &Path) -> Result<Self> {
        let content = fs::read_to_string(path)?;
        let toml_config: TomlConfig = toml::from_str(&content)?;

        let default_extensions = vec![
            "*.java", "*.php", "*.js", "*.jsx", "*.ts", "*.tsx", "*.vue", "*.svelte", "*.scss",
            "*.css", "*.html", "*.rs", "*.py", "*.pyi", "*.go", "*.c", "*.cpp", "*.h", "*.hpp",
            "*.cs", "*.rb", "*.swift", "*.kt", "*.kts", "*.dart", "*.sh", "*.sql", "*.md", "*.yml",
            "*.yaml", "*.toml", "*.json",
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

        let provider = match toml_config.general.active_provider {
            Provider::Gemini => {
                let gemini = toml_config
                    .gemini
                    .as_ref()
                    .ok_or_else(|| anyhow!("Missing [gemini] configuration section"))?;
                ProviderConfig::Gemini {
                    api_key: gemini.api_key.clone(),
                    model: gemini.model.clone(),
                    url: gemini.url.clone(),
                }
            }
            Provider::Ollama => {
                let ollama = toml_config
                    .ollama
                    .as_ref()
                    .ok_or_else(|| anyhow!("Missing [ollama] configuration section"))?;
                ProviderConfig::Ollama {
                    model: ollama.model.clone(),
                    url: ollama.url.clone(),
                }
            }
            Provider::OpenAI => {
                let openai = toml_config
                    .openai
                    .as_ref()
                    .ok_or_else(|| anyhow!("Missing [openai] configuration section"))?;
                ProviderConfig::OpenAI {
                    api_key: openai.api_key.clone(),
                    model: openai.model.clone(),
                    url: openai.url.clone(),
                }
            }
            Provider::VertexAI => {
                let vertex = toml_config
                    .vertexai
                    .as_ref()
                    .ok_or_else(|| anyhow!("Missing [vertexai] configuration section"))?;
                ProviderConfig::VertexAI {
                    project_id: vertex.project_id.clone(),
                    location: vertex.location.clone(),
                    model: vertex.model.clone(),
                    access_token: vertex.access_token.clone(),
                    url: vertex.url.clone(),
                }
            }
        };

        Ok(AsumConfig {
            provider,
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
}

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
    }
}
