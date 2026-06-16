//! OpenAI API provider for ASUM.
//!
//! This module implements the `Summarizer` trait using the OpenAI Chat API
//! (or compatible endpoints) to generate commit messages.

use crate::summarizer::{
    AIConfig, Summarizer, build_http_client, check_response_status, generate_prompt,
    log_verbose_prompt, parse_openai_response,
};
use anyhow::Context;
use async_trait::async_trait;
use reqwest::Client;

/// Implementation of the `Summarizer` trait using the OpenAI API.
pub struct OpenAIProvider {
    pub config: AIConfig,
    client: Client,
    pub base_url: String,
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

        let response = self
            .client
            .post(&self.base_url)
            .bearer_auth(api_key)
            .json(&payload)
            .send()
            .await?;

        let response = check_response_status(response, "OpenAI API").await?;

        let res_text = response.text().await?;
        parse_openai_response(&res_text, self.config.verbose)
    }
}
