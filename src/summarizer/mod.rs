pub mod gemini;
pub mod ollama;

use crate::config::AsumConfig;
use async_trait::async_trait;
use tracing::info;

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
}

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait Summarizer: Send + Sync {
    async fn summarize(&self, diff: &str) -> anyhow::Result<String>;
}

pub async fn get_summarizer(config: AsumConfig) -> anyhow::Result<Box<dyn Summarizer>> {
    let provider = config.active_provider.clone();

    let model = match provider.as_str() {
        "gemini" => config.gemini_model.clone().unwrap_or_default(),
        "ollama" => config.ollama_model.clone().unwrap_or_default(),
        _ => "".to_string(),
    };

    let ai_config = AIConfig {
        model,
        temperature: config.ai_temperature,
        top_p: config.ai_top_p,
        num_predict: config.ai_num_predict,
        api_url: config.ollama_url.clone(),
        api_key: config.gemini_api_key.clone(),
        system_prompt: config.system_prompt.clone(),
        user_prompt: config.user_prompt.clone(),
    };

    info!("Using provider: {}", provider);
    info!("Using model: {}", ai_config.model);
    if let Some(key) = ai_config.api_key.as_ref().filter(|k| !k.is_empty()) {
        let masked_key = if key.len() > 8 {
            format!("{}...{}", &key[..4], &key[key.len() - 4..])
        } else {
            "****".to_string()
        };
        info!("Using API key: {}", masked_key);
    }

    match provider.as_str() {
        "ollama" => Ok(Box::new(ollama::OllamaProvider::new(ai_config))),
        "gemini" => Ok(Box::new(gemini::GeminiProvider::new(ai_config))),
        _ => Err(anyhow::anyhow!("Unknown provider: {}", provider)),
    }
}

pub fn generate_prompt(prompt_template: &str, diff: &str) -> String {
    prompt_template.replace("{{diff}}", diff)
}

#[cfg(test)]
mod tests {
    use super::*;

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
            active_provider: "ollama".to_string(),
            max_diff_length: 1000,
            git_extensions: vec![],
            system_prompt: "sys".to_string(),
            user_prompt: "user".to_string(),
            ai_temperature: 0.7,
            ai_top_p: 1.0,
            ai_num_predict: 100,
            ollama_url: Some("http://localhost:11434".to_string()),
            ollama_model: Some("llama3".to_string()),
            gemini_api_key: None,
            gemini_model: None,
        };

        let result = get_summarizer(config).await;
        assert!(result.is_ok());
        let summarizer = result.unwrap();
        // Since we can't easily downcast Box<dyn Summarizer>, we just check it doesn't error
        // and rely on the fact that it returns Ok.
        assert!(summarizer.summarize("test").await.is_err()); // Should error because no server is running
    }

    #[tokio::test]
    async fn test_get_summarizer_gemini() {
        let config = AsumConfig {
            active_provider: "gemini".to_string(),
            max_diff_length: 1000,
            git_extensions: vec![],
            system_prompt: "sys".to_string(),
            user_prompt: "user".to_string(),
            ai_temperature: 0.7,
            ai_top_p: 1.0,
            ai_num_predict: 100,
            ollama_url: None,
            ollama_model: None,
            gemini_api_key: Some("test_key".to_string()),
            gemini_model: Some("gemini-pro".to_string()),
        };

        let result = get_summarizer(config).await;
        assert!(result.is_ok());
        let summarizer = result.unwrap();
        assert!(summarizer.summarize("test").await.is_err());
    }

    #[tokio::test]
    async fn test_get_summarizer_gemini_long_key() {
        let config = AsumConfig {
            active_provider: "gemini".to_string(),
            max_diff_length: 1000,
            git_extensions: vec![],
            system_prompt: "sys".to_string(),
            user_prompt: "user".to_string(),
            ai_temperature: 0.7,
            ai_top_p: 1.0,
            ai_num_predict: 100,
            ollama_url: None,
            ollama_model: None,
            gemini_api_key: Some("very_long_api_key_for_testing".to_string()),
            gemini_model: Some("gemini-pro".to_string()),
        };

        let result = get_summarizer(config).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_summarizer_unknown() {
        let config = AsumConfig {
            active_provider: "unknown".to_string(),
            max_diff_length: 1000,
            git_extensions: vec![],
            system_prompt: "sys".to_string(),
            user_prompt: "user".to_string(),
            ai_temperature: 0.7,
            ai_top_p: 1.0,
            ai_num_predict: 100,
            ollama_url: None,
            ollama_model: None,
            gemini_api_key: None,
            gemini_model: None,
        };

        let result = get_summarizer(config).await;
        assert!(result.is_err());
        match result {
            Err(e) => assert_eq!(e.to_string(), "Unknown provider: unknown"),
            _ => panic!("Expected error"),
        }
    }
}
