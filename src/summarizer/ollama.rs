//! Ollama AI provider for ASUM.
//!
//! This module implements the `Summarizer` trait using the Ollama API
//! (local or remote) to generate commit messages.

use crate::summarizer::{AIConfig, Summarizer, generate_prompt};
use async_trait::async_trait;
use reqwest::Client;
use serde_json::json;

/// Implementation of the `Summarizer` trait using a local or remote Ollama API.
pub struct OllamaProvider {
    config: AIConfig,
    client: Client,
}

impl OllamaProvider {
    /// Creates a new instance of `OllamaProvider`.
    pub fn new(config: AIConfig) -> Self {
        Self {
            config,
            client: Client::new(),
        }
    }
}

#[async_trait]
impl Summarizer for OllamaProvider {
    /// Generates a commit summary using the Ollama API.
    /// Sends the system prompt and the diff to the configured model.
    async fn summarize(&self, diff: &str) -> anyhow::Result<String> {
        let prompt = generate_prompt(&self.config.user_prompt, diff);

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
        let res_json: serde_json::Value = response.json().await?;

        // Try to get content from "message.content" (chat API) or "response" (generate API)
        let commit_msg = res_json["message"]["content"]
            .as_str()
            .or_else(|| res_json["response"].as_str())
            .unwrap_or("")
            .trim();

        // Post-process the generated message to remove boilerplate text
        // that AI models sometimes include in their responses.
        let final_msg = commit_msg
            .lines()
            .map(|l| l.trim())
            .filter(|l| {
                // Remove empty lines and lines that echo the input diff instructions
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::summarizer::AIConfig;

    #[test]
    fn test_ollama_provider_new() {
        let ai_config = AIConfig {
            model: "llama3".to_string(),
            temperature: 0.7,
            top_p: 1.0,
            num_predict: 100,
            api_url: None,
            api_key: None,
            system_prompt: "sys".to_string(),
            user_prompt: "user".to_string(),
        };
        let provider = OllamaProvider::new(ai_config);
        assert_eq!(provider.config.model, "llama3");
    }

    #[test]
    fn test_ollama_filtering() {
        let commit_msg = "feat: add feature\n\nInput diff to analyze:\nSome diff\nActual message";
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

        assert!(final_msg.contains("feat: add feature"));
        assert!(final_msg.contains("Actual message"));
        assert!(!final_msg.to_lowercase().contains("input diff"));
    }

    #[tokio::test]
    async fn test_ollama_summarize_fail() {
        let ai_config = AIConfig {
            model: "llama3".to_string(),
            temperature: 0.7,
            top_p: 1.0,
            num_predict: 100,
            api_url: Some("http://localhost:1".to_string()), // Invalid port
            api_key: None,
            system_prompt: "sys".to_string(),
            user_prompt: "user".to_string(),
        };
        let provider = OllamaProvider::new(ai_config);
        let result = provider.summarize("diff").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_ollama_summarize_success() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let url = format!("http://{}", addr);

        tokio::spawn(async move {
            let (mut socket, _) = listener.accept().await.unwrap();
            let mut buf = [0; 1024];
            let _ = tokio::io::AsyncReadExt::read(&mut socket, &mut buf)
                .await
                .unwrap();

            let response = "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\n\r\n{\"message\": {\"content\": \"feat: success\"}}";
            tokio::io::AsyncWriteExt::write_all(&mut socket, response.as_bytes())
                .await
                .unwrap();
        });

        let ai_config = AIConfig {
            model: "llama3".to_string(),
            temperature: 0.7,
            top_p: 1.0,
            num_predict: 100,
            api_url: Some(url),
            api_key: None,
            system_prompt: "sys".to_string(),
            user_prompt: "user".to_string(),
        };
        let provider = OllamaProvider::new(ai_config);
        let result = provider.summarize("diff").await.unwrap();
        assert_eq!(result, "feat: success");
    }

    #[tokio::test]
    async fn test_ollama_summarize_generate_endpoint_success() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let url = format!("http://{}/api/generate", addr);

        tokio::spawn(async move {
            let (mut socket, _) = listener.accept().await.unwrap();
            let mut buf = [0; 1024];
            let _ = tokio::io::AsyncReadExt::read(&mut socket, &mut buf)
                .await
                .unwrap();

            // Ollama /api/generate returns "response" field
            let response = "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\n\r\n{\"response\": \"feat: success from generate\"}";
            tokio::io::AsyncWriteExt::write_all(&mut socket, response.as_bytes())
                .await
                .unwrap();
        });

        let ai_config = AIConfig {
            model: "llama3".to_string(),
            temperature: 0.7,
            top_p: 1.0,
            num_predict: 100,
            api_url: Some(url),
            api_key: None,
            system_prompt: "sys".to_string(),
            user_prompt: "user".to_string(),
        };
        let provider = OllamaProvider::new(ai_config);
        let result = provider.summarize("diff").await.unwrap();
        assert_eq!(result, "feat: success from generate");
    }
}
