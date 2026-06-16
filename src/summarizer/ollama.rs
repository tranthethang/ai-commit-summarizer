//! Ollama AI provider for ASUM.
//!
//! This module implements the `Summarizer` trait using the Ollama API
//! (local or remote) to generate commit messages.

use crate::summarizer::{
    AIConfig, Summarizer, build_http_client, clean_ai_response, generate_prompt,
    log_verbose_prompt, log_verbose_response,
};
use async_trait::async_trait;
use reqwest::Client;
use serde_json::json;

/// Implementation of the `Summarizer` trait using a local or remote Ollama API.
pub struct OllamaProvider {
    pub config: AIConfig,
    client: Client,
}

impl OllamaProvider {
    /// Creates a new instance of `OllamaProvider`.
    pub fn new(config: AIConfig) -> Self {
        Self {
            config,
            client: build_http_client(),
        }
    }
}

#[async_trait]
impl Summarizer for OllamaProvider {
    /// Generates a commit summary using the Ollama API.
    /// Sends the system prompt and the diff to the configured model.
    async fn summarize(&self, diff: &str) -> anyhow::Result<String> {
        let prompt = generate_prompt(&self.config.user_prompt, diff);

        if self.config.verbose {
            log_verbose_prompt(&self.config.system_prompt, &prompt);
        }

        // Determine the Ollama API endpoint, defaulting to localhost
        let url = self
            .config
            .api_url
            .as_deref()
            .unwrap_or("http://localhost:11434/api/chat");

        let is_generate_api = url.ends_with("/api/generate");

        // Prepare the request payload based on the API endpoint
        let payload = if is_generate_api {
            json!({
                "model": self.config.model,
                "prompt": format!("{}\n\n{}", self.config.system_prompt, prompt),
                "stream": false,
                "options": {
                    "temperature": self.config.temperature,
                    "num_predict": self.config.num_predict,
                    "top_p": self.config.top_p
                }
            })
        } else {
            json!({
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
                "stream": false,
                "options": {
                    "temperature": self.config.temperature,
                    "num_predict": self.config.num_predict,
                    "top_p": self.config.top_p
                }
            })
        };

        // Send the request to the Ollama model
        let response = self.client.post(url).json(&payload).send().await?;

        if !response.status().is_success() {
            anyhow::bail!("Ollama API returned error: {}", response.status());
        }

        // Parse the JSON response from Ollama
        let res_text = response.text().await?;
        if self.config.verbose {
            log_verbose_response(&res_text);
        }
        let res_json: serde_json::Value = serde_json::from_str(&res_text)?;

        // Try to get content from "message.content" (chat API) or "response" (generate API)
        let commit_msg = res_json["message"]["content"]
            .as_str()
            .or_else(|| res_json["response"].as_str())
            .unwrap_or("")
            .trim();

        let final_msg = clean_ai_response(commit_msg)?;

        Ok(final_msg)
    }
}
