use asum::summarizer::{AIConfig, Summarizer, openai::OpenAIProvider};

#[test]
fn test_openai_provider_new() {
    let ai_config = AIConfig {
        model: "gpt-4".to_string(),
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
    let provider = OpenAIProvider::new(ai_config);
    assert_eq!(provider.config.model, "gpt-4");
    assert_eq!(
        provider.base_url,
        "https://api.openai.com/v1/chat/completions"
    );
}

#[tokio::test]
async fn test_openai_summarize_missing_key() {
    let ai_config = AIConfig {
        model: "gpt-4".to_string(),
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
    let provider = OpenAIProvider::new(ai_config);
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
async fn test_openai_summarize_success() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let url = format!("http://{}", addr);

    tokio::spawn(async move {
        let (mut socket, _) = listener.accept().await.unwrap();
        let mut buf = [0; 32768];
        let _ = tokio::io::AsyncReadExt::read(&mut socket, &mut buf)
            .await
            .unwrap();

        let response = "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\n\r\n{\"choices\": [{\"message\": {\"content\": \"fix: openai success\"}}]}";
        tokio::io::AsyncWriteExt::write_all(&mut socket, response.as_bytes())
            .await
            .unwrap();
    });

    let ai_config = AIConfig {
        model: "gpt-4".to_string(),
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
    let provider = OpenAIProvider::new(ai_config);
    let result = provider.summarize("diff").await.unwrap();
    assert_eq!(result, "fix: openai success");
}

#[tokio::test]
async fn test_openai_summarize_api_error() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let url = format!("http://{}", addr);

    tokio::spawn(async move {
        let (mut socket, _) = listener.accept().await.unwrap();
        let mut buf = [0; 32768];
        let _ = tokio::io::AsyncReadExt::read(&mut socket, &mut buf)
            .await
            .unwrap();

        let response = "HTTP/1.1 400 Bad Request\r\nContent-Type: text/plain\r\n\r\nBad Request";
        tokio::io::AsyncWriteExt::write_all(&mut socket, response.as_bytes())
            .await
            .unwrap();
    });

    let ai_config = AIConfig {
        model: "gpt-4".to_string(),
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
    let provider = OpenAIProvider::new(ai_config);
    let result = provider.summarize("diff").await;
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("returned error: 400")
    );
}

#[tokio::test]
async fn test_openai_summarize_empty_response() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let url = format!("http://{}", addr);

    tokio::spawn(async move {
        let (mut socket, _) = listener.accept().await.unwrap();
        let mut buf = [0; 32768];
        let _ = tokio::io::AsyncReadExt::read(&mut socket, &mut buf)
            .await
            .unwrap();

        let response = "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\n\r\n{\"choices\": [{\"message\": {\"content\": \"\\n  \\n\"}}]}";
        tokio::io::AsyncWriteExt::write_all(&mut socket, response.as_bytes())
            .await
            .unwrap();
    });

    let ai_config = AIConfig {
        model: "gpt-4".to_string(),
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
    let provider = OpenAIProvider::new(ai_config);
    let result = provider.summarize("diff").await;
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("empty or invalid message")
    );
}

#[tokio::test]
async fn test_openai_summarize_reasoning_model_success() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let url = format!("http://{}", addr);

    tokio::spawn(async move {
        let (mut socket, _) = listener.accept().await.unwrap();
        let mut buf = [0; 32768];
        let n = tokio::io::AsyncReadExt::read(&mut socket, &mut buf)
            .await
            .unwrap();
        let request_str = String::from_utf8_lossy(&buf[..n]);
        let parts: Vec<&str> = request_str.split("\r\n\r\n").collect();
        assert!(parts.len() >= 2);
        let body_json: serde_json::Value = serde_json::from_str(parts[1]).unwrap();

        assert_eq!(body_json["model"], "o1-mini");
        assert_eq!(body_json["max_completion_tokens"], 150);
        assert!(body_json.get("max_tokens").is_none());
        assert!(body_json.get("temperature").is_none());
        assert!(body_json.get("top_p").is_none());

        let response = "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\n\r\n{\"choices\": [{\"message\": {\"content\": \"fix: openai reasoning success\"}}]}";
        tokio::io::AsyncWriteExt::write_all(&mut socket, response.as_bytes())
            .await
            .unwrap();
    });

    let ai_config = AIConfig {
        model: "o1-mini".to_string(),
        temperature: 0.7,
        top_p: 1.0,
        num_predict: 150,
        api_url: Some(url),
        api_key: Some("test_key".to_string()),
        system_prompt: "sys".to_string(),
        user_prompt: "user".to_string(),
        project_id: None,
        location: None,
        verbose: false,
    };
    let provider = OpenAIProvider::new(ai_config);
    let result = provider.summarize("diff").await.unwrap();
    assert_eq!(result, "fix: openai reasoning success");
}

#[tokio::test]
async fn test_openai_summarize_standard_model_payload_verification() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let url = format!("http://{}", addr);

    tokio::spawn(async move {
        let (mut socket, _) = listener.accept().await.unwrap();
        let mut buf = [0; 32768];
        let n = tokio::io::AsyncReadExt::read(&mut socket, &mut buf)
            .await
            .unwrap();
        let request_str = String::from_utf8_lossy(&buf[..n]);
        let parts: Vec<&str> = request_str.split("\r\n\r\n").collect();
        assert!(parts.len() >= 2);
        let body_json: serde_json::Value = serde_json::from_str(parts[1]).unwrap();

        assert_eq!(body_json["model"], "gpt-4");
        assert_eq!(body_json["max_tokens"], 100);
        assert_eq!(body_json["temperature"], 0.7);
        assert_eq!(body_json["top_p"], 1.0);
        assert!(body_json.get("max_completion_tokens").is_none());

        let response = "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\n\r\n{\"choices\": [{\"message\": {\"content\": \"fix: openai standard success\"}}]}";
        tokio::io::AsyncWriteExt::write_all(&mut socket, response.as_bytes())
            .await
            .unwrap();
    });

    let ai_config = AIConfig {
        model: "gpt-4".to_string(),
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
    let provider = OpenAIProvider::new(ai_config);
    let result = provider.summarize("diff").await.unwrap();
    assert_eq!(result, "fix: openai standard success");
}
