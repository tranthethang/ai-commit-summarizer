use asum::summarizer::{AIConfig, Summarizer, ollama::OllamaProvider};

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
        project_id: None,
        location: None,
        verbose: false,
    };
    let provider = OllamaProvider::new(ai_config);
    assert_eq!(provider.config.model, "llama3");
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
        project_id: None,
        location: None,
        verbose: false,
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
        let mut buf = [0; 32768];
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
        project_id: None,
        location: None,
        verbose: false,
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
        let mut buf = [0; 32768];
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
        project_id: None,
        location: None,
        verbose: false,
    };
    let provider = OllamaProvider::new(ai_config);
    let result = provider.summarize("diff").await.unwrap();
    assert_eq!(result, "feat: success from generate");
}
