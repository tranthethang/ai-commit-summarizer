use crate::summarizer::{AIConfig, Summarizer, generate_prompt};
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
        let prompt = generate_prompt(&self.config.user_prompt, diff);

        let url = self
            .config
            .api_url
            .as_deref()
            .unwrap_or("http://localhost:11434/api/chat");

        let response = self
            .client
            .post(url)
            .json(&json!({
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
            }))
            .send()
            .await?;

        if !response.status().is_success() {
            anyhow::bail!("Ollama API returned error: {}", response.status());
        }

        let res_json: serde_json::Value = response.json().await?;
        let commit_msg = res_json["message"]["content"].as_str().unwrap_or("").trim();

        let final_msg = commit_msg
            .lines()
            .map(|l| l.trim())
            .filter(|l| {
                !l.is_empty()
                    && !l.to_lowercase().contains("diff to analyze")
                    && !l.to_lowercase().contains("input diff")
            })
            .collect::<Vec<_>>()
            .join("\n");

        if final_msg.is_empty() {
            anyhow::bail!("AI generated an empty or invalid message.");
        }

        Ok(final_msg)
    }
}
