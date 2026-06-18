//! Mistral AI provider for ASUM.
//!
//! This module implements the `Summarizer` trait using Mistral's API
//! (which is compatible with OpenAI's format) to generate commit messages.

use crate::summarizer::{
    AIConfig, Summarizer, build_http_client, check_response_status, generate_prompt,
    log_verbose_prompt, parse_openai_response,
};
use anyhow::Context;
use async_trait::async_trait;
use reqwest::Client;
use std::time::Duration;
use tokio::time::sleep;
use tracing::warn;

/// Implementation of the `Summarizer` trait using Mistral's API.
pub struct MistralProvider {
    pub config: AIConfig,
    client: Client,
    pub base_url: String,
}

impl MistralProvider {
    /// Creates a new instance of `MistralProvider`.
    pub fn new(config: AIConfig) -> Self {
        let base_url = config
            .api_url
            .clone()
            .unwrap_or_else(|| "https://api.mistral.ai/v1/chat/completions".to_string());
        Self {
            config,
            client: build_http_client(),
            base_url,
        }
    }
}

#[async_trait]
impl Summarizer for MistralProvider {
    /// Generates a commit summary using the Mistral API.
    /// Implements retry logic for rate limits and cleans the output message.
    async fn summarize(&self, diff: &str) -> anyhow::Result<String> {
        let api_key = self
            .config
            .api_key
            .as_deref()
            .context("Mistral API key is missing")?;

        let prompt = generate_prompt(&self.config.user_prompt, diff);

        if self.config.verbose {
            log_verbose_prompt(&self.config.system_prompt, &prompt);
        }

        let payload = serde_json::json!({
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

        // Implementation of exponential backoff for rate limiting (HTTP 429)
        let mut retries = 0;
        let max_retries = 3;
        let mut backoff = 2;

        let response = loop {
            let res = self
                .client
                .post(&self.base_url)
                .bearer_auth(api_key)
                .json(&payload)
                .send()
                .await?;

            if res.status() == reqwest::StatusCode::TOO_MANY_REQUESTS && retries < max_retries {
                retries += 1;
                warn!(
                    "Mistral API rate limited (429). Retrying in {}s... (Attempt {}/{})",
                    backoff, retries, max_retries
                );
                sleep(Duration::from_secs(backoff)).await;
                backoff *= 2;
                continue;
            }

            let res = check_response_status(res, "Mistral API").await?;
            break res;
        };

        let res_text = response.text().await?;
        parse_openai_response(&res_text, self.config.verbose)
    }
}
