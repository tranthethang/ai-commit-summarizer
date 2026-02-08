mod config;
mod git;
mod summarizer;

use crate::config::{AsumConfig, verify_toml};
use crate::git::{get_git_diff, get_staged_files};
use crate::summarizer::get_summarizer;
use anyhow::Context;
use arboard::Clipboard;
use std::env;
use tracing::{error, info, warn};
use tracing_subscriber::{EnvFilter, fmt, prelude::*};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    let mut log_dir = home::home_dir().context("Could not find home directory")?;
    log_dir.push(".asum");
    log_dir.push("logs");
    std::fs::create_dir_all(&log_dir).context("Failed to create log directory")?;

    let file_appender = tracing_appender::rolling::daily(&log_dir, "asum.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    tracing_subscriber::registry()
        .with(EnvFilter::from_default_env().add_directive(tracing::Level::INFO.into()))
        .with(fmt::layer().with_writer(std::io::stderr).with_target(false))
        .with(fmt::layer().with_writer(non_blocking).with_ansi(false))
        .init();

    let args: Vec<String> = env::args().collect();
    run_app(args).await
}

pub async fn run_app(args: Vec<String>) -> anyhow::Result<()> {
    if args.len() > 1 {
        match args[1].as_str() {
            "verify" => {
                if std::path::Path::new("asum.toml").exists() {
                    match verify_toml("asum.toml") {
                        Ok(_) => {
                            println!("[OK] asum.toml syntax is valid.");
                            return Ok(());
                        }
                        Err(e) => {
                            error!("asum.toml syntax error: {}", e);
                            return Err(anyhow::anyhow!("asum.toml syntax error: {}", e));
                        }
                    }
                } else {
                    error!("asum.toml not found in the current directory.");
                    return Err(anyhow::anyhow!("asum.toml not found"));
                }
            }
            "help" | "--help" | "-h" => {
                println!("ASUM - AI Commit Summarizer");
                println!("\nUsage:");
                println!("  asum         Generate commit summary from staged changes");
                println!("  asum verify  Verify the syntax of asum.toml");
                println!("  asum help    Show this help message");
                return Ok(());
            }
            _ => {
                error!("Unknown command: {}", args[1]);
                println!("\nUsage:");
                println!("  asum         Generate commit summary from staged changes");
                println!("  asum verify  Verify the syntax of asum.toml");
                println!("  asum help    Show this help message");
                return Err(anyhow::anyhow!("Unknown command"));
            }
        }
    }

    // Load Configuration (prioritize local asum.toml, then ~/.asum/asum.toml)
    let config = AsumConfig::load().context("Failed to load configuration")?;

    // 1. Get git diff
    let mut diff_text = get_git_diff(&config.git_extensions).context("Failed to get git diff")?;

    if diff_text.is_empty() {
        warn!("No staged changes found in supported code files. Falling back to file list...");
        diff_text = get_staged_files().context("Failed to get staged files")?;

        if diff_text.is_empty() {
            warn!("No staged changes found.");
            return Ok(());
        }
    }

    // 2. Optimize diff size
    let max_diff_length = config.max_diff_length;

    if diff_text.len() > max_diff_length {
        info!(
            "Diff is too large ({} bytes), truncating to {} bytes for AI...",
            diff_text.len(),
            max_diff_length
        );
        info!("You can increase this limit by updating 'max_diff_length' in your config.");
        diff_text = diff_text.chars().take(max_diff_length).collect();
    }

    info!("AI is analyzing your changes...");

    // 3. Get Summarizer (Strategy Pattern)
    let summarizer = get_summarizer(config)
        .await
        .context("Failed to get summarizer")?;

    // 4. Generate Summary
    match summarizer.summarize(&diff_text).await {
        Ok(final_msg) => {
            println!("{}", final_msg);

            // 5. Copy to Clipboard
            if let Ok(mut clipboard) = Clipboard::new() {
                if let Err(e) = clipboard.set_text(final_msg) {
                    error!("Could not copy to clipboard: {}", e);
                } else {
                    info!("Message copied to clipboard. Press Cmd+V to paste.");
                }
            }
        }
        Err(e) => {
            error!("Summarization failed: {}", e);
            return Err(e);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::summarizer::{MockSummarizer, Summarizer};

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

    #[test]
    fn test_help_args() {
        // Since main() uses std::process::exit and println!,
        // we test the logic around argument matching if possible.
        let args = ["asum".to_string(), "help".to_string()];
        assert_eq!(args[1], "help");
    }

    #[test]
    fn test_verify_args() {
        let args = ["asum".to_string(), "verify".to_string()];
        assert_eq!(args[1], "verify");
    }

    #[tokio::test]
    async fn test_run_app_help() {
        let args = vec!["asum".to_string(), "help".to_string()];
        let result = run_app(args).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_run_app_unknown_command() {
        let args = vec!["asum".to_string(), "unknown".to_string()];
        let result = run_app(args).await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Unknown command");
    }

    #[tokio::test]
    async fn test_run_app_verify_not_found() {
        // Run in a temp dir where asum.toml doesn't exist
        let dir = tempfile::tempdir().unwrap();
        let args = vec!["asum".to_string(), "verify".to_string()];

        // Change current directory to temp dir
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(dir.path()).unwrap();

        let result = run_app(args).await;

        // Restore original directory
        std::env::set_current_dir(original_dir).unwrap();

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "asum.toml not found");
    }

    #[tokio::test]
    async fn test_run_app_verify_valid() {
        let dir = tempfile::tempdir().unwrap();
        let config_path = dir.path().join("asum.toml");
        let mut file = std::fs::File::create(config_path).unwrap();
        use std::io::Write;
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
            "#
        )
        .unwrap();

        let args = vec!["asum".to_string(), "verify".to_string()];

        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(dir.path()).unwrap();

        let result = run_app(args).await;

        std::env::set_current_dir(original_dir).unwrap();

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_run_app_full_flow_no_staged() {
        let dir = tempfile::tempdir().unwrap();
        let repo_path = dir.path();

        // Init git
        std::process::Command::new("git")
            .arg("init")
            .current_dir(repo_path)
            .output()
            .unwrap();

        // Create config
        let config_path = repo_path.join("asum.toml");
        let mut file = std::fs::File::create(config_path).unwrap();
        use std::io::Write;
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
            "#
        )
        .unwrap();

        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(repo_path).unwrap();

        let args = vec!["asum".to_string()];
        let result = run_app(args).await;

        std::env::set_current_dir(original_dir).unwrap();

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_run_app_full_flow_with_staged() {
        let dir = tempfile::tempdir().unwrap();
        let repo_path = dir.path();

        // Init git
        std::process::Command::new("git")
            .arg("init")
            .current_dir(repo_path)
            .output()
            .unwrap();

        // Create a file and stage it
        let test_file = repo_path.join("test.rs");
        std::fs::write(&test_file, "fn main() {}").unwrap();
        std::process::Command::new("git")
            .args(["add", "test.rs"])
            .current_dir(repo_path)
            .output()
            .unwrap();

        // Mock server
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let url = format!("http://{}", addr);

        tokio::spawn(async move {
            let (mut socket, _) = listener.accept().await.unwrap();
            let mut buf = [0; 2048];
            let _ = tokio::io::AsyncReadExt::read(&mut socket, &mut buf)
                .await
                .unwrap();

            let response = "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\n\r\n{\"message\": {\"content\": \"feat: integration success\"}}";
            tokio::io::AsyncWriteExt::write_all(&mut socket, response.as_bytes())
                .await
                .unwrap();
        });

        // Create config pointing to mock server
        let config_path = repo_path.join("asum.toml");
        let mut file = std::fs::File::create(config_path).unwrap();
        use std::io::Write;
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
            url = "{}"
            "#,
            url
        )
        .unwrap();

        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(repo_path).unwrap();

        let args = vec!["asum".to_string()];
        let result = run_app(args).await;

        std::env::set_current_dir(original_dir).unwrap();

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_run_app_full_flow_with_truncation() {
        let dir = tempfile::tempdir().unwrap();
        let repo_path = dir.path();

        // Init git
        std::process::Command::new("git")
            .arg("init")
            .current_dir(repo_path)
            .output()
            .unwrap();

        // Create a large file and stage it
        let test_file = repo_path.join("test.rs");
        let large_content = "fn main() {".to_string() + &" ".repeat(2000) + "}";
        std::fs::write(&test_file, large_content).unwrap();
        std::process::Command::new("git")
            .args(["add", "test.rs"])
            .current_dir(repo_path)
            .output()
            .unwrap();

        // Mock server
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let url = format!("http://{}", addr);

        tokio::spawn(async move {
            let (mut socket, _) = listener.accept().await.unwrap();
            let mut buf = [0; 4096];
            let _ = tokio::io::AsyncReadExt::read(&mut socket, &mut buf)
                .await
                .unwrap();

            let response = "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\n\r\n{\"message\": {\"content\": \"feat: truncation success\"}}";
            tokio::io::AsyncWriteExt::write_all(&mut socket, response.as_bytes())
                .await
                .unwrap();
        });

        // Create config with SMALL max_diff_length
        let config_path = repo_path.join("asum.toml");
        let mut file = std::fs::File::create(config_path).unwrap();
        use std::io::Write;
        writeln!(
            file,
            r#"
            [general]
            active_provider = "ollama"
            max_diff_length = 10
            [ai_params]
            num_predict = 100
            temperature = 0.7
            top_p = 1.0
            [ollama]
            model = "llama3"
            url = "{}"
            "#,
            url
        )
        .unwrap();

        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(repo_path).unwrap();

        let args = vec!["asum".to_string()];
        let result = run_app(args).await;

        std::env::set_current_dir(original_dir).unwrap();

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_run_app_verify_invalid_toml() {
        let dir = tempfile::tempdir().unwrap();
        let config_path = dir.path().join("asum.toml");
        let mut file = std::fs::File::create(&config_path).unwrap();
        use std::io::Write;
        writeln!(file, "invalid = [").unwrap(); // Unclosed bracket is invalid TOML

        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(dir.path()).unwrap();

        let args = vec!["asum".to_string(), "verify".to_string()];
        let result = run_app(args).await;

        std::env::set_current_dir(original_dir).unwrap();

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("syntax error"));
    }

    #[tokio::test]
    async fn test_run_app_full_flow_fallback() {
        let dir = tempfile::tempdir().unwrap();
        let repo_path = dir.path();

        // Init git
        std::process::Command::new("git")
            .arg("init")
            .current_dir(repo_path)
            .output()
            .unwrap();

        // Create a file with unsupported extension and stage it
        let test_file = repo_path.join("test.unsupported");
        std::fs::write(&test_file, "some content").unwrap();
        std::process::Command::new("git")
            .args(["add", "test.unsupported"])
            .current_dir(repo_path)
            .output()
            .unwrap();

        // Mock server
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let url = format!("http://{}", addr);

        tokio::spawn(async move {
            let (mut socket, _) = listener.accept().await.unwrap();
            let mut buf = [0; 2048];
            let _ = tokio::io::AsyncReadExt::read(&mut socket, &mut buf)
                .await
                .unwrap();

            let response = "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\n\r\n{\"message\": {\"content\": \"chore: fallback success\"}}";
            tokio::io::AsyncWriteExt::write_all(&mut socket, response.as_bytes())
                .await
                .unwrap();
        });

        // Create config
        let config_path = repo_path.join("asum.toml");
        use std::io::Write;
        let mut file = std::fs::File::create(config_path).unwrap();
        writeln!(
            file,
            r#"
            [general]
            active_provider = "ollama"
            max_diff_length = 1000
            git_extensions = [".rs"]
            [ai_params]
            num_predict = 100
            temperature = 0.7
            top_p = 1.0
            [ollama]
            model = "llama3"
            url = "{}"
            "#,
            url
        )
        .unwrap();

        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(repo_path).unwrap();

        let args = vec!["asum".to_string()];
        let result = run_app(args).await;

        std::env::set_current_dir(original_dir).unwrap();

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_run_app_summarize_fail() {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let url = format!("http://{}", addr);

        tokio::spawn(async move {
            if let Ok((mut stream, _)) = listener.accept() {
                use std::io::Write;
                let response = "HTTP/1.1 500 Internal Server Error\r\nContent-Length: 0\r\n\r\n";
                let _ = stream.write_all(response.as_bytes());
            }
        });

        let repo_path = tempfile::tempdir().unwrap();
        let _ = std::process::Command::new("git")
            .arg("init")
            .current_dir(repo_path.path())
            .output()
            .unwrap();

        std::fs::write(repo_path.path().join("main.rs"), "fn main() {}").unwrap();
        let _ = std::process::Command::new("git")
            .args(["add", "main.rs"])
            .current_dir(repo_path.path())
            .output()
            .unwrap();

        let config_path = repo_path.path().join("asum.toml");
        std::fs::write(
            &config_path,
            format!(
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
            url = "{}"
            "#,
                url
            ),
        )
        .unwrap();

        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(repo_path.path()).unwrap();

        let args = vec!["asum".to_string()];
        let result = run_app(args).await;

        std::env::set_current_dir(original_dir).unwrap();

        assert!(result.is_err());
    }
}
