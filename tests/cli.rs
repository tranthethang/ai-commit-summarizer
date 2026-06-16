use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use std::io::Write;
use tempfile::tempdir;

#[test]
fn test_help_args() {
    let mut cmd = Command::cargo_bin("asum").unwrap();
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("AI Commit Summarizer"));
}

#[test]
fn test_run_app_unknown_command() {
    let mut cmd = Command::cargo_bin("asum").unwrap();
    cmd.arg("unknown")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "error: unrecognized subcommand 'unknown'",
        ));
}

#[test]
fn test_run_app_verify_not_found() {
    let dir = tempdir().unwrap();
    let mut cmd = Command::cargo_bin("asum").unwrap();
    cmd.current_dir(dir.path())
        .env("HOME", dir.path())
        .arg("verify")
        .assert()
        .failure()
        .stderr(predicate::str::contains("asum.toml not found"));
}

#[test]
fn test_run_app_verify_valid() {
    let dir = tempdir().unwrap();
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

    let mut cmd = Command::cargo_bin("asum").unwrap();
    cmd.current_dir(dir.path())
        .arg("verify")
        .assert()
        .success()
        .stdout(predicate::str::contains("syntax is valid"));
}

#[test]
fn test_run_app_full_flow_no_staged() {
    let dir = tempdir().unwrap();
    let repo_path = dir.path();

    // Init git
    std::process::Command::new("git")
        .arg("init")
        .current_dir(repo_path)
        .output()
        .unwrap();

    // Create config
    let config_path = repo_path.join("asum.toml");
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

    let mut cmd = Command::cargo_bin("asum").unwrap();
    cmd.current_dir(repo_path).assert().success();
}

#[test]
fn test_run_app_full_flow_with_staged() {
    let dir = tempdir().unwrap();
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
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    let url = format!("http://{}", addr);

    std::thread::spawn(move || {
        if let Ok((mut socket, _)) = listener.accept() {
            let mut buf = [0; 1024];
            let _ = std::io::Read::read(&mut socket, &mut buf);

            let response = "HTTP/1.1 200 OK\r\nConnection: close\r\nContent-Type: application/json\r\n\r\n{\"message\": {\"content\": \"feat: integration success\"}}";
            let _ = std::io::Write::write_all(&mut socket, response.as_bytes());
            let _ = socket.shutdown(std::net::Shutdown::Both);
        }
    });

    // Create config pointing to mock server
    let config_path = repo_path.join("asum.toml");
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
        url = "{}"
        "#,
        url
    )
    .unwrap();

    let mut cmd = Command::cargo_bin("asum").unwrap();
    cmd.current_dir(repo_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("feat: integration success"));
}

#[test]
fn test_run_app_with_tree_view_and_hunk_reduction() {
    let dir = tempdir().unwrap();
    let repo_path = dir.path();

    // Init git
    std::process::Command::new("git")
        .arg("init")
        .current_dir(repo_path)
        .output()
        .unwrap();

    // Create a file and stage it
    let test_file = repo_path.join("test.rs");
    std::fs::write(&test_file, "fn main() {\n// line 1\n// line 2\n}").unwrap();
    std::process::Command::new("git")
        .args(["add", "test.rs"])
        .current_dir(repo_path)
        .output()
        .unwrap();

    // Mock server
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    let url = format!("http://{}", addr);

    std::thread::spawn(move || {
        if let Ok((mut socket, _)) = listener.accept() {
            let mut buf = [0; 32768];
            let _ = std::io::Read::read(&mut socket, &mut buf);

            let request_str = String::from_utf8_lossy(&buf);
            // Verify that the tree view and input diff header are present in the payload
            assert!(request_str.contains("[STAGED FILES TREE]"));
            assert!(request_str.contains("test.rs (Added)"));
            assert!(request_str.contains("[INPUT DIFF]"));

            let response = "HTTP/1.1 200 OK\r\nConnection: close\r\nContent-Type: application/json\r\n\r\n{\"message\": {\"content\": \"feat: integration success\"}}";
            let _ = std::io::Write::write_all(&mut socket, response.as_bytes());
            let _ = socket.shutdown(std::net::Shutdown::Both);
        }
    });

    // Create config pointing to mock server with hunk mode enabled
    let config_path = repo_path.join("asum.toml");
    let mut file = fs::File::create(config_path).unwrap();
    writeln!(
        file,
        r#"
        [general]
        active_provider = "ollama"
        max_diff_length = 1000
        enable_tree_view = true
        diff_reduction_mode = "hunk"
        max_hunks_per_file = 1
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

    let mut cmd = Command::cargo_bin("asum").unwrap();
    cmd.current_dir(repo_path).assert().success();
}

#[test]
fn test_run_app_full_flow_with_truncation() {
    let dir = tempdir().unwrap();
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
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    let url = format!("http://{}", addr);

    std::thread::spawn(move || {
        if let Ok((mut socket, _)) = listener.accept() {
            let mut buf = [0; 4096];
            let _ = std::io::Read::read(&mut socket, &mut buf);

            let response = "HTTP/1.1 200 OK\r\nConnection: close\r\nContent-Type: application/json\r\n\r\n{\"message\": {\"content\": \"feat: truncation success\"}}";
            let _ = std::io::Write::write_all(&mut socket, response.as_bytes());
            let _ = socket.shutdown(std::net::Shutdown::Both);
        }
    });

    // Create config with SMALL max_diff_length
    let config_path = repo_path.join("asum.toml");
    let mut file = fs::File::create(config_path).unwrap();
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

    let mut cmd = Command::cargo_bin("asum").unwrap();
    cmd.current_dir(repo_path).assert().success();
}

#[test]
fn test_run_app_verify_invalid_toml() {
    let dir = tempdir().unwrap();
    let config_path = dir.path().join("asum.toml");
    let mut file = fs::File::create(&config_path).unwrap();
    writeln!(file, "invalid = [").unwrap(); // Unclosed bracket is invalid TOML

    let mut cmd = Command::cargo_bin("asum").unwrap();
    cmd.current_dir(dir.path())
        .arg("verify")
        .assert()
        .failure()
        .stderr(predicate::str::contains("syntax error"));
}

#[test]
fn test_run_app_full_flow_fallback() {
    let dir = tempdir().unwrap();
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
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    let url = format!("http://{}", addr);

    std::thread::spawn(move || {
        if let Ok((mut socket, _)) = listener.accept() {
            let mut buf = [0; 1024];
            let _ = std::io::Read::read(&mut socket, &mut buf);

            let response = "HTTP/1.1 200 OK\r\nConnection: close\r\nContent-Type: application/json\r\n\r\n{\"message\": {\"content\": \"chore: fallback success\"}}";
            let _ = std::io::Write::write_all(&mut socket, response.as_bytes());
            let _ = socket.shutdown(std::net::Shutdown::Both);
        }
    });

    // Create config
    let config_path = repo_path.join("asum.toml");
    let mut file = fs::File::create(config_path).unwrap();
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

    let mut cmd = Command::cargo_bin("asum").unwrap();
    cmd.current_dir(repo_path).assert().success();
}

#[test]
fn test_run_app_summarize_fail() {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    let url = format!("http://{}", addr);

    std::thread::spawn(move || {
        if let Ok((mut stream, _)) = listener.accept() {
            let response = "HTTP/1.1 500 Internal Server Error\r\nContent-Length: 0\r\n\r\n";
            let _ = std::io::Write::write_all(&mut stream, response.as_bytes());
        }
    });

    let repo_path = tempdir().unwrap();
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

    let mut cmd = Command::cargo_bin("asum").unwrap();
    cmd.current_dir(repo_path.path()).assert().failure();
}

#[test]
fn test_run_app_verify_global() {
    let dir = tempdir().unwrap();

    // Create global config in ~/.asum/asum.toml
    let global_dir = dir.path().join(".asum");
    std::fs::create_dir_all(&global_dir).unwrap();
    let config_path = global_dir.join("asum.toml");
    std::fs::write(
        &config_path,
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
        "#,
    )
    .unwrap();

    // Run verify command while current dir does not have local asum.toml
    let empty_dir = tempdir().unwrap();

    let mut cmd = Command::cargo_bin("asum").unwrap();
    cmd.current_dir(empty_dir.path())
        .env("HOME", dir.path())
        .arg("verify")
        .assert()
        .success();
}
