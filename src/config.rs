use anyhow::{Context, Result, anyhow};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AsumConfig {
    pub active_provider: String,
    pub max_diff_length: usize,
    pub git_extensions: Vec<String>,
    pub system_prompt: String,
    pub user_prompt: String,
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

pub fn verify_toml<P: AsRef<Path>>(path: P) -> Result<()> {
    let content = fs::read_to_string(path)?;
    let _: TomlConfig = toml::from_str(&content)?;
    Ok(())
}
