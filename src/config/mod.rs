//! Configuration management for ASUM.
//!
//! This module handles loading, parsing, and validating the application settings
//! from local or global TOML configuration files.

pub mod parser;
pub mod validation;

use anyhow::{Context, Result, anyhow};
use serde::{Deserialize, Serialize};
use std::path::Path;

pub use validation::verify_toml;

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
    #[serde(rename = "groq")]
    Groq,
    #[serde(rename = "mistral")]
    Mistral,
    #[serde(rename = "github")]
    Github,
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
    Groq {
        api_key: String,
        model: String,
        url: Option<String>,
    },
    Mistral {
        api_key: String,
        model: String,
        url: Option<String>,
    },
    Github {
        api_key: String,
        model: String,
        url: Option<String>,
    },
}

/// Main configuration structure for the application.
/// It holds settings for AI providers, git filters, and prompt templates.
#[derive(Debug, Clone, PartialEq)]
pub struct AsumConfig {
    /// The AI provider config containing model and connection details.
    pub provider: ProviderConfig,
    /// Ordered list of fallback provider configurations to try on primary failure.
    pub fallbacks: Vec<ProviderConfig>,
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
        parser::load_from_toml_impl(path.as_ref())
    }
}
