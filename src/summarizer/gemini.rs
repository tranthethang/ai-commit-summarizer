//! Gemini AI provider for ASUM.
//!
//! This module implements the `Summarizer` trait using Google's Gemini API
//! to generate commit messages.

use crate::summarizer::{
    AIConfig, Summarizer, build_http_client, clean_ai_response, generate_prompt,
    log_verbose_prompt, log_verbose_response,
};
use anyhow::Context;
use async_trait::async_trait;
use reqwest::Client;
use serde_json::json;
use std::time::Duration;
use tokio::time::sleep;
use tracing::warn;

/// Implementation of the `Summarizer` trait using Google's Gemini API.
pub struct GeminiProvider {
    config: AIConfig,
    client: Client,
    base_url: String,
}

impl GeminiProvider {
    /// Creates a new instance of `GeminiProvider` with the default base URL.
    pub fn new(config: AIConfig) -> Self {
        let base_url = config
            .api_url
            .clone()
            .unwrap_or_else(|| "https://generativelanguage.googleapis.com".to_string());
        Self {
            config,
            client: build_http_client(),
            base_url,
        }
    }

    #[cfg(test)]
    pub fn new_with_url(config: AIConfig, url: String) -> Self {
        Self {
            config,
            client: build_http_client(),
            base_url: url,
        }
    }
}

#[async_trait]
impl Summarizer for GeminiProvider {
    /// Generates a commit summary using the Gemini API.
    /// Implements retry logic for rate limits and cleans the output message.
    async fn summarize(&self, diff: &str) -> anyhow::Result<String> {
        let api_key = self
            .config
            .api_key
            .as_deref()
            .context("Gemini API key is missing")?;

        let prompt = generate_prompt(&self.config.user_prompt, diff);

        if self.config.verbose {
            log_verbose_prompt(&self.config.system_prompt, &prompt);
        }

        let url = format!(
            "{}/v1beta/models/{}:generateContent",
            self.base_url, self.config.model
        );

        // Implementation of exponential backoff for rate limiting (HTTP 429)
        let mut retries = 0;
        let max_retries = 3;
        let mut backoff = 2;

        let response = loop {
            let res = self
                .client
                .post(&url)
                .header("x-goog-api-key", api_key)
                .json(&json!({
                    "system_instruction": {
                        "parts": [{
                            "text": &self.config.system_prompt
                        }]
                    },
                    "contents": [{
                        "role": "user",
                        "parts": [{
                            "text": &prompt
                        }]
                    }],
                    "generationConfig": {
                        "temperature": self.config.temperature,
                        "topP": self.config.top_p,
                        "maxOutputTokens": self.config.num_predict,
                    }
                }))
                .send()
                .await?;

            if res.status() == reqwest::StatusCode::TOO_MANY_REQUESTS && retries < max_retries {
                retries += 1;
                warn!(
                    "Gemini API rate limited (429). Retrying in {}s... (Attempt {}/{})",
                    backoff, retries, max_retries
                );
                sleep(Duration::from_secs(backoff)).await;
                backoff *= 2;
                continue;
            }

            if !res.status().is_success() {
                let status = res.status();
                let error_text = res
                    .text()
                    .await
                    .unwrap_or_else(|_| "Unknown error".to_string());
                anyhow::bail!("Gemini API returned error: {} - {}", status, error_text);
            }

            break res;
        };

        // Parse the JSON response from Gemini
        let res_text = response.text().await?;
        if self.config.verbose {
            log_verbose_response(&res_text);
        }
        let res_json: serde_json::Value = serde_json::from_str(&res_text)?;

        // Gemini response structure: candidates[0].content.parts[0].text
        let commit_msg = res_json["candidates"][0]["content"]["parts"][0]["text"]
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
    fn test_gemini_provider_new() {
        let ai_config = AIConfig {
            model: "gemini-pro".to_string(),
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
        let provider = GeminiProvider::new(ai_config);
        assert_eq!(provider.config.model, "gemini-pro");
    }

    #[tokio::test]
    async fn test_gemini_summarize_missing_key() {
        let ai_config = AIConfig {
            model: "gemini-pro".to_string(),
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
        let provider = GeminiProvider::new(ai_config);
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
    async fn test_gemini_summarize_success() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let url = format!("http://{}", addr);

        tokio::spawn(async move {
            let (mut socket, _) = listener.accept().await.unwrap();
            let mut buf = [0; 32768];
            let _ = tokio::io::AsyncReadExt::read(&mut socket, &mut buf)
                .await
                .unwrap();

            let response = "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\n\r\n{\"candidates\": [{\"content\": {\"parts\": [{\"text\": \"fix: gemini success\"}]}}]}";
            tokio::io::AsyncWriteExt::write_all(&mut socket, response.as_bytes())
                .await
                .unwrap();
        });

        let ai_config = AIConfig {
            model: "gemini-pro".to_string(),
            temperature: 0.7,
            top_p: 1.0,
            num_predict: 100,
            api_url: None,
            api_key: Some("test_key".to_string()),
            system_prompt: "sys".to_string(),
            user_prompt: "user".to_string(),
            project_id: None,
            location: None,
            verbose: false,
        };
        let provider = GeminiProvider::new_with_url(ai_config, url);
        let result = provider.summarize("diff").await.unwrap();
        assert_eq!(result, "fix: gemini success");
    }
}
