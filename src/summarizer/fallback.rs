//! Fallback orchestration for AI providers.
//!
//! This module provides a `FallbackSummarizer` that wraps a primary summarizer
//! and a list of fallback summarizers. When the primary fails, it automatically
//! retries with each fallback in order until one succeeds or all are exhausted.

use crate::summarizer::Summarizer;
use async_trait::async_trait;
use tracing::{error, info, warn};

/// A named summarizer that pairs a provider name with its implementation.
pub struct NamedSummarizer {
    pub name: String,
    pub summarizer: Box<dyn Summarizer>,
}

/// Orchestrates failover across multiple AI providers.
///
/// When `summarize()` is called, it tries the primary provider first.
/// On failure, it logs the error and retries sequentially with each
/// fallback provider until one succeeds or all are exhausted.
pub struct FallbackSummarizer {
    primary: NamedSummarizer,
    fallbacks: Vec<NamedSummarizer>,
}

impl FallbackSummarizer {
    /// Creates a new `FallbackSummarizer` with the given primary and fallback providers.
    pub fn new(primary: NamedSummarizer, fallbacks: Vec<NamedSummarizer>) -> Self {
        Self { primary, fallbacks }
    }
}

#[async_trait]
impl Summarizer for FallbackSummarizer {
    /// Attempts summarization with the primary provider first, then falls back
    /// to each configured provider in order. Collects all errors and returns
    /// a combined error if every provider fails.
    async fn summarize(&self, diff: &str) -> anyhow::Result<String> {
        // Try the primary provider first
        match self.primary.summarizer.summarize(diff).await {
            Ok(result) => return Ok(result),
            Err(e) => {
                error!(
                    "Primary provider '{}' failed: {}. Trying fallbacks...",
                    self.primary.name, e
                );
            }
        }

        // Try each fallback provider in order
        let mut errors = Vec::new();
        for (index, named) in self.fallbacks.iter().enumerate() {
            info!(
                "Retrying with fallback provider '{}' ({}/{})...",
                named.name,
                index + 1,
                self.fallbacks.len()
            );

            match named.summarizer.summarize(diff).await {
                Ok(result) => {
                    info!("Fallback provider '{}' succeeded.", named.name);
                    return Ok(result);
                }
                Err(e) => {
                    warn!("Fallback provider '{}' failed: {}", named.name, e);
                    errors.push(format!("{}: {}", named.name, e));
                }
            }
        }

        // All providers failed
        Err(anyhow::anyhow!(
            "All providers failed. Primary '{}' and {} fallback(s) exhausted.\nErrors:\n- {}",
            self.primary.name,
            self.fallbacks.len(),
            errors.join("\n- ")
        ))
    }
}
