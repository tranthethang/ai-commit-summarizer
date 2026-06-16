//! Vertex AI provider for ASUM.
//!
//! This module implements the `Summarizer` trait using the Google Cloud Vertex AI API
//! to generate commit messages.

use crate::summarizer::{
    AIConfig, Summarizer, build_gemini_payload, build_http_client, check_response_status,
    generate_prompt, log_verbose_prompt, parse_gemini_response,
};
use anyhow::Context;
use async_trait::async_trait;
use reqwest::Client;
use std::process::Command;

/// Implementation of the `Summarizer` trait using the Vertex AI API.
pub struct VertexAIProvider {
    pub config: AIConfig,
    client: Client,
}

impl VertexAIProvider {
    /// Creates a new instance of `VertexAIProvider`.
    pub fn new(config: AIConfig) -> Self {
        Self {
            config,
            client: build_http_client(),
        }
    }

    /// Retrieves the access token, either from config or via gcloud CLI.
    pub fn get_access_token(&self) -> anyhow::Result<String> {
        if let Some(token) = self
            .config
            .api_key
            .as_deref()
            .filter(|t| !t.trim().is_empty())
        {
            return Ok(token.trim().to_string());
        }

        // Fallback to gcloud
        let output = Command::new("gcloud")
            .args(["auth", "print-access-token"])
            .output()
            .context("Failed to execute gcloud auth print-access-token. Is gcloud installed?")?;

        if !output.status.success() {
            let err = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("gcloud failed to get access token: {}", err);
        }

        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }

    /// Builds the request URL for Vertex AI API.
    pub fn build_url(&self, project_id: &str, location: &str) -> String {
        self.config.api_url.clone().unwrap_or_else(|| {
            let host = if location == "global" {
                "aiplatform.googleapis.com".to_string()
            } else {
                format!("{}-aiplatform.googleapis.com", location)
            };
            format!(
                "https://{}/v1/projects/{}/locations/{}/publishers/google/models/{}:generateContent",
                host, project_id, location, self.config.model
            )
        })
    }
}

#[async_trait]
impl Summarizer for VertexAIProvider {
    /// Generates a commit summary using the Vertex AI API.
    async fn summarize(&self, diff: &str) -> anyhow::Result<String> {
        let project_id = self
            .config
            .project_id
            .as_deref()
            .context("Vertex AI project_id is missing")?;
        let location = self
            .config
            .location
            .as_deref()
            .context("Vertex AI location is missing")?;

        let access_token = self.get_access_token()?;

        let base_url = self.build_url(project_id, location);

        let prompt = generate_prompt(&self.config.user_prompt, diff);

        if self.config.verbose {
            log_verbose_prompt(&self.config.system_prompt, &prompt);
        }

        // Payload structure is the same as Gemini
        let payload = build_gemini_payload(&self.config.system_prompt, &prompt, &self.config);

        let response = self
            .client
            .post(&base_url)
            .bearer_auth(access_token)
            .json(&payload)
            .send()
            .await?;

        let response = check_response_status(response, "Vertex AI").await?;

        let res_text = response.text().await?;
        parse_gemini_response(&res_text, self.config.verbose)
    }
}
