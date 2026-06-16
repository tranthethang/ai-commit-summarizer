use asum::summarizer::{AIConfig, Summarizer, vertexai::VertexAIProvider};

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
        verbose: true,
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

#[test]
fn test_vertexai_get_access_token_gcloud_fallback_fails() {
    let ai_config = AIConfig {
        model: "gemini-1.5-pro".to_string(),
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
    let provider = VertexAIProvider::new(ai_config);

    // Save the old PATH
    let old_path = std::env::var("PATH").unwrap_or_default();

    // Clear PATH so gcloud is not found
    unsafe {
        std::env::set_var("PATH", "");
    }

    let result = provider.get_access_token();

    // Restore PATH
    unsafe {
        std::env::set_var("PATH", old_path);
    }

    assert!(result.is_err());
}

#[test]
fn test_vertexai_get_access_token_gcloud_returns_exit_status_1() {
    let ai_config = AIConfig {
        model: "gemini-1.5-pro".to_string(),
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
    let provider = VertexAIProvider::new(ai_config);

    // Create a temporary directory containing an executable mock gcloud script that returns exit code 1
    let temp_dir = tempfile::tempdir().unwrap();
    let gcloud_path = temp_dir.path().join("gcloud");

    // Write shell script that exits with 1
    std::fs::write(&gcloud_path, "#!/bin/sh\nexit 1\n").unwrap();

    // Make executable
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&gcloud_path, std::fs::Permissions::from_mode(0o755)).unwrap();
    }

    // Save old PATH
    let old_path = std::env::var("PATH").unwrap_or_default();

    // Prepend temp_dir to PATH
    let new_path = format!("{}:{}", temp_dir.path().display(), old_path);
    unsafe {
        std::env::set_var("PATH", new_path);
    }

    let result = provider.get_access_token();

    // Restore PATH
    unsafe {
        std::env::set_var("PATH", old_path);
    }

    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("gcloud failed to get access token")
    );
}
