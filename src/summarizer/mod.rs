//! AI summarizer module for ASUM.
//!
//! This module defines the summarization interface and factory logic
//! for various AI providers like Gemini and Ollama.

pub mod gemini;
pub mod ollama;
pub mod openai;
pub mod vertexai;

use crate::config::{AsumConfig, ProviderConfig};
use async_trait::async_trait;
use std::time::Duration;
use tracing::info;

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

/// Creates a new HTTP client with a 120-second timeout.
pub fn build_http_client() -> reqwest::Client {
    reqwest::Client::builder()
        .timeout(Duration::from_secs(120))
        .build()
        .expect("Failed to build HTTP client")
}

/// Factory function that returns a concrete implementation of a `Summarizer`
/// based on the configuration's `active_provider`.
pub async fn get_summarizer(
    config: AsumConfig,
    verbose: bool,
) -> anyhow::Result<Box<dyn Summarizer>> {
    let (model, api_url, api_key, project_id, location, provider_name) = match &config.provider {
        ProviderConfig::Gemini {
            api_key,
            model,
            url,
        } => {
            if model.is_empty() {
                anyhow::bail!("Model is required: add [gemini] section with 'model' in asum.toml");
            }
            if api_key.is_empty() {
                anyhow::bail!(
                    "API key is required: add 'api_key' to [gemini] section in asum.toml"
                );
            }
            (
                model.clone(),
                url.clone(),
                Some(api_key.clone()),
                None,
                None,
                "gemini",
            )
        }
        ProviderConfig::Ollama { model, url } => {
            if model.is_empty() {
                anyhow::bail!("Model is required: add [ollama] section with 'model' in asum.toml");
            }
            if url.is_empty() {
                anyhow::bail!("URL is required: add 'url' to [ollama] section in asum.toml");
            }
            (model.clone(), Some(url.clone()), None, None, None, "ollama")
        }
        ProviderConfig::OpenAI {
            api_key,
            model,
            url,
        } => {
            if model.is_empty() {
                anyhow::bail!("Model is required: add [openai] section with 'model' in asum.toml");
            }
            if api_key.is_empty() {
                anyhow::bail!(
                    "API key is required: add 'api_key' to [openai] section in asum.toml"
                );
            }
            (
                model.clone(),
                url.clone(),
                Some(api_key.clone()),
                None,
                None,
                "openai",
            )
        }
        ProviderConfig::VertexAI {
            project_id,
            location,
            model,
            access_token,
            url,
        } => {
            if model.is_empty() {
                anyhow::bail!(
                    "Model is required: add [vertexai] section with 'model' in asum.toml"
                );
            }
            if project_id.is_empty() {
                anyhow::bail!(
                    "Project ID is required: add 'project_id' to [vertexai] section in asum.toml"
                );
            }
            if location.is_empty() {
                anyhow::bail!(
                    "Location is required: add 'location' to [vertexai] section in asum.toml"
                );
            }
            (
                model.clone(),
                url.clone(),
                access_token.clone(),
                Some(project_id.clone()),
                Some(location.clone()),
                "vertexai",
            )
        }
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
        _ => Err(anyhow::anyhow!("Unknown provider: {}", provider_name)),
    }
}

/// Injects the git diff into the provided prompt template.
/// Replaces the `{{diff}}` placeholder with the actual diff content.
pub fn generate_prompt(prompt_template: &str, diff: &str) -> String {
    prompt_template.replace("{{diff}}", diff)
}

/// Cleans the raw AI response by removing empty lines and
/// lines that echo input diff instructions.
pub fn clean_ai_response(raw: &str) -> anyhow::Result<String> {
    let cleaned = raw
        .lines()
        .map(|l| l.trim())
        .filter(|l| {
            !l.is_empty()
                && !l.to_lowercase().contains("diff to analyze")
                && !l.to_lowercase().contains("input diff")
        })
        .collect::<Vec<_>>()
        .join("\n");

    if cleaned.is_empty() {
        anyhow::bail!("AI generated an empty or invalid message.");
    }

    Ok(cleaned)
}

/// Logs the system and user prompts in verbose mode.
pub fn log_verbose_prompt(system_prompt: &str, user_prompt: &str) {
    eprintln!("================ PROMPT ================");
    eprintln!("*** System Prompt ***\n{}", system_prompt);
    eprintln!("*** User Prompt ***\n{}", user_prompt);
    eprintln!("========================================");
}

/// Logs the raw API response JSON in verbose mode.
pub fn log_verbose_response(body: &str) {
    eprintln!("================ RESPONSE JSON ================");
    if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(body) {
        if let Ok(pretty) = serde_json::to_string_pretty(&parsed) {
            eprintln!("{}", pretty);
        } else {
            eprintln!("{}", body);
        }
    } else {
        eprintln!("{}", body);
    }
    eprintln!("===============================================");
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::DiffReductionMode;

    #[test]
    fn test_generate_prompt_table_driven() {
        struct TestCase {
            template: &'static str,
            diff: &'static str,
            expected: &'static str,
        }

        let cases = vec![
            TestCase {
                template: "Changes: {{diff}}",
                diff: "fix bug",
                expected: "Changes: fix bug",
            },
            TestCase {
                template: "{{diff}} only",
                diff: "feat",
                expected: "feat only",
            },
            TestCase {
                template: "no placeholder",
                diff: "anything",
                expected: "no placeholder",
            },
        ];

        for case in cases {
            assert_eq!(generate_prompt(case.template, case.diff), case.expected);
        }
    }

    #[test]
    fn test_api_key_masking_table_driven() {
        struct TestCase {
            key: &'static str,
            expected: &'static str,
        }

        let cases = vec![
            TestCase {
                key: "",
                expected: "****",
            },
            TestCase {
                key: "123",
                expected: "****",
            },
            TestCase {
                key: "12345678",
                expected: "****",
            },
            TestCase {
                key: "123456789",
                expected: "1234...6789",
            },
            TestCase {
                key: "abcdefghijkl",
                expected: "abcd...ijkl",
            },
        ];

        for case in cases {
            let masked = if case.key.len() > 8 {
                format!("{}...{}", &case.key[..4], &case.key[case.key.len() - 4..])
            } else {
                "****".to_string()
            };
            assert_eq!(masked, case.expected, "Failed for key: {}", case.key);
        }
    }

    #[tokio::test]
    async fn test_get_summarizer_ollama() {
        let config = AsumConfig {
            provider: ProviderConfig::Ollama {
                model: "llama3".to_string(),
                url: "http://localhost:11434".to_string(),
            },
            max_diff_length: 1000,
            git_extensions: vec![],
            enable_tree_view: true,
            diff_reduction_mode: DiffReductionMode::File,
            max_hunks_per_file: 3,
            system_prompt: "sys".to_string(),
            user_prompt: "user".to_string(),
            ai_temperature: 0.7,
            ai_top_p: 1.0,
            ai_num_predict: 100,
        };

        let result = get_summarizer(config, false).await;
        assert!(result.is_ok());
        let summarizer = result.unwrap();
        assert!(summarizer.summarize("test").await.is_err());
    }

    #[tokio::test]
    async fn test_get_summarizer_gemini() {
        let config = AsumConfig {
            provider: ProviderConfig::Gemini {
                api_key: "test_key".to_string(),
                model: "gemini-pro".to_string(),
                url: None,
            },
            max_diff_length: 1000,
            git_extensions: vec![],
            enable_tree_view: true,
            diff_reduction_mode: DiffReductionMode::File,
            max_hunks_per_file: 3,
            system_prompt: "sys".to_string(),
            user_prompt: "user".to_string(),
            ai_temperature: 0.7,
            ai_top_p: 1.0,
            ai_num_predict: 100,
        };

        let result = get_summarizer(config, false).await;
        assert!(result.is_ok());
        let summarizer = result.unwrap();
        assert!(summarizer.summarize("test").await.is_err());
    }

    #[tokio::test]
    async fn test_get_summarizer_gemini_long_key() {
        let config = AsumConfig {
            provider: ProviderConfig::Gemini {
                api_key: "very_long_api_key_for_testing".to_string(),
                model: "gemini-pro".to_string(),
                url: None,
            },
            max_diff_length: 1000,
            git_extensions: vec![],
            enable_tree_view: true,
            diff_reduction_mode: DiffReductionMode::File,
            max_hunks_per_file: 3,
            system_prompt: "sys".to_string(),
            user_prompt: "user".to_string(),
            ai_temperature: 0.7,
            ai_top_p: 1.0,
            ai_num_predict: 100,
        };

        let result = get_summarizer(config, false).await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_clean_ai_response() {
        // Test normal message cleaning
        let raw = "  fix: some bug  \n\n  Input diff:\n  Result description  ";
        let cleaned = clean_ai_response(raw).unwrap();
        assert_eq!(cleaned, "fix: some bug\nResult description");

        // Test filtering "diff to analyze"
        let raw_diff = "feat: add feature\nDiff to analyze:\nDone";
        let cleaned_diff = clean_ai_response(raw_diff).unwrap();
        assert_eq!(cleaned_diff, "feat: add feature\nDone");

        // Test empty/invalid message returns error
        let raw_empty = "\n\n  Input diff  \n  Diff to analyze  \n";
        assert!(clean_ai_response(raw_empty).is_err());
    }
}
