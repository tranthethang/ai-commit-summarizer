use crate::summarizer::{AIConfig, Summarizer};
use async_trait::async_trait;
use reqwest::Client;
use serde_json::json;

pub struct OllamaProvider {
    config: AIConfig,
    client: Client,
}

impl OllamaProvider {
    pub fn new(config: AIConfig) -> Self {
        Self {
            config,
            client: Client::new(),
        }
    }
}

#[async_trait]
impl Summarizer for OllamaProvider {
    async fn summarize(&self, diff: &str) -> anyhow::Result<String> {
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

        let url = self
            .config
            .api_url
            .as_deref()
            .unwrap_or("http://localhost:11434/api/generate");

        let response = self
            .client
            .post(url)
            .json(&json!({
                "model": self.config.model,
                "prompt": prompt,
                "stream": false,
                "options": {
                    "temperature": self.config.temperature,
                    "num_predict": self.config.num_predict,
                    "top_p": self.config.top_p
                }
            }))
            .send()
            .await?;

        if !response.status().is_success() {
            anyhow::bail!("Ollama API returned error: {}", response.status());
        }

        let res_json: serde_json::Value = response.json().await?;
        let commit_msg = res_json["response"].as_str().unwrap_or("").trim();

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
