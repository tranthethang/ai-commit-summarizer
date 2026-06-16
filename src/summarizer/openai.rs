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

        // Reasoning models (like o1, o3-mini) do not support `max_tokens`, `temperature`,
        // or `top_p`. They require `max_completion_tokens` instead.
        let model_name = self
            .config
            .model
            .split('/')
            .next_back()
            .unwrap_or(&self.config.model);
        let is_reasoning = model_name.starts_with('o')
            && model_name
                .chars()
                .nth(1)
                .is_some_and(|c| c.is_ascii_digit() || c == '-');

        let mut payload = serde_json::json!({
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
        });

        if is_reasoning {
            payload["max_completion_tokens"] = serde_json::json!(self.config.num_predict);
        } else {
            payload["temperature"] = serde_json::json!(self.config.temperature);
            payload["top_p"] = serde_json::json!(self.config.top_p);
            payload["max_tokens"] = serde_json::json!(self.config.num_predict);
        }

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
