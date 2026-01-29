use crate::summarizer::{AIConfig, Summarizer};
use anyhow::Context;
use async_trait::async_trait;
use reqwest::Client;
use serde_json::json;
use std::time::Duration;
use tokio::time::sleep;

pub struct GeminiProvider {
    config: AIConfig,
    client: Client,
}

impl GeminiProvider {
    pub fn new(config: AIConfig) -> Self {
        Self {
            config,
            client: Client::new(),
        }
    }
}

#[async_trait]
impl Summarizer for GeminiProvider {
    async fn summarize(&self, diff: &str) -> anyhow::Result<String> {
        let api_key = self
            .config
            .api_key
            .as_deref()
            .context("Gemini API key is missing")?;

        let prompt = format!(
            "SYSTEM: You are a professional Git Commit Generator.
RULES:
1. Output ONLY a bulleted list of changes.
2. Max 10 items.
3. Use Conventional Commits format (feat:, fix:, refactor:, etc.).
4. NO code snippets, NO preamble, NO explanations, NO echo of the input.
5. Use plain text only, NO emojis.

EXAMPLE OUTPUT:
- feat: add ollama api integration for rust
- fix: handle null pointer in java controller
- refactor: optimize scss variables for dark mode

INPUT DIFF:
{}
",
            diff
        );

        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
            self.config.model, api_key
        );

        let mut retries = 0;
        let max_retries = 3;
        let mut backoff = 2;

        let response = loop {
            let res = self
                .client
                .post(&url)
                .json(&json!({
                    "contents": [{
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
                eprintln!(
                    "[WARN] Gemini API rate limited (429). Retrying in {}s... (Attempt {}/{})",
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

        let res_json: serde_json::Value = response.json().await?;

        // Gemini response structure: candidates[0].content.parts[0].text
        let commit_msg = res_json["candidates"][0]["content"]["parts"][0]["text"]
            .as_str()
            .unwrap_or("")
            .trim();

        let final_msg = commit_msg
            .lines()
            .map(|l| l.trim())
            .filter(|l| {
                !l.is_empty()
                    && !l.to_lowercase().contains("diff to analyze")
                    && !l.to_lowercase().contains("input diff")
                    && l.starts_with("- ")
            })
            .collect::<Vec<_>>()
            .join("\n");

        if final_msg.is_empty() {
            anyhow::bail!("AI generated an empty or invalid message.");
        }

        Ok(final_msg)
    }
}
