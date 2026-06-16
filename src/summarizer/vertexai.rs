//! Vertex AI provider for ASUM.
//!
//! This module implements the `Summarizer` trait using the Google Cloud Vertex AI API
//! to generate commit messages.

use crate::summarizer::{AIConfig, Summarizer, generate_prompt};
use anyhow::Context;
use async_trait::async_trait;
use reqwest::Client;
use serde_json::json;
use std::process::Command;

/// Implementation of the `Summarizer` trait using the Vertex AI API.
pub struct VertexAIProvider {
    config: AIConfig,
    client: Client,
}

impl VertexAIProvider {
    /// Creates a new instance of `VertexAIProvider`.
    pub fn new(config: AIConfig) -> Self {
        Self {
            config,
            client: Client::new(),
        }
    }

    /// Retrieves the access token, either from config or via gcloud CLI.
    fn get_access_token(&self) -> anyhow::Result<String> {
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
    fn build_url(&self, project_id: &str, location: &str) -> String {
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
            eprintln!("================ PROMPT ================");
            eprintln!("*** System Prompt ***\n{}", self.config.system_prompt);
            eprintln!("*** User Prompt ***\n{}", prompt);
            eprintln!("========================================");
        }

        // Payload structure is the same as Gemini
        let payload = json!({
            "system_instruction": {
                "parts": [{
                    "text": &self.config.system_prompt
                }]
            },
            "contents": [{
                "role": "user",
                "parts": [{
                    "text": &prompt
                }]
            }],
            "generationConfig": {
                "temperature": self.config.temperature,
                "topP": self.config.top_p,
                "maxOutputTokens": self.config.num_predict,
            }
        });

        let response = self
            .client
            .post(&base_url)
            .bearer_auth(access_token)
            .json(&payload)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            anyhow::bail!("Vertex AI returned error: {} - {}", status, error_text);
        }

        let res_text = response.text().await?;
        if self.config.verbose {
            eprintln!("================ RESPONSE JSON ================");
            if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&res_text) {
                if let Ok(pretty) = serde_json::to_string_pretty(&parsed) {
                    eprintln!("{}", pretty);
                } else {
                    eprintln!("{}", res_text);
                }
            } else {
                eprintln!("{}", res_text);
            }
            eprintln!("===============================================");
        }
        let res_json: serde_json::Value = serde_json::from_str(&res_text)?;

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
    fn test_vertexai_provider_new() {
        let ai_config = AIConfig {
            model: "gemini-1.5-pro".to_string(),
            temperature: 0.7,
            top_p: 1.0,
            num_predict: 100,
            api_url: None,
            api_key: None,
            system_prompt: "sys".to_string(),
            user_prompt: "user".to_string(),
            project_id: Some("my-project".to_string()),
            location: Some("us-central1".to_string()),
            verbose: false,
        };
        let provider = VertexAIProvider::new(ai_config);
        assert_eq!(provider.config.model, "gemini-1.5-pro");
    }

    #[test]
    fn test_vertexai_get_access_token_from_config() {
        let ai_config = AIConfig {
            model: "gemini-1.5-pro".to_string(),
            temperature: 0.7,
            top_p: 1.0,
            num_predict: 100,
            api_url: None,
            api_key: Some("static_token".to_string()),
            system_prompt: "sys".to_string(),
            user_prompt: "user".to_string(),
            project_id: None,
            location: None,
            verbose: false,
        };
        let provider = VertexAIProvider::new(ai_config);
        let token = provider.get_access_token().unwrap();
        assert_eq!(token, "static_token");
    }

    #[tokio::test]
    async fn test_vertexai_summarize_missing_project_id() {
        let ai_config = AIConfig {
            model: "gemini-1.5-pro".to_string(),
            temperature: 0.7,
            top_p: 1.0,
            num_predict: 100,
            api_url: None,
            api_key: Some("token".to_string()),
            system_prompt: "sys".to_string(),
            user_prompt: "user".to_string(),
            project_id: None,
            location: Some("us-central1".to_string()),
            verbose: false,
        };
        let provider = VertexAIProvider::new(ai_config);
        let result = provider.summarize("diff").await;
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("project_id is missing")
        );
    }

    #[tokio::test]
    async fn test_vertexai_summarize_missing_location() {
        let ai_config = AIConfig {
            model: "gemini-1.5-pro".to_string(),
            temperature: 0.7,
            top_p: 1.0,
            num_predict: 100,
            api_url: None,
            api_key: Some("token".to_string()),
            system_prompt: "sys".to_string(),
            user_prompt: "user".to_string(),
            project_id: Some("proj".to_string()),
            location: None,
            verbose: false,
        };
        let provider = VertexAIProvider::new(ai_config);
        let result = provider.summarize("diff").await;
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("location is missing")
        );
    }

    #[tokio::test]
    async fn test_vertexai_summarize_success() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let url = format!("http://{}", addr);

        tokio::spawn(async move {
            let (mut socket, _) = listener.accept().await.unwrap();
            let mut buf = [0; 32768];
            let _ = tokio::io::AsyncReadExt::read(&mut socket, &mut buf)
                .await
                .unwrap();

            let response = "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\n\r\n{\"candidates\": [{\"content\": {\"parts\": [{\"text\": \"fix: vertex success\"}]}}]}";
            tokio::io::AsyncWriteExt::write_all(&mut socket, response.as_bytes())
                .await
                .unwrap();
        });

        let ai_config = AIConfig {
            model: "gemini-1.5-pro".to_string(),
            temperature: 0.7,
            top_p: 1.0,
            num_predict: 100,
            api_url: Some(url),
            api_key: Some("static_token".to_string()),
            system_prompt: "sys".to_string(),
            user_prompt: "user".to_string(),
            project_id: Some("proj".to_string()),
            location: Some("loc".to_string()),
            verbose: false,
        };
        let provider = VertexAIProvider::new(ai_config);
        let result = provider.summarize("diff").await.unwrap();
        assert_eq!(result, "fix: vertex success");
    }

    #[tokio::test]
    async fn test_vertexai_summarize_api_error() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let url = format!("http://{}", addr);

        tokio::spawn(async move {
            let (mut socket, _) = listener.accept().await.unwrap();
            let mut buf = [0; 32768];
            let _ = tokio::io::AsyncReadExt::read(&mut socket, &mut buf)
                .await
                .unwrap();

            let response = "HTTP/1.1 403 Forbidden\r\nContent-Type: text/plain\r\n\r\nForbidden";
            tokio::io::AsyncWriteExt::write_all(&mut socket, response.as_bytes())
                .await
                .unwrap();
        });

        let ai_config = AIConfig {
            model: "gemini-1.5-pro".to_string(),
            temperature: 0.7,
            top_p: 1.0,
            num_predict: 100,
            api_url: Some(url),
            api_key: Some("static_token".to_string()),
            system_prompt: "sys".to_string(),
            user_prompt: "user".to_string(),
            project_id: Some("proj".to_string()),
            location: Some("loc".to_string()),
            verbose: false,
        };
        let provider = VertexAIProvider::new(ai_config);
        let result = provider.summarize("diff").await;
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("returned error: 403")
        );
    }

    #[test]
    fn test_vertexai_build_url() {
        let ai_config_regional = AIConfig {
            model: "gemini-3.5-flash".to_string(),
            temperature: 0.7,
            top_p: 1.0,
            num_predict: 100,
            api_url: None,
            api_key: Some("static_token".to_string()),
            system_prompt: "sys".to_string(),
            user_prompt: "user".to_string(),
            project_id: Some("proj".to_string()),
            location: Some("us-central1".to_string()),
            verbose: false,
        };
        let provider_regional = VertexAIProvider::new(ai_config_regional);
        let url_regional = provider_regional.build_url("proj", "us-central1");
        assert_eq!(
            url_regional,
            "https://us-central1-aiplatform.googleapis.com/v1/projects/proj/locations/us-central1/publishers/google/models/gemini-3.5-flash:generateContent"
        );

        let ai_config_global = AIConfig {
            model: "gemini-3.5-flash".to_string(),
            temperature: 0.7,
            top_p: 1.0,
            num_predict: 100,
            api_url: None,
            api_key: Some("static_token".to_string()),
            system_prompt: "sys".to_string(),
            user_prompt: "user".to_string(),
            project_id: Some("proj".to_string()),
            location: Some("global".to_string()),
            verbose: false,
        };
        let provider_global = VertexAIProvider::new(ai_config_global);
        let url_global = provider_global.build_url("proj", "global");
        assert_eq!(
            url_global,
            "https://aiplatform.googleapis.com/v1/projects/proj/locations/global/publishers/google/models/gemini-3.5-flash:generateContent"
        );
    }
}
