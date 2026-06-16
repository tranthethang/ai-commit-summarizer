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
    fn load_from_toml<P: AsRef<Path>>(path: P) -> Result<Self> {
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

pub fn verify_toml<P: AsRef<Path>>(path: P) -> Result<()> {
    let content = fs::read_to_string(path)?;
    let toml_config: TomlConfig = toml::from_str(&content)?;

    match toml_config.general.active_provider {
        Provider::Gemini => {
            let gemini = toml_config.gemini.as_ref().ok_or_else(|| {
                anyhow::anyhow!("[gemini] section is required when active_provider = \"gemini\"")
            })?;
            if gemini.model.is_empty() {
                anyhow::bail!("model in [gemini] section cannot be empty");
            }
            if gemini.api_key.is_empty() {
                anyhow::bail!("api_key in [gemini] section cannot be empty");
            }
        }
        Provider::Ollama => {
            let ollama = toml_config.ollama.as_ref().ok_or_else(|| {
                anyhow::anyhow!("[ollama] section is required when active_provider = \"ollama\"")
            })?;
            if ollama.model.is_empty() {
                anyhow::bail!("model in [ollama] section cannot be empty");
            }
            if ollama.url.is_empty() {
                anyhow::bail!("url in [ollama] section cannot be empty");
            }
        }
        Provider::OpenAI => {
            let openai = toml_config.openai.as_ref().ok_or_else(|| {
                anyhow::anyhow!("[openai] section is required when active_provider = \"openai\"")
            })?;
            if openai.model.is_empty() {
                anyhow::bail!("model in [openai] section cannot be empty");
            }
            if openai.api_key.is_empty() {
                anyhow::bail!("api_key in [openai] section cannot be empty");
            }
        }
        Provider::VertexAI => {
            let vertex = toml_config.vertexai.as_ref().ok_or_else(|| {
                anyhow::anyhow!(
                    "[vertexai] section is required when active_provider = \"vertexai\""
                )
            })?;
            if vertex.model.is_empty() {
                anyhow::bail!("model in [vertexai] section cannot be empty");
            }
            if vertex.project_id.is_empty() {
                anyhow::bail!("project_id in [vertexai] section cannot be empty");
            }
            if vertex.location.is_empty() {
                anyhow::bail!("location in [vertexai] section cannot be empty");
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_load_from_toml_full() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(
            file,
            r#"
            [general]
            active_provider = "gemini"
            max_diff_length = 1000
            git_extensions = [".rs", ".py"]

            [ai_params]
            num_predict = 100
            temperature = 0.5
            top_p = 0.9

            [gemini]
            api_key = "test_key"
            model = "gemini-pro"
            "#
        )
        .unwrap();

        let config = AsumConfig::load_from_toml(file.path()).unwrap();
        assert_eq!(
            config.provider,
            ProviderConfig::Gemini {
                api_key: "test_key".to_string(),
                model: "gemini-pro".to_string(),
                url: None,
            }
        );
        assert_eq!(config.max_diff_length, 1000);
        assert_eq!(config.git_extensions, vec![".rs", ".py"]);
    }

    #[test]
    fn test_load_from_toml_defaults() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(
            file,
            r#"
            [general]
            active_provider = "ollama"
            max_diff_length = 2000

            [ai_params]
            num_predict = 50
            temperature = 0.7
            top_p = 1.0

            [ollama]
            model = "llama3"
            url = "http://localhost:11434"
            "#
        )
        .unwrap();

        let config = AsumConfig::load_from_toml(file.path()).unwrap();
        assert_eq!(
            config.provider,
            ProviderConfig::Ollama {
                model: "llama3".to_string(),
                url: "http://localhost:11434".to_string(),
            }
        );
        // Check if default extensions are loaded
        assert!(!config.git_extensions.is_empty());
        assert!(config.git_extensions.contains(&"*.rs".to_string()));
        // Check if default system prompt is loaded
        assert!(config.system_prompt.contains("expert Git Commit Generator"));
    }

    #[test]
    fn test_verify_toml_table_driven() {
        struct TestCase {
            name: &'static str,
            content: &'static str,
            is_ok: bool,
        }

        let cases = vec![
            TestCase {
                name: "valid full config",
                content: r#"
                    [general]
                    active_provider = "ollama"
                    max_diff_length = 2000
                    [ai_params]
                    num_predict = 50
                    temperature = 0.7
                    top_p = 1.0
                    [ollama]
                    model = "llama3"
                    url = "http://localhost:11434"
                "#,
                is_ok: true,
            },
            TestCase {
                name: "missing general section",
                content: r#"
                    [ai_params]
                    num_predict = 50
                    temperature = 0.7
                    top_p = 1.0
                "#,
                is_ok: false,
            },
            TestCase {
                name: "invalid toml syntax",
                content: "invalid = [",
                is_ok: false,
            },
            TestCase {
                name: "missing required active provider section",
                content: r#"
                    [general]
                    active_provider = "gemini"
                    max_diff_length = 2000
                    [ai_params]
                    num_predict = 50
                    temperature = 0.7
                    top_p = 1.0
                "#,
                is_ok: false,
            },
            TestCase {
                name: "empty model in active provider section",
                content: r#"
                    [general]
                    active_provider = "gemini"
                    max_diff_length = 2000
                    [ai_params]
                    num_predict = 50
                    temperature = 0.7
                    top_p = 1.0
                    [gemini]
                    api_key = "test"
                    model = ""
                "#,
                is_ok: false,
            },
        ];

        for case in cases {
            let mut file = NamedTempFile::new().unwrap();
            writeln!(file, "{}", case.content).unwrap();
            let result = verify_toml(file.path());
            assert_eq!(
                result.is_ok(),
                case.is_ok,
                "Failed test case: {}",
                case.name
            );
        }
    }

    #[test]
    #[should_panic(expected = "No such file or directory")]
    fn test_load_from_toml_non_existent() {
        AsumConfig::load_from_toml("non_existent_file.toml").unwrap();
    }

    #[test]
    fn test_load_from_toml_minimal() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(
            file,
            r#"
            [general]
            active_provider = "ollama"
            max_diff_length = 500

            [ai_params]
            num_predict = 10
            temperature = 0.1
            top_p = 0.1

            [ollama]
            model = "llama3"
            url = "http://localhost:11434"
            "#
        )
        .unwrap();

        let config = AsumConfig::load_from_toml(file.path()).unwrap();
        assert_eq!(
            config.provider,
            ProviderConfig::Ollama {
                model: "llama3".to_string(),
                url: "http://localhost:11434".to_string(),
            }
        );
        assert_eq!(config.max_diff_length, 500);
        assert_eq!(config.ai_num_predict, 10);
    }

    #[test]
    fn test_load_from_toml_with_custom_prompts() {
        let mut file = NamedTempFile::new().unwrap();
        let toml_content = r#"
            [general]
            active_provider = "ollama"
            max_diff_length = 1000

            [ai_params]
            num_predict = 100
            temperature = 0.7
            top_p = 1.0

            [prompts]
            system_prompt = "Custom system prompt"
            user_prompt = "Custom user prompt: {{diff}}"

            [ollama]
            model = "llama3"
            url = "http://localhost:11434"
            "#;
        writeln!(file, "{}", toml_content).unwrap();

        let config = AsumConfig::load_from_toml(file.path()).unwrap();
        if config.user_prompt != "Custom user prompt: {{diff}}" {
            panic!(
                "CONTENT: [{}], PARSED: [{}]",
                toml_content, config.user_prompt
            );
        }
        assert_eq!(config.system_prompt, "Custom system prompt");
    }

    #[test]
    fn test_load_from_toml_with_tree_and_hunks() {
        let mut file = NamedTempFile::new().unwrap();
        let toml_content = r#"
            [general]
            active_provider = "ollama"
            max_diff_length = 1000
            enable_tree_view = false
            diff_reduction_mode = "hunk"
            max_hunks_per_file = 5

            [ai_params]
            num_predict = 100
            temperature = 0.7
            top_p = 1.0

            [ollama]
            model = "llama3"
            url = "http://localhost:11434"
            "#;
        writeln!(file, "{}", toml_content).unwrap();

        let config = AsumConfig::load_from_toml(file.path()).unwrap();
        assert!(!config.enable_tree_view);
        assert_eq!(config.diff_reduction_mode, DiffReductionMode::Hunk);
        assert_eq!(config.max_hunks_per_file, 5);
    }

    #[test]
    fn test_asum_config_load_local() {
        let dir = tempfile::tempdir().unwrap();
        let config_path = dir.path().join("asum.toml");
        let mut file = fs::File::create(config_path).unwrap();
        writeln!(
            file,
            r#"
            [general]
            active_provider = "ollama"
            max_diff_length = 1000
            [ai_params]
            num_predict = 100
            temperature = 0.7
            top_p = 1.0
            [ollama]
            model = "llama3"
            url = "http://localhost:11434"
            "#
        )
        .unwrap();

        let result = AsumConfig::load_with_search(Some(dir.path()), None);

        assert!(result.is_ok());
        assert_eq!(
            result.unwrap().provider,
            ProviderConfig::Ollama {
                model: "llama3".to_string(),
                url: "http://localhost:11434".to_string(),
            }
        );
    }

    #[test]
    fn test_asum_config_load_global() {
        let temp_home =
            std::env::temp_dir().join(format!("fake_home_global_{}", std::process::id()));
        let global_dir = temp_home.join(".asum");
        fs::create_dir_all(&global_dir).unwrap();
        let config_path = global_dir.join("asum.toml");

        let mut file = fs::File::create(&config_path).unwrap();
        writeln!(
            file,
            r#"
            [general]
            active_provider = "ollama"
            max_diff_length = 500
            [ai_params]
            num_predict = 100
            temperature = 0.7
            top_p = 1.0
            [ollama]
            model = "llama3"
            url = "http://localhost:11434"
            "#
        )
        .unwrap();

        let temp_cwd = std::env::temp_dir().join(format!("empty_cwd_{}", std::process::id()));
        fs::create_dir_all(&temp_cwd).unwrap();

        let result = AsumConfig::load_with_search(Some(&temp_cwd), Some(&temp_home));

        // Clean up temp dirs
        let _ = fs::remove_dir_all(&temp_home);
        let _ = fs::remove_dir_all(&temp_cwd);

        let config = result.expect("Should load global config");
        assert_eq!(
            config.provider,
            ProviderConfig::Ollama {
                model: "llama3".to_string(),
                url: "http://localhost:11434".to_string(),
            }
        );
        assert_eq!(config.max_diff_length, 500);
    }

    #[test]
    fn test_asum_config_load_no_config() {
        let temp_dir = std::env::temp_dir().join(format!("no_config_test_{}", std::process::id()));
        fs::create_dir_all(&temp_dir).unwrap();

        let result = AsumConfig::load_with_search(Some(&temp_dir), Some(&temp_dir));

        let _ = fs::remove_dir_all(&temp_dir);

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }
}
