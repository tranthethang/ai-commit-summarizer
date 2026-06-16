//! Gemini AI provider for ASUM.
//!
//! This module implements the `Summarizer` trait using Google's Gemini API
//! to generate commit messages.

use crate::summarizer::{
    AIConfig, Summarizer, build_gemini_payload, build_http_client, check_response_status,
    generate_prompt, log_verbose_prompt, parse_gemini_response,
};
use anyhow::Context;
use async_trait::async_trait;
use reqwest::Client;
use std::time::Duration;
use tokio::time::sleep;
use tracing::warn;

/// Implementation of the `Summarizer` trait using Google's Gemini API.
pub struct GeminiProvider {
    pub config: AIConfig,
    client: Client,
    pub base_url: String,
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

        let payload = build_gemini_payload(&self.config.system_prompt, &prompt, &self.config);

        // Implementation of exponential backoff for rate limiting (HTTP 429)
        let mut retries = 0;
        let max_retries = 3;
        let mut backoff = 2;

        let response = loop {
            let res = self
                .client
                .post(&url)
                .header("x-goog-api-key", api_key)
                .json(&payload)
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

            let res = check_response_status(res, "Gemini API").await?;
            break res;
        };

        // Parse the JSON response from Gemini
        let res_text = response.text().await?;
        parse_gemini_response(&res_text, self.config.verbose)
    }
}
