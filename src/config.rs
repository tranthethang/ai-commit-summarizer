//! Configuration management for ASUM.
//!
//! This module handles loading, parsing, and validating the application settings
//! from local or global TOML configuration files.

use anyhow::{Context, Result, anyhow};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// Main configuration structure for the application.
/// It holds settings for AI providers, git filters, and prompt templates.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AsumConfig {
    /// The AI provider to use (e.g., "gemini" or "ollama").
    pub active_provider: String,
    /// Maximum character length of the git diff to send to the AI.
    pub max_diff_length: usize,
    /// List of file extensions to include in the git diff.
    pub git_extensions: Vec<String>,
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
    /// Base URL for the Ollama API.
    pub ollama_url: Option<String>,
    /// Model name for Ollama (e.g., "llama3").
    pub ollama_model: Option<String>,
    /// API key for Google Gemini.
    pub gemini_api_key: Option<String>,
    /// Model name for Gemini (e.g., "gemini-1.5-flash").
    pub gemini_model: Option<String>,
}

/// Internal structure representing the raw TOML file layout.
#[derive(Debug, Deserialize, Serialize, Clone)]
struct TomlConfig {
    pub general: GeneralConfig,
    pub prompts: Option<PromptsConfig>,
    pub ai_params: AIParamsConfig,
    pub gemini: Option<GeminiConfig>,
    pub ollama: Option<OllamaConfig>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct GeneralConfig {
    pub active_provider: String,
    pub max_diff_length: usize,
    pub git_extensions: Option<Vec<String>>,
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
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct OllamaConfig {
    pub model: String,
    pub url: String,
}

impl AsumConfig {
    /// Loads configuration by searching for 'asum.toml' in the current directory,
    /// then falling back to '~/.asum/asum.toml'.
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
3. DESCRIPTION: Use imperative mood, lowercase, no period, max 50 chars.
4. BODY (OPTIONAL): Use bullet points ("- ") to explain "what" and "why".
5. OUTPUT: Return ONLY the raw commit message. No preamble, no backticks, no markdown blocks.

# FEW-SHOT EXAMPLES

Example 1 (Simple Fix):
fix(ui): correct button alignment on mobile

Example 2 (Feature with Body):
feat(auth): implement oauth2 login flow

- add google and github provider support
- implement secure callback handling
- encrypt user tokens before storage

Example 3 (Breaking Change):
refactor(api)!: migrate to async/await syntax

- rewrite all controllers to be non-blocking
- update database driver to support pooling

BREAKING CHANGE: the synchronous API is no longer supported."#.to_string();

        let default_user_prompt = r#"[INPUT DIFF]
{{diff}}

[OUTPUT]"#
            .to_string();

        Ok(AsumConfig {
            active_provider: toml_config.general.active_provider,
            max_diff_length: toml_config.general.max_diff_length,
            git_extensions: toml_config
                .general
                .git_extensions
                .unwrap_or(default_extensions),
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
            ollama_url: toml_config.ollama.as_ref().map(|o| o.url.clone()),
            ollama_model: toml_config.ollama.as_ref().map(|o| o.model.clone()),
            gemini_api_key: toml_config.gemini.as_ref().map(|g| g.api_key.clone()),
            gemini_model: toml_config.gemini.as_ref().map(|g| g.model.clone()),
        })
    }
}

/// Validates that a TOML file follows the expected schema.
pub fn verify_toml<P: AsRef<Path>>(path: P) -> Result<()> {
    let content = fs::read_to_string(path)?;
    let _: TomlConfig = toml::from_str(&content)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
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
        assert_eq!(config.active_provider, "gemini");
        assert_eq!(config.max_diff_length, 1000);
        assert_eq!(config.git_extensions, vec![".rs", ".py"]);
        assert_eq!(config.gemini_api_key.unwrap(), "test_key");
        assert_eq!(config.gemini_model.unwrap(), "gemini-pro");
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
            "#
        )
        .unwrap();

        let config = AsumConfig::load_from_toml(file.path()).unwrap();
        assert_eq!(config.active_provider, "ollama");
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
            "#
        )
        .unwrap();

        let config = AsumConfig::load_from_toml(file.path()).unwrap();
        assert_eq!(config.active_provider, "ollama");
        assert_eq!(config.max_diff_length, 500);
        assert_eq!(config.ai_num_predict, 10);
        assert!(config.ollama_url.is_none());
        assert!(config.gemini_api_key.is_none());
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
            "#
        )
        .unwrap();

        let _guard = crate::test_utils::TEST_MUTEX.lock().unwrap();
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(dir.path()).unwrap();

        let result = AsumConfig::load();

        std::env::set_current_dir(original_dir).unwrap();

        assert!(result.is_ok());
        assert_eq!(result.unwrap().active_provider, "ollama");
    }

    #[test]
    fn test_asum_config_load_global() {
        let _guard = crate::test_utils::TEST_MUTEX.lock().unwrap();
        let temp_home =
            std::env::temp_dir().join(format!("fake_home_global_{}", std::process::id()));
        fs::create_dir_all(temp_home.join(".asum")).unwrap();
        let config_path = temp_home.join(".asum").join("asum.toml");

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
            "#
        )
        .unwrap();

        let temp_cwd = std::env::temp_dir().join(format!("empty_cwd_{}", std::process::id()));
        fs::create_dir_all(&temp_cwd).unwrap();

        let old_cwd = env::current_dir().unwrap();
        env::set_current_dir(&temp_cwd).unwrap();

        let old_home = env::var("HOME").ok();
        unsafe { env::set_var("HOME", &temp_home) };

        let result = AsumConfig::load();

        // Restore
        env::set_current_dir(old_cwd).unwrap();
        if let Some(val) = old_home {
            unsafe { env::set_var("HOME", val) };
        } else {
            unsafe { env::remove_var("HOME") };
        }

        let config = result.expect("Should load global config");
        assert_eq!(config.active_provider, "ollama");
        assert_eq!(config.max_diff_length, 500);
    }

    #[test]
    fn test_asum_config_load_no_config() {
        let _guard = crate::test_utils::TEST_MUTEX.lock().unwrap();
        let temp_dir = std::env::temp_dir().join(format!("no_config_test_{}", std::process::id()));
        fs::create_dir_all(&temp_dir).unwrap();

        let old_cwd = env::current_dir().unwrap();
        env::set_current_dir(&temp_dir).unwrap();

        let old_home = env::var("HOME").ok();
        unsafe { env::set_var("HOME", &temp_dir) }; // Point HOME to empty temp dir

        let result = AsumConfig::load();

        env::set_current_dir(old_cwd).unwrap();
        if let Some(val) = old_home {
            unsafe { env::set_var("HOME", val) };
        } else {
            unsafe { env::remove_var("HOME") };
        }

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }
}
