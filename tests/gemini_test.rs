use asum::summarizer::{AIConfig, Summarizer, gemini::GeminiProvider};

#[test]
fn test_gemini_provider_new() {
    let ai_config = AIConfig {
        model: "gemini-pro".to_string(),
        temperature: 0.7,
        top_p: 1.0,
        num_predict: 100,
        api_url: None,
        api_key: Some("key".to_string()),
        system_prompt: "sys".to_string(),
        user_prompt: "user".to_string(),
        project_id: None,
        location: None,
        verbose: false,
    };
    let provider = GeminiProvider::new(ai_config);
    assert_eq!(provider.config.model, "gemini-pro");
}

#[tokio::test]
async fn test_gemini_summarize_missing_key() {
    let ai_config = AIConfig {
        model: "gemini-pro".to_string(),
        temperature: 0.7,
        top_p: 1.0,
        num_predict: 100,
        api_url: None,
        api_key: None,
        system_prompt: "sys".to_string(),
        user_prompt: "user".to_string(),
        project_id: None,
        location: None,
        verbose: false,
    };
    let provider = GeminiProvider::new(ai_config);
    let result = provider.summarize("diff").await;
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("API key is missing")
    );
}

#[tokio::test]
async fn test_gemini_summarize_success() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let url = format!("http://{}", addr);

    tokio::spawn(async move {
        let (mut socket, _) = listener.accept().await.unwrap();
        let mut buf = [0; 32768];
        let _ = tokio::io::AsyncReadExt::read(&mut socket, &mut buf)
            .await
            .unwrap();

        let response = "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\n\r\n{\"candidates\": [{\"content\": {\"parts\": [{\"text\": \"fix: gemini success\"}]}}]}";
        tokio::io::AsyncWriteExt::write_all(&mut socket, response.as_bytes())
            .await
            .unwrap();
    });

    let ai_config = AIConfig {
        model: "gemini-pro".to_string(),
        temperature: 0.7,
        top_p: 1.0,
        num_predict: 100,
        api_url: Some(url),
        api_key: Some("test_key".to_string()),
        system_prompt: "sys".to_string(),
        user_prompt: "user".to_string(),
        project_id: None,
        location: None,
        verbose: false,
    };
    let provider = GeminiProvider::new(ai_config);
    let result = provider.summarize("diff").await.unwrap();
    assert_eq!(result, "fix: gemini success");
}

#[tokio::test]
async fn test_gemini_summarize_rate_limit_retry() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let url = format!("http://{}", addr);

    tokio::spawn(async move {
        // First request receives 429
        let (mut socket1, _) = listener.accept().await.unwrap();
        let mut buf = [0; 4096];
        let _ = tokio::io::AsyncReadExt::read(&mut socket1, &mut buf)
            .await
            .unwrap();
        let response1 =
            "HTTP/1.1 429 Too Many Requests\r\nConnection: close\r\nContent-Length: 0\r\n\r\n";
        tokio::io::AsyncWriteExt::write_all(&mut socket1, response1.as_bytes())
            .await
            .unwrap();
        let _ = tokio::io::AsyncWriteExt::shutdown(&mut socket1).await;

        // Second request receives 200
        let (mut socket2, _) = listener.accept().await.unwrap();
        let _ = tokio::io::AsyncReadExt::read(&mut socket2, &mut buf)
            .await
            .unwrap();
        let response2 = "HTTP/1.1 200 OK\r\nConnection: close\r\nContent-Type: application/json\r\n\r\n{\"candidates\": [{\"content\": {\"parts\": [{\"text\": \"fix: success after retry\"}]}}]}";
        tokio::io::AsyncWriteExt::write_all(&mut socket2, response2.as_bytes())
            .await
            .unwrap();
        let _ = tokio::io::AsyncWriteExt::shutdown(&mut socket2).await;
    });

    let ai_config = AIConfig {
        model: "gemini-pro".to_string(),
        temperature: 0.7,
        top_p: 1.0,
        num_predict: 100,
        api_url: Some(url),
        api_key: Some("test_key".to_string()),
        system_prompt: "sys".to_string(),
        user_prompt: "user".to_string(),
        project_id: None,
        location: None,
        verbose: false,
    };
    let provider = GeminiProvider::new(ai_config);
    let result = provider.summarize("diff").await.unwrap();
    assert_eq!(result, "fix: success after retry");
}

#[tokio::test]
async fn test_gemini_summarize_api_error() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let url = format!("http://{}", addr);

    tokio::spawn(async move {
        let (mut socket, _) = listener.accept().await.unwrap();
        let mut buf = [0; 4096];
        let _ = tokio::io::AsyncReadExt::read(&mut socket, &mut buf)
            .await
            .unwrap();
        let response =
            "HTTP/1.1 500 Internal Server Error\r\nConnection: close\r\nContent-Length: 0\r\n\r\n";
        tokio::io::AsyncWriteExt::write_all(&mut socket, response.as_bytes())
            .await
            .unwrap();
        let _ = tokio::io::AsyncWriteExt::shutdown(&mut socket).await;
    });

    let ai_config = AIConfig {
        model: "gemini-pro".to_string(),
        temperature: 0.7,
        top_p: 1.0,
        num_predict: 100,
        api_url: Some(url),
        api_key: Some("test_key".to_string()),
        system_prompt: "sys".to_string(),
        user_prompt: "user".to_string(),
        project_id: None,
        location: None,
        verbose: false,
    };
    let provider = GeminiProvider::new(ai_config);
    let result = provider.summarize("diff").await;
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("returned error: 500")
    );
}
