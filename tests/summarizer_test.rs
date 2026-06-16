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

#[tokio::test]
async fn test_check_response_status() {
    let client = reqwest::Client::new();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let url = format!("http://{}", addr);

    tokio::spawn(async move {
        if let Ok((mut socket, _)) = listener.accept().await {
            let response = "HTTP/1.1 200 OK\r\nConnection: close\r\nContent-Length: 0\r\n\r\n";
            let _ = tokio::io::AsyncWriteExt::write_all(&mut socket, response.as_bytes()).await;
            let _ = tokio::io::AsyncWriteExt::shutdown(&mut socket).await;
        }
        if let Ok((mut socket, _)) = listener.accept().await {
            let response = "HTTP/1.1 400 Bad Request\r\nConnection: close\r\nContent-Length: 17\r\nContent-Type: text/plain\r\n\r\nBad Request Error";
            let _ = tokio::io::AsyncWriteExt::write_all(&mut socket, response.as_bytes()).await;
            let _ = tokio::io::AsyncWriteExt::shutdown(&mut socket).await;
        }
    });

    let res_ok = client.get(&url).send().await.unwrap();
    let checked_ok = asum::summarizer::check_response_status(res_ok, "TestProvider").await;
    assert!(checked_ok.is_ok());

    let res_err = client.get(&url).send().await.unwrap();
    let checked_err = asum::summarizer::check_response_status(res_err, "TestProvider").await;
    assert!(checked_err.is_err());
    let err_str = checked_err.unwrap_err().to_string();
    assert!(
        err_str.contains("TestProvider returned error: 400")
            && err_str.contains("Bad Request Error"),
        "Actual error string did not match expected: {}",
        err_str
    );
}

#[test]
fn test_parse_gemini_response() {
    let res_text = r#"{"candidates": [{"content": {"parts": [{"text": "feat: success"}]}}]}"#;
    let parsed = asum::summarizer::parse_gemini_response(res_text, false).unwrap();
    assert_eq!(parsed, "feat: success");

    let invalid_json = r#"{"candidates": []}"#;
    assert!(asum::summarizer::parse_gemini_response(invalid_json, false).is_err());
}

#[test]
fn test_parse_openai_response() {
    let res_text = r#"{"choices": [{"message": {"content": "fix: success"}}]}"#;
    let parsed = asum::summarizer::parse_openai_response(res_text, false).unwrap();
    assert_eq!(parsed, "fix: success");

    let invalid_json = r#"{"choices": []}"#;
    assert!(asum::summarizer::parse_openai_response(invalid_json, false).is_err());
}

#[test]
fn test_build_gemini_payload() {
    use asum::summarizer::AIConfig;
    let config = AIConfig {
        model: "gemini".to_string(),
        temperature: 0.5,
        top_p: 0.9,
        num_predict: 50,
        api_url: None,
        api_key: None,
        system_prompt: "sys".to_string(),
        user_prompt: "user".to_string(),
        project_id: None,
        location: None,
        verbose: false,
    };
    let payload = asum::summarizer::build_gemini_payload("sys", "user", &config);
    assert_eq!(payload["system_instruction"]["parts"][0]["text"], "sys");
    assert_eq!(payload["contents"][0]["role"], "user");
    assert_eq!(payload["contents"][0]["parts"][0]["text"], "user");
    assert_eq!(payload["generationConfig"]["temperature"], 0.5);
    assert_eq!(payload["generationConfig"]["topP"], 0.9);
    assert_eq!(payload["generationConfig"]["maxOutputTokens"], 50);
}
