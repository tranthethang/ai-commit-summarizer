use asum::summarizer::Summarizer;
use asum::summarizer::fallback::{FallbackSummarizer, NamedSummarizer};
use async_trait::async_trait;

/// A mock summarizer that always succeeds with a predefined message.
struct SuccessSummarizer {
    message: String,
}

#[async_trait]
impl Summarizer for SuccessSummarizer {
    async fn summarize(&self, _diff: &str) -> anyhow::Result<String> {
        Ok(self.message.clone())
    }
}

/// A mock summarizer that always fails with a predefined error.
struct FailSummarizer {
    error_message: String,
}

#[async_trait]
impl Summarizer for FailSummarizer {
    async fn summarize(&self, _diff: &str) -> anyhow::Result<String> {
        Err(anyhow::anyhow!("{}", self.error_message))
    }
}

#[tokio::test]
async fn test_fallback_primary_succeeds_no_fallback_triggered() {
    let primary = NamedSummarizer {
        name: "primary".to_string(),
        summarizer: Box::new(SuccessSummarizer {
            message: "feat(core): primary result".to_string(),
        }),
    };
    let fallbacks = vec![NamedSummarizer {
        name: "fallback1".to_string(),
        summarizer: Box::new(SuccessSummarizer {
            message: "feat(core): fallback1 result".to_string(),
        }),
    }];

    let orchestrator = FallbackSummarizer::new(primary, fallbacks);
    let result = orchestrator.summarize("test diff").await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "feat(core): primary result");
}

#[tokio::test]
async fn test_fallback_primary_fails_first_fallback_succeeds() {
    let primary = NamedSummarizer {
        name: "openai".to_string(),
        summarizer: Box::new(FailSummarizer {
            error_message: "API rate limited".to_string(),
        }),
    };
    let fallbacks = vec![
        NamedSummarizer {
            name: "github".to_string(),
            summarizer: Box::new(SuccessSummarizer {
                message: "fix(auth): github result".to_string(),
            }),
        },
        NamedSummarizer {
            name: "groq".to_string(),
            summarizer: Box::new(SuccessSummarizer {
                message: "fix(auth): groq result".to_string(),
            }),
        },
    ];

    let orchestrator = FallbackSummarizer::new(primary, fallbacks);
    let result = orchestrator.summarize("test diff").await;

    assert!(result.is_ok());
    // The first fallback should succeed, so we get github's result
    assert_eq!(result.unwrap(), "fix(auth): github result");
}

#[tokio::test]
async fn test_fallback_primary_and_first_fallback_fail_second_succeeds() {
    let primary = NamedSummarizer {
        name: "openai".to_string(),
        summarizer: Box::new(FailSummarizer {
            error_message: "OpenAI timeout".to_string(),
        }),
    };
    let fallbacks = vec![
        NamedSummarizer {
            name: "github".to_string(),
            summarizer: Box::new(FailSummarizer {
                error_message: "GitHub rate limited".to_string(),
            }),
        },
        NamedSummarizer {
            name: "groq".to_string(),
            summarizer: Box::new(SuccessSummarizer {
                message: "refactor(api): groq result".to_string(),
            }),
        },
    ];

    let orchestrator = FallbackSummarizer::new(primary, fallbacks);
    let result = orchestrator.summarize("test diff").await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "refactor(api): groq result");
}

#[tokio::test]
async fn test_fallback_all_providers_fail_returns_combined_error() {
    let primary = NamedSummarizer {
        name: "openai".to_string(),
        summarizer: Box::new(FailSummarizer {
            error_message: "OpenAI connection refused".to_string(),
        }),
    };
    let fallbacks = vec![
        NamedSummarizer {
            name: "github".to_string(),
            summarizer: Box::new(FailSummarizer {
                error_message: "GitHub 503 Service Unavailable".to_string(),
            }),
        },
        NamedSummarizer {
            name: "groq".to_string(),
            summarizer: Box::new(FailSummarizer {
                error_message: "Groq rate limited".to_string(),
            }),
        },
    ];

    let orchestrator = FallbackSummarizer::new(primary, fallbacks);
    let result = orchestrator.summarize("test diff").await;

    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("All providers failed"));
    assert!(error_msg.contains("openai"));
    assert!(error_msg.contains("github: GitHub 503 Service Unavailable"));
    assert!(error_msg.contains("groq: Groq rate limited"));
}

#[tokio::test]
async fn test_fallback_no_fallbacks_configured_primary_succeeds() {
    let primary = NamedSummarizer {
        name: "gemini".to_string(),
        summarizer: Box::new(SuccessSummarizer {
            message: "docs(readme): update readme".to_string(),
        }),
    };
    let fallbacks: Vec<NamedSummarizer> = vec![];

    let orchestrator = FallbackSummarizer::new(primary, fallbacks);
    let result = orchestrator.summarize("test diff").await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "docs(readme): update readme");
}

#[tokio::test]
async fn test_fallback_no_fallbacks_configured_primary_fails() {
    let primary = NamedSummarizer {
        name: "gemini".to_string(),
        summarizer: Box::new(FailSummarizer {
            error_message: "Gemini API error".to_string(),
        }),
    };
    let fallbacks: Vec<NamedSummarizer> = vec![];

    let orchestrator = FallbackSummarizer::new(primary, fallbacks);
    let result = orchestrator.summarize("test diff").await;

    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("All providers failed"));
    assert!(error_msg.contains("gemini"));
    assert!(error_msg.contains("0 fallback(s) exhausted"));
}
