//! OpenAI API provider for ASUM.
//!
//! This module implements the `Summarizer` trait using the OpenAI Chat API
//! (or compatible endpoints) to generate commit messages.

use crate::summarizer::{
    AIConfig, Summarizer, build_http_client, clean_ai_response, generate_prompt,
    log_verbose_prompt, log_verbose_response,
};
use anyhow::Context;
use async_trait::async_trait;
use reqwest::Client;
use serde_json::json;

/// Implementation of the `Summarizer` trait using the OpenAI API.
pub struct OpenAIProvider {
    config: AIConfig,
    client: Client,
    base_url: String,
}

impl OpenAIProvider {
    /// Creates a new instance of `OpenAIProvider`.
    pub fn new(config: AIConfig) -> Self {
        let base_url = config
            .api_url
            .clone()
            .unwrap_or_else(|| "https://api.openai.com/v1/chat/completions".to_string());
        Self {
            config,
            client: build_http_client(),
            base_url,
        }
    }
}

#[async_trait]
impl Summarizer for OpenAIProvider {
    /// Generates a commit summary using the OpenAI API.
    async fn summarize(&self, diff: &str) -> anyhow::Result<String> {
        let api_key = self
            .config
            .api_key
            .as_deref()
            .context("OpenAI API key is missing")?;

        let prompt = generate_prompt(&self.config.user_prompt, diff);

        if self.config.verbose {
            log_verbose_prompt(&self.config.system_prompt, &prompt);
        }

        let payload = json!({
            "model": self.config.model,
            "messages": [
                {
                    "role": "system",
                    "content": &self.config.system_prompt
                },
                {
                    "role": "user",
                    "content": &prompt
                }
            ],
            "temperature": self.config.temperature,
            "top_p": self.config.top_p,
            "max_tokens": self.config.num_predict,
        });

        let response = self
            .client
            .post(&self.base_url)
            .bearer_auth(api_key)
            .json(&payload)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            anyhow::bail!("OpenAI API returned error: {} - {}", status, error_text);
        }

        let res_text = response.text().await?;
        if self.config.verbose {
            log_verbose_response(&res_text);
        }
        let res_json: serde_json::Value = serde_json::from_str(&res_text)?;

        let commit_msg = res_json["choices"][0]["message"]["content"]
            .as_str()
            .unwrap_or("")
            .trim();

        let final_msg = clean_ai_response(commit_msg)?;

        Ok(final_msg)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::summarizer::AIConfig;

    #[test]
    fn test_openai_provider_new() {
        let ai_config = AIConfig {
            model: "gpt-4".to_string(),
            temperature: 0.7,
            top_p: 1.0,
            num_predict: 100,
            api_url: None,
            api_key: Some("key".to_string()),
            system_prompt: "sys".to_string(),
            user_prompt: "user".to_string(),
            project_id: None,
            location: None,
            verbose: false,
        };
        let provider = OpenAIProvider::new(ai_config);
        assert_eq!(provider.config.model, "gpt-4");
        assert_eq!(
            provider.base_url,
            "https://api.openai.com/v1/chat/completions"
        );
    }

    #[tokio::test]
    async fn test_openai_summarize_missing_key() {
        let ai_config = AIConfig {
            model: "gpt-4".to_string(),
            temperature: 0.7,
            top_p: 1.0,
            num_predict: 100,
            api_url: None,
            api_key: None,
            system_prompt: "sys".to_string(),
            user_prompt: "user".to_string(),
            project_id: None,
            location: None,
            verbose: false,
        };
        let provider = OpenAIProvider::new(ai_config);
        let result = provider.summarize("diff").await;
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("API key is missing")
        );
    }

    #[tokio::test]
    async fn test_openai_summarize_success() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let url = format!("http://{}", addr);

        tokio::spawn(async move {
            let (mut socket, _) = listener.accept().await.unwrap();
            let mut buf = [0; 32768];
            let _ = tokio::io::AsyncReadExt::read(&mut socket, &mut buf)
                .await
                .unwrap();

            let response = "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\n\r\n{\"choices\": [{\"message\": {\"content\": \"fix: openai success\"}}]}";
            tokio::io::AsyncWriteExt::write_all(&mut socket, response.as_bytes())
                .await
                .unwrap();
        });

        let ai_config = AIConfig {
            model: "gpt-4".to_string(),
            temperature: 0.7,
            top_p: 1.0,
            num_predict: 100,
            api_url: Some(url),
            api_key: Some("test_key".to_string()),
            system_prompt: "sys".to_string(),
            user_prompt: "user".to_string(),
            project_id: None,
            location: None,
            verbose: false,
        };
        let provider = OpenAIProvider::new(ai_config);
        let result = provider.summarize("diff").await.unwrap();
        assert_eq!(result, "fix: openai success");
    }

    #[tokio::test]
    async fn test_openai_summarize_api_error() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let url = format!("http://{}", addr);

        tokio::spawn(async move {
            let (mut socket, _) = listener.accept().await.unwrap();
            let mut buf = [0; 32768];
            let _ = tokio::io::AsyncReadExt::read(&mut socket, &mut buf)
                .await
                .unwrap();

            let response =
                "HTTP/1.1 400 Bad Request\r\nContent-Type: text/plain\r\n\r\nBad Request";
            tokio::io::AsyncWriteExt::write_all(&mut socket, response.as_bytes())
                .await
                .unwrap();
        });

        let ai_config = AIConfig {
            model: "gpt-4".to_string(),
            temperature: 0.7,
            top_p: 1.0,
            num_predict: 100,
            api_url: Some(url),
            api_key: Some("test_key".to_string()),
            system_prompt: "sys".to_string(),
            user_prompt: "user".to_string(),
            project_id: None,
            location: None,
            verbose: false,
        };
        let provider = OpenAIProvider::new(ai_config);
        let result = provider.summarize("diff").await;
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("returned error: 400")
        );
    }

    #[tokio::test]
    async fn test_openai_summarize_empty_response() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let url = format!("http://{}", addr);

        tokio::spawn(async move {
            let (mut socket, _) = listener.accept().await.unwrap();
            let mut buf = [0; 32768];
            let _ = tokio::io::AsyncReadExt::read(&mut socket, &mut buf)
                .await
                .unwrap();

            let response = "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\n\r\n{\"choices\": [{\"message\": {\"content\": \"\\n  \\n\"}}]}";
            tokio::io::AsyncWriteExt::write_all(&mut socket, response.as_bytes())
                .await
                .unwrap();
        });

        let ai_config = AIConfig {
            model: "gpt-4".to_string(),
            temperature: 0.7,
            top_p: 1.0,
            num_predict: 100,
            api_url: Some(url),
            api_key: Some("test_key".to_string()),
            system_prompt: "sys".to_string(),
            user_prompt: "user".to_string(),
            project_id: None,
            location: None,
            verbose: false,
        };
        let provider = OpenAIProvider::new(ai_config);
        let result = provider.summarize("diff").await;
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("empty or invalid message")
        );
    }
}
