use asum::config::{AsumConfig, DiffReductionMode, ProviderConfig, verify_toml};
use std::fs;
use std::io::Write;
use tempfile::NamedTempFile;

#[test]
fn test_load_from_toml_full() {
    let mut file = NamedTempFile::new().unwrap();
    writeln!(
        file,
        r#"
        [general]
        active_provider = "gemini"
        max_diff_length = 1000
        git_extensions = [".rs", ".py"]

        [ai_params]
        num_predict = 100
        temperature = 0.5
        top_p = 0.9

        [gemini]
        api_key = "test_key"
        model = "gemini-pro"
        "#
    )
    .unwrap();

    let config = AsumConfig::load_from_toml(file.path()).unwrap();
    assert_eq!(
        config.provider,
        ProviderConfig::Gemini {
            api_key: "test_key".to_string(),
            model: "gemini-pro".to_string(),
            url: None,
        }
    );
    assert_eq!(config.max_diff_length, 1000);
    assert_eq!(config.git_extensions, vec![".rs", ".py"]);
}

#[test]
fn test_load_from_toml_missing_provider_sections() {
    let mut file = NamedTempFile::new().unwrap();
    writeln!(
        file,
        r#"
        [general]
        active_provider = "openai"
        max_diff_length = 1000

        [ai_params]
        num_predict = 100
        temperature = 0.5
        top_p = 0.9
        "#
    )
    .unwrap();

    let result = AsumConfig::load_from_toml(file.path());
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Missing [openai]"));

    let mut file2 = NamedTempFile::new().unwrap();
    writeln!(
        file2,
        r#"
        [general]
        active_provider = "vertexai"
        max_diff_length = 1000

        [ai_params]
        num_predict = 100
        temperature = 0.5
        top_p = 0.9
        "#
    )
    .unwrap();
    let result2 = AsumConfig::load_from_toml(file2.path());
    assert!(result2.is_err());
    assert!(
        result2
            .unwrap_err()
            .to_string()
            .contains("Missing [vertexai]")
    );

    let mut file3 = NamedTempFile::new().unwrap();
    writeln!(
        file3,
        r#"
        [general]
        active_provider = "gemini"
        max_diff_length = 1000

        [ai_params]
        num_predict = 100
        temperature = 0.5
        top_p = 0.9
        "#
    )
    .unwrap();
    let result3 = AsumConfig::load_from_toml(file3.path());
    assert!(result3.is_err());
    assert!(
        result3
            .unwrap_err()
            .to_string()
            .contains("Missing [gemini]")
    );

    let mut file4 = NamedTempFile::new().unwrap();
    writeln!(
        file4,
        r#"
        [general]
        active_provider = "ollama"
        max_diff_length = 1000

        [ai_params]
        num_predict = 100
        temperature = 0.5
        top_p = 0.9
        "#
    )
    .unwrap();
    let mut file5 = NamedTempFile::new().unwrap();
    writeln!(
        file5,
        r#"
        [general]
        active_provider = "groq"
        max_diff_length = 1000

        [ai_params]
        num_predict = 100
        temperature = 0.5
        top_p = 0.9
        "#
    )
    .unwrap();
    let result5 = AsumConfig::load_from_toml(file5.path());
    assert!(result5.is_err());
    assert!(result5.unwrap_err().to_string().contains("Missing [groq]"));

    let mut file6 = NamedTempFile::new().unwrap();
    writeln!(
        file6,
        r#"
        [general]
        active_provider = "mistral"
        max_diff_length = 1000

        [ai_params]
        num_predict = 100
        temperature = 0.5
        top_p = 0.9
        "#
    )
    .unwrap();
    let result6 = AsumConfig::load_from_toml(file6.path());
    assert!(result6.is_err());
    assert!(
        result6
            .unwrap_err()
            .to_string()
            .contains("Missing [mistral]")
    );

    let mut file7 = NamedTempFile::new().unwrap();
    writeln!(
        file7,
        r#"
        [general]
        active_provider = "github"
        max_diff_length = 1000

        [ai_params]
        num_predict = 100
        temperature = 0.5
        top_p = 0.9
        "#
    )
    .unwrap();
    let result7 = AsumConfig::load_from_toml(file7.path());
    assert!(result7.is_err());
    assert!(
        result7
            .unwrap_err()
            .to_string()
            .contains("Missing [github]")
    );
}

#[test]
fn test_load_from_toml_defaults() {
    let mut file = NamedTempFile::new().unwrap();
    writeln!(
        file,
        r#"
        [general]
        active_provider = "ollama"
        max_diff_length = 2000

        [ai_params]
        num_predict = 50
        temperature = 0.7
        top_p = 1.0

        [ollama]
        model = "llama3"
        url = "http://localhost:11434"
        "#
    )
    .unwrap();

    let config = AsumConfig::load_from_toml(file.path()).unwrap();
    assert_eq!(
        config.provider,
        ProviderConfig::Ollama {
            model: "llama3".to_string(),
            url: "http://localhost:11434".to_string(),
        }
    );
    // Check if default extensions are loaded
    assert!(!config.git_extensions.is_empty());
    assert!(config.git_extensions.contains(&"*.rs".to_string()));
    // Check if default system prompt is loaded
    assert!(config.system_prompt.contains("expert Git Commit Generator"));
}

#[test]
fn test_verify_toml_table_driven() {
    struct TestCase {
        name: &'static str,
        content: &'static str,
        is_ok: bool,
    }

    let cases = vec![
        TestCase {
            name: "valid full config",
            content: r#"
                [general]
                active_provider = "ollama"
                max_diff_length = 2000
                [ai_params]
                num_predict = 50
                temperature = 0.7
                top_p = 1.0
                [ollama]
                model = "llama3"
                url = "http://localhost:11434"
            "#,
            is_ok: true,
        },
        TestCase {
            name: "missing general section",
            content: r#"
                [ai_params]
                num_predict = 50
                temperature = 0.7
                top_p = 1.0
            "#,
            is_ok: false,
        },
        TestCase {
            name: "invalid toml syntax",
            content: "invalid = [",
            is_ok: false,
        },
        TestCase {
            name: "missing required active provider section",
            content: r#"
                [general]
                active_provider = "gemini"
                max_diff_length = 2000
                [ai_params]
                num_predict = 50
                temperature = 0.7
                top_p = 1.0
            "#,
            is_ok: false,
        },
        TestCase {
            name: "empty model in active provider section",
            content: r#"
                [general]
                active_provider = "gemini"
                max_diff_length = 2000
                [ai_params]
                num_predict = 50
                temperature = 0.7
                top_p = 1.0
                [gemini]
                api_key = "test"
                model = ""
            "#,
            is_ok: false,
        },
        TestCase {
            name: "empty api_key in gemini provider section",
            content: r#"
                [general]
                active_provider = "gemini"
                max_diff_length = 2000
                [ai_params]
                num_predict = 50
                temperature = 0.7
                top_p = 1.0
                [gemini]
                api_key = ""
                model = "test"
            "#,
            is_ok: false,
        },
        TestCase {
            name: "valid full config openai",
            content: r#"
                [general]
                active_provider = "openai"
                max_diff_length = 2000
                [ai_params]
                num_predict = 50
                temperature = 0.7
                top_p = 1.0
                [openai]
                model = "gpt-4"
                api_key = "sk-123"
            "#,
            is_ok: true,
        },
        TestCase {
            name: "invalid config openai missing model",
            content: r#"
                [general]
                active_provider = "openai"
                max_diff_length = 2000
                [ai_params]
                num_predict = 50
                temperature = 0.7
                top_p = 1.0
                [openai]
                api_key = "sk-123"
                model = ""
            "#,
            is_ok: false,
        },
        TestCase {
            name: "invalid config openai missing api_key",
            content: r#"
                [general]
                active_provider = "openai"
                max_diff_length = 2000
                [ai_params]
                num_predict = 50
                temperature = 0.7
                top_p = 1.0
                [openai]
                api_key = ""
                model = "gpt-4"
            "#,
            is_ok: false,
        },
        TestCase {
            name: "valid full config vertexai",
            content: r#"
                [general]
                active_provider = "vertexai"
                max_diff_length = 2000
                [ai_params]
                num_predict = 50
                temperature = 0.7
                top_p = 1.0
                [vertexai]
                model = "gemini-pro"
                project_id = "test-project"
                location = "us-central1"
            "#,
            is_ok: true,
        },
        TestCase {
            name: "invalid config vertexai missing project",
            content: r#"
                [general]
                active_provider = "vertexai"
                max_diff_length = 2000
                [ai_params]
                num_predict = 50
                temperature = 0.7
                top_p = 1.0
                [vertexai]
                model = "gemini-pro"
                project_id = ""
                location = "us-central1"
            "#,
            is_ok: false,
        },
        TestCase {
            name: "invalid config vertexai missing location",
            content: r#"
                [general]
                active_provider = "vertexai"
                max_diff_length = 2000
                [ai_params]
                num_predict = 50
                temperature = 0.7
                top_p = 1.0
                [vertexai]
                model = "gemini-pro"
                project_id = "test-project"
                location = ""
            "#,
            is_ok: false,
        },
        TestCase {
            name: "invalid config vertexai missing model",
            content: r#"
                [general]
                active_provider = "vertexai"
                max_diff_length = 2000
                [ai_params]
                num_predict = 50
                temperature = 0.7
                top_p = 1.0
                [vertexai]
                model = ""
                project_id = "test-project"
                location = "us-central1"
            "#,
            is_ok: false,
        },
        TestCase {
            name: "empty model in ollama provider section",
            content: r#"
                [general]
                active_provider = "ollama"
                max_diff_length = 2000
                [ai_params]
                num_predict = 50
                temperature = 0.7
                top_p = 1.0
                [ollama]
                model = ""
                url = "http://localhost"
            "#,
            is_ok: false,
        },
        TestCase {
            name: "empty url in ollama provider section",
            content: r#"
                [general]
                active_provider = "ollama"
                max_diff_length = 2000
                [ai_params]
                num_predict = 50
                temperature = 0.7
                top_p = 1.0
                [ollama]
                model = "test"
                url = ""
            "#,
            is_ok: false,
        },
        TestCase {
            name: "valid full config groq",
            content: r#"
                [general]
                active_provider = "groq"
                max_diff_length = 2000
                [ai_params]
                num_predict = 50
                temperature = 0.7
                top_p = 1.0
                [groq]
                model = "llama-3.3-70b-versatile"
                api_key = "gsk_123"
            "#,
            is_ok: true,
        },
        TestCase {
            name: "invalid config groq missing model",
            content: r#"
                [general]
                active_provider = "groq"
                max_diff_length = 2000
                [ai_params]
                num_predict = 50
                temperature = 0.7
                top_p = 1.0
                [groq]
                api_key = "gsk_123"
                model = ""
            "#,
            is_ok: false,
        },
        TestCase {
            name: "invalid config groq missing api_key",
            content: r#"
                [general]
                active_provider = "groq"
                max_diff_length = 2000
                [ai_params]
                num_predict = 50
                temperature = 0.7
                top_p = 1.0
                [groq]
                api_key = ""
                model = "llama-3.3-70b-versatile"
            "#,
            is_ok: false,
        },
        TestCase {
            name: "valid config mistral",
            content: r#"
                [general]
                active_provider = "mistral"
                max_diff_length = 2000
                [ai_params]
                num_predict = 50
                temperature = 0.7
                top_p = 1.0
                [mistral]
                model = "mistral-small-latest"
                api_key = "mistral_key_123"
            "#,
            is_ok: true,
        },
        TestCase {
            name: "invalid config mistral missing model",
            content: r#"
                [general]
                active_provider = "mistral"
                max_diff_length = 2000
                [ai_params]
                num_predict = 50
                temperature = 0.7
                top_p = 1.0
                [mistral]
                api_key = "mistral_key_123"
                model = ""
            "#,
            is_ok: false,
        },
        TestCase {
            name: "invalid config mistral missing api_key",
            content: r#"
                [general]
                active_provider = "mistral"
                max_diff_length = 2000
                [ai_params]
                num_predict = 50
                temperature = 0.7
                top_p = 1.0
                [mistral]
                api_key = ""
                model = "mistral-small-latest"
            "#,
            is_ok: false,
        },
        TestCase {
            name: "valid full config github",
            content: r#"
                [general]
                active_provider = "github"
                max_diff_length = 2000
                [ai_params]
                num_predict = 50
                temperature = 0.7
                top_p = 1.0
                [github]
                model = "gpt-4o-mini"
                api_key = "ghp_123"
            "#,
            is_ok: true,
        },
        TestCase {
            name: "invalid config github missing model",
            content: r#"
                [general]
                active_provider = "github"
                max_diff_length = 2000
                [ai_params]
                num_predict = 50
                temperature = 0.7
                top_p = 1.0
                [github]
                api_key = "ghp_123"
                model = ""
            "#,
            is_ok: false,
        },
        TestCase {
            name: "invalid config github missing api_key",
            content: r#"
                [general]
                active_provider = "github"
                max_diff_length = 2000
                [ai_params]
                num_predict = 50
                temperature = 0.7
                top_p = 1.0
                [github]
                api_key = ""
                model = "gpt-4o-mini"
            "#,
            is_ok: false,
        },
    ];

    for case in cases {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "{}", case.content).unwrap();
        let result = verify_toml(file.path());
        assert_eq!(
            result.is_ok(),
            case.is_ok,
            "Failed test case: {}",
            case.name
        );
    }
}

#[test]
#[should_panic(expected = "No such file or directory")]
fn test_load_from_toml_non_existent() {
    AsumConfig::load_from_toml("non_existent_file.toml").unwrap();
}

#[test]
fn test_load_from_toml_minimal() {
    let mut file = NamedTempFile::new().unwrap();
    writeln!(
        file,
        r#"
        [general]
        active_provider = "ollama"
        max_diff_length = 500

        [ai_params]
        num_predict = 10
        temperature = 0.1
        top_p = 0.1

        [ollama]
        model = "llama3"
        url = "http://localhost:11434"
        "#
    )
    .unwrap();

    let config = AsumConfig::load_from_toml(file.path()).unwrap();
    assert_eq!(
        config.provider,
        ProviderConfig::Ollama {
            model: "llama3".to_string(),
            url: "http://localhost:11434".to_string(),
        }
    );
    assert_eq!(config.max_diff_length, 500);
    assert_eq!(config.ai_num_predict, 10);
}

#[test]
fn test_load_from_toml_with_custom_prompts() {
    let mut file = NamedTempFile::new().unwrap();
    let toml_content = r#"
        [general]
        active_provider = "ollama"
        max_diff_length = 1000

        [ai_params]
        num_predict = 100
        temperature = 0.7
        top_p = 1.0

        [prompts]
        system_prompt = "Custom system prompt"
        user_prompt = "Custom user prompt: {{diff}}"

        [ollama]
        model = "llama3"
        url = "http://localhost:11434"
        "#;
    writeln!(file, "{}", toml_content).unwrap();

    let config = AsumConfig::load_from_toml(file.path()).unwrap();
    if config.user_prompt != "Custom user prompt: {{diff}}" {
        panic!(
            "CONTENT: [{}], PARSED: [{}]",
            toml_content, config.user_prompt
        );
    }
    assert_eq!(config.system_prompt, "Custom system prompt");
}

#[test]
fn test_load_from_toml_with_tree_and_hunks() {
    let mut file = NamedTempFile::new().unwrap();
    let toml_content = r#"
        [general]
        active_provider = "ollama"
        max_diff_length = 1000
        enable_tree_view = false
        diff_reduction_mode = "hunk"
        max_hunks_per_file = 5

        [ai_params]
        num_predict = 100
        temperature = 0.7
        top_p = 1.0

        [ollama]
        model = "llama3"
        url = "http://localhost:11434"
        "#;
    writeln!(file, "{}", toml_content).unwrap();

    let config = AsumConfig::load_from_toml(file.path()).unwrap();
    assert!(!config.enable_tree_view);
    assert_eq!(config.diff_reduction_mode, DiffReductionMode::Hunk);
    assert_eq!(config.max_hunks_per_file, 5);
}

#[test]
fn test_asum_config_load_local() {
    let dir = tempfile::tempdir().unwrap();
    let config_path = dir.path().join("asum.toml");
    let mut file = fs::File::create(config_path).unwrap();
    writeln!(
        file,
        r#"
        [general]
        active_provider = "ollama"
        max_diff_length = 1000
        [ai_params]
        num_predict = 100
        temperature = 0.7
        top_p = 1.0
        [ollama]
        model = "llama3"
        url = "http://localhost:11434"
        "#
    )
    .unwrap();

    let result = AsumConfig::load_with_search(Some(dir.path()), None);

    assert!(result.is_ok());
    assert_eq!(
        result.unwrap().provider,
        ProviderConfig::Ollama {
            model: "llama3".to_string(),
            url: "http://localhost:11434".to_string(),
        }
    );
}

#[test]
fn test_asum_config_load_global() {
    let temp_home = std::env::temp_dir().join(format!("fake_home_global_{}", std::process::id()));
    let global_dir = temp_home.join(".asum");
    fs::create_dir_all(&global_dir).unwrap();
    let config_path = global_dir.join("asum.toml");

    let mut file = fs::File::create(&config_path).unwrap();
    writeln!(
        file,
        r#"
        [general]
        active_provider = "ollama"
        max_diff_length = 500
        [ai_params]
        num_predict = 100
        temperature = 0.7
        top_p = 1.0
        [ollama]
        model = "llama3"
        url = "http://localhost:11434"
        "#
    )
    .unwrap();

    let temp_cwd = std::env::temp_dir().join(format!("empty_cwd_{}", std::process::id()));
    fs::create_dir_all(&temp_cwd).unwrap();

    let result = AsumConfig::load_with_search(Some(&temp_cwd), Some(&temp_home));

    // Clean up temp dirs
    let _ = fs::remove_dir_all(&temp_home);
    let _ = fs::remove_dir_all(&temp_cwd);

    let config = result.expect("Should load global config");
    assert_eq!(
        config.provider,
        ProviderConfig::Ollama {
            model: "llama3".to_string(),
            url: "http://localhost:11434".to_string(),
        }
    );
    assert_eq!(config.max_diff_length, 500);
}

#[test]
fn test_asum_config_load_no_config() {
    let temp_dir = std::env::temp_dir().join(format!("no_config_test_{}", std::process::id()));
    fs::create_dir_all(&temp_dir).unwrap();

    let result = AsumConfig::load_with_search(Some(&temp_dir), Some(&temp_dir));

    let _ = fs::remove_dir_all(&temp_dir);

    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("not found"));
}

#[test]
fn test_load_from_toml_with_fallbacks() {
    let mut file = NamedTempFile::new().unwrap();
    writeln!(
        file,
        r#"
        [general]
        active_provider = "groq"
        max_diff_length = 1000
        fallbacks = ["github", "mistral"]

        [ai_params]
        num_predict = 100
        temperature = 0.5
        top_p = 0.9

        [groq]
        api_key = "gsk_test"
        model = "llama-3.3-70b-versatile"

        [github]
        api_key = "ghp_test"
        model = "gpt-4o-mini"

        [mistral]
        api_key = "mistral_key"
        model = "mistral-small-latest"
        "#
    )
    .unwrap();

    let config = AsumConfig::load_from_toml(file.path()).unwrap();
    assert_eq!(
        config.provider,
        ProviderConfig::Groq {
            api_key: "gsk_test".to_string(),
            model: "llama-3.3-70b-versatile".to_string(),
            url: None,
        }
    );
    assert_eq!(config.fallbacks.len(), 2);
    assert_eq!(
        config.fallbacks[0],
        ProviderConfig::Github {
            api_key: "ghp_test".to_string(),
            model: "gpt-4o-mini".to_string(),
            url: None,
        }
    );
    assert_eq!(
        config.fallbacks[1],
        ProviderConfig::Mistral {
            api_key: "mistral_key".to_string(),
            model: "mistral-small-latest".to_string(),
            url: None,
        }
    );
}

#[test]
fn test_load_from_toml_empty_fallbacks() {
    let mut file = NamedTempFile::new().unwrap();
    writeln!(
        file,
        r#"
        [general]
        active_provider = "groq"
        max_diff_length = 1000
        fallbacks = []

        [ai_params]
        num_predict = 100
        temperature = 0.5
        top_p = 0.9

        [groq]
        api_key = "gsk_test"
        model = "llama-3.3-70b-versatile"
        "#
    )
    .unwrap();

    let config = AsumConfig::load_from_toml(file.path()).unwrap();
    assert!(config.fallbacks.is_empty());
}

#[test]
fn test_load_from_toml_no_fallbacks_field() {
    let mut file = NamedTempFile::new().unwrap();
    writeln!(
        file,
        r#"
        [general]
        active_provider = "groq"
        max_diff_length = 1000

        [ai_params]
        num_predict = 100
        temperature = 0.5
        top_p = 0.9

        [groq]
        api_key = "gsk_test"
        model = "llama-3.3-70b-versatile"
        "#
    )
    .unwrap();

    let config = AsumConfig::load_from_toml(file.path()).unwrap();
    assert!(config.fallbacks.is_empty());
}

#[test]
fn test_load_from_toml_fallback_missing_provider_section() {
    let mut file = NamedTempFile::new().unwrap();
    writeln!(
        file,
        r#"
        [general]
        active_provider = "groq"
        max_diff_length = 1000
        fallbacks = ["github"]

        [ai_params]
        num_predict = 100
        temperature = 0.5
        top_p = 0.9

        [groq]
        api_key = "gsk_test"
        model = "llama-3.3-70b-versatile"
        "#
    )
    .unwrap();

    let result = AsumConfig::load_from_toml(file.path());
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Missing [github]"));
}

#[test]
fn test_verify_toml_with_valid_fallbacks() {
    let mut file = NamedTempFile::new().unwrap();
    writeln!(
        file,
        r#"
        [general]
        active_provider = "groq"
        max_diff_length = 1000
        fallbacks = ["github", "mistral"]

        [ai_params]
        num_predict = 100
        temperature = 0.5
        top_p = 0.9

        [groq]
        api_key = "gsk_test"
        model = "llama-3.3-70b-versatile"

        [github]
        api_key = "ghp_test"
        model = "gpt-4o-mini"

        [mistral]
        api_key = "mistral_key"
        model = "mistral-small-latest"
        "#
    )
    .unwrap();

    let result = verify_toml(file.path());
    assert!(result.is_ok());
}

#[test]
fn test_verify_toml_with_invalid_fallback_config() {
    let mut file = NamedTempFile::new().unwrap();
    writeln!(
        file,
        r#"
        [general]
        active_provider = "groq"
        max_diff_length = 1000
        fallbacks = ["github"]

        [ai_params]
        num_predict = 100
        temperature = 0.5
        top_p = 0.9

        [groq]
        api_key = "gsk_test"
        model = "llama-3.3-70b-versatile"

        [github]
        api_key = ""
        model = "gpt-4o-mini"
        "#
    )
    .unwrap();

    let result = verify_toml(file.path());
    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("Fallback provider"));
}
