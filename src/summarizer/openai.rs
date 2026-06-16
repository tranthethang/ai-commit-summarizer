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
