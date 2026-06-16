use asum::config::{AsumConfig, DiffReductionMode, ProviderConfig};
use asum::summarizer::{Summarizer, clean_ai_response, generate_prompt, get_summarizer};

mockall::mock! {
    pub Summarizer {}
    #[async_trait::async_trait]
    impl Summarizer for Summarizer {
        async fn summarize(&self, diff: &str) -> anyhow::Result<String>;
    }
}

#[test]
fn test_generate_prompt_table_driven() {
    struct TestCase {
        template: &'static str,
        diff: &'static str,
        expected: &'static str,
    }

    let cases = vec![
        TestCase {
            template: "Changes: {{diff}}",
            diff: "fix bug",
            expected: "Changes: fix bug",
        },
        TestCase {
            template: "{{diff}} only",
            diff: "feat",
            expected: "feat only",
        },
        TestCase {
            template: "no placeholder",
            diff: "anything",
            expected: "no placeholder",
        },
    ];

    for case in cases {
        assert_eq!(generate_prompt(case.template, case.diff), case.expected);
    }
}

#[test]
fn test_api_key_masking_table_driven() {
    struct TestCase {
        key: &'static str,
        expected: &'static str,
    }

    let cases = vec![
        TestCase {
            key: "",
            expected: "****",
        },
        TestCase {
            key: "123",
            expected: "****",
        },
        TestCase {
            key: "12345678",
            expected: "****",
        },
        TestCase {
            key: "123456789",
            expected: "1234...6789",
        },
        TestCase {
            key: "abcdefghijkl",
            expected: "abcd...ijkl",
        },
    ];

    for case in cases {
        let masked = if case.key.len() > 8 {
            format!("{}...{}", &case.key[..4], &case.key[case.key.len() - 4..])
        } else {
            "****".to_string()
        };
        assert_eq!(masked, case.expected, "Failed for key: {}", case.key);
    }
}

#[tokio::test]
async fn test_get_summarizer_ollama() {
    let config = AsumConfig {
        provider: ProviderConfig::Ollama {
            model: "llama3".to_string(),
            url: "http://localhost:11434".to_string(),
        },
        max_diff_length: 1000,
        git_extensions: vec![],
        enable_tree_view: true,
        diff_reduction_mode: DiffReductionMode::File,
        max_hunks_per_file: 3,
        system_prompt: "sys".to_string(),
        user_prompt: "user".to_string(),
        ai_temperature: 0.7,
        ai_top_p: 1.0,
        ai_num_predict: 100,
    };

    let result = get_summarizer(config, false).await;
    assert!(result.is_ok());
    let summarizer = result.unwrap();
    assert!(summarizer.summarize("test").await.is_err());
}

#[tokio::test]
async fn test_get_summarizer_gemini() {
    let config = AsumConfig {
        provider: ProviderConfig::Gemini {
            api_key: "test_key".to_string(),
            model: "gemini-pro".to_string(),
            url: None,
        },
        max_diff_length: 1000,
        git_extensions: vec![],
        enable_tree_view: true,
        diff_reduction_mode: DiffReductionMode::File,
        max_hunks_per_file: 3,
        system_prompt: "sys".to_string(),
        user_prompt: "user".to_string(),
        ai_temperature: 0.7,
        ai_top_p: 1.0,
        ai_num_predict: 100,
    };

    let result = get_summarizer(config, false).await;
    assert!(result.is_ok());
    let summarizer = result.unwrap();
    assert!(summarizer.summarize("test").await.is_err());
}

#[tokio::test]
async fn test_get_summarizer_gemini_long_key() {
    let config = AsumConfig {
        provider: ProviderConfig::Gemini {
            api_key: "very_long_api_key_for_testing".to_string(),
            model: "gemini-pro".to_string(),
            url: None,
        },
        max_diff_length: 1000,
        git_extensions: vec![],
        enable_tree_view: true,
        diff_reduction_mode: DiffReductionMode::File,
        max_hunks_per_file: 3,
        system_prompt: "sys".to_string(),
        user_prompt: "user".to_string(),
        ai_temperature: 0.7,
        ai_top_p: 1.0,
        ai_num_predict: 100,
    };

    let result = get_summarizer(config, false).await;
    assert!(result.is_ok());
}

#[test]
fn test_clean_ai_response() {
    // Test normal message cleaning
    let raw = "  fix: some bug  \n\n  Input diff:\n  Result description  ";
    let cleaned = clean_ai_response(raw).unwrap();
    assert_eq!(cleaned, "fix: some bug\nResult description");

    // Test filtering \"diff to analyze\"
    let raw_diff = "feat: add feature\nDiff to analyze:\nDone";
    let cleaned_diff = clean_ai_response(raw_diff).unwrap();
    assert_eq!(cleaned_diff, "feat: add feature\nDone");

    // Test empty/invalid message returns error
    let raw_empty = "\n\n  Input diff  \n  Diff to analyze  \n";
    assert!(clean_ai_response(raw_empty).is_err());
}

#[tokio::test]
async fn test_get_summarizer_openai() {
    let config = AsumConfig {
        provider: ProviderConfig::OpenAI {
            api_key: "test_key".to_string(),
            model: "gpt-4".to_string(),
            url: None,
        },
        max_diff_length: 1000,
        git_extensions: vec![],
        enable_tree_view: true,
        diff_reduction_mode: DiffReductionMode::File,
        max_hunks_per_file: 3,
        system_prompt: "sys".to_string(),
        user_prompt: "user".to_string(),
        ai_temperature: 0.7,
        ai_top_p: 1.0,
        ai_num_predict: 100,
    };

    let result = get_summarizer(config, false).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_get_summarizer_vertexai() {
    let config = AsumConfig {
        provider: ProviderConfig::VertexAI {
            project_id: "test".to_string(),
            location: "us-central1".to_string(),
            model: "gemini-pro".to_string(),
            access_token: None,
            url: None,
        },
        max_diff_length: 1000,
        git_extensions: vec![],
        enable_tree_view: true,
        diff_reduction_mode: DiffReductionMode::File,
        max_hunks_per_file: 3,
        system_prompt: "sys".to_string(),
        user_prompt: "user".to_string(),
        ai_temperature: 0.7,
        ai_top_p: 1.0,
        ai_num_predict: 100,
    };

    let result = get_summarizer(config, false).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_get_summarizer_missing_fields() {
    let config = AsumConfig {
        provider: ProviderConfig::Ollama {
            model: "".to_string(),
            url: "http://localhost:11434".to_string(),
        },
        max_diff_length: 1000,
        git_extensions: vec![],
        enable_tree_view: true,
        diff_reduction_mode: DiffReductionMode::File,
        max_hunks_per_file: 3,
        system_prompt: "sys".to_string(),
        user_prompt: "user".to_string(),
        ai_temperature: 0.7,
        ai_top_p: 1.0,
        ai_num_predict: 100,
    };
    assert!(get_summarizer(config, false).await.is_err());

    let config2 = AsumConfig {
        provider: ProviderConfig::Gemini {
            model: "gemini".to_string(),
            api_key: "".to_string(),
            url: None,
        },
        max_diff_length: 1000,
        git_extensions: vec![],
        enable_tree_view: true,
        diff_reduction_mode: DiffReductionMode::File,
        max_hunks_per_file: 3,
        system_prompt: "sys".to_string(),
        user_prompt: "user".to_string(),
        ai_temperature: 0.7,
        ai_top_p: 1.0,
        ai_num_predict: 100,
    };
    assert!(get_summarizer(config2, false).await.is_err());

    let config3 = AsumConfig {
        provider: ProviderConfig::OpenAI {
            model: "".to_string(),
            api_key: "key".to_string(),
            url: None,
        },
        max_diff_length: 1000,
        git_extensions: vec![],
        enable_tree_view: true,
        diff_reduction_mode: DiffReductionMode::File,
        max_hunks_per_file: 3,
        system_prompt: "sys".to_string(),
        user_prompt: "user".to_string(),
        ai_temperature: 0.7,
        ai_top_p: 1.0,
        ai_num_predict: 100,
    };
    assert!(get_summarizer(config3, false).await.is_err());

    let config4 = AsumConfig {
        provider: ProviderConfig::VertexAI {
            model: "gemini".to_string(),
            project_id: "".to_string(),
            location: "us".to_string(),
            url: None,
            access_token: None,
        },
        max_diff_length: 1000,
        git_extensions: vec![],
        enable_tree_view: true,
        diff_reduction_mode: DiffReductionMode::File,
        max_hunks_per_file: 3,
        system_prompt: "sys".to_string(),
        user_prompt: "user".to_string(),
        ai_temperature: 0.7,
        ai_top_p: 1.0,
        ai_num_predict: 100,
    };
    assert!(get_summarizer(config4, false).await.is_err());
}

#[tokio::test]
async fn test_summarize_with_mock() {
    let mut mock = MockSummarizer::new();
    mock.expect_summarize()
        .with(mockall::predicate::eq("fake diff"))
        .times(1)
        .returning(|_| Ok("feat: mock summary".to_string()));

    let result = mock.summarize("fake diff").await.unwrap();
    assert_eq!(result, "feat: mock summary");
}
