use arboard::Clipboard;
use dotenvy::dotenv;
use reqwest::Client;
use serde_json::json;
use std::env;
use std::process::Command;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment variables from .env file
    dotenv().ok();

    let client = Client::builder().timeout(Duration::from_secs(30)).build()?;

    // 1. Get git diff with language filters and exclude binary/lock files
    let output = Command::new("git")
        .args([
            "diff",
            "--cached",
            "--",
            "*.java",
            "*.php",
            "*.js",
            "*.jsx",
            "*.ts",
            "*.tsx",
            "*.scss",
            "*.css",
            "*.rs",
            "*.py",
            "*.go",
            "*.c",
            "*.cpp",
            ":(exclude)*-lock.json",
            ":(exclude)package-lock.json",
            ":(exclude)pnpm-lock.yaml",
            ":(exclude)*.min.js",
        ])
        .output()?;

    let mut diff_text = String::from_utf8_lossy(&output.stdout).to_string();

    if diff_text.is_empty() {
        eprintln!("[WARN] No staged changes found in supported code files.");
        return Ok(());
    }

    // 2. Optimize diff size (Avoid high memory usage)
    let max_diff_length: usize = env::var("MAX_DIFF_LENGTH")
        .unwrap_or_else(|_| "6000".to_string())
        .parse()
        .unwrap_or(6000);

    if diff_text.len() > max_diff_length {
        eprintln!(
            "[INFO] Diff is too large ({} bytes), truncating for AI...",
            diff_text.len()
        );
        diff_text = diff_text.chars().take(max_diff_length).collect();
    }

    // 3. "Few-Shot" prompt optimized for Llama-3.2-1B
    let prompt = format!(
        "SYSTEM: You are a professional Git Commit Generator.
RULES:
1. Output ONLY a bulleted list of changes.
2. Max 10 items.
3. Use Conventional Commits format (feat:, fix:, refactor:, etc.).
4. NO code snippets, NO preamble, NO explanations, NO echo of the input.
5. Use plain text only, NO emojis.

EXAMPLE OUTPUT:
- feat: add ollama api integration for rust
- fix: handle null pointer in java controller
- refactor: optimize scss variables for dark mode

INPUT DIFF:
{}
",
        diff_text
    );

    eprintln!("[INFO] AI is analyzing your changes...");

    // 4. Call Ollama API with configurable parameters
    let ollama_url = env::var("OLLAMA_API_URL")
        .unwrap_or_else(|_| "http://localhost:11434/api/generate".to_string());
    let ollama_model = env::var("OLLAMA_MODEL").unwrap_or_else(|_| "llama3.2:1b".to_string());
    let temperature: f64 = env::var("AI_TEMPERATURE")
        .unwrap_or_else(|_| "0.1".to_string())
        .parse()
        .unwrap_or(0.1);
    let num_predict: i32 = env::var("AI_NUM_PREDICT")
        .unwrap_or_else(|_| "250".to_string())
        .parse()
        .unwrap_or(250);
    let top_p: f64 = env::var("AI_TOP_P")
        .unwrap_or_else(|_| "0.9".to_string())
        .parse()
        .unwrap_or(0.9);

    let response = client
        .post(&ollama_url)
        .json(&json!({
            "model": ollama_model,
            "prompt": prompt,
            "stream": false,
            "options": {
                "temperature": temperature,
                "num_predict": num_predict,
                "top_p": top_p
            }
        }))
        .send()
        .await;

    match response {
        Ok(res) => {
            if res.status().is_success() {
                let res_json: serde_json::Value = res.json().await?;
                let commit_msg = res_json["response"].as_str().unwrap_or("").trim();

                // Clean up extra whitespace and filter out hallucinations/echoes
                let final_msg = commit_msg
                    .lines()
                    .map(|l| l.trim())
                    .filter(|l| {
                        !l.is_empty() 
                        && !l.to_lowercase().contains("diff to analyze")
                        && !l.to_lowercase().contains("input diff")
                        && l.starts_with("- ")
                    })
                    .collect::<Vec<_>>()
                    .join("\n");

                if final_msg.is_empty() {
                    eprintln!("[ERROR] AI generated an empty or invalid message.");
                    return Ok(());
                }

                println!("{}", final_msg);

                // 5. Copy to Clipboard (Native macOS)
                if let Ok(mut clipboard) = Clipboard::new() {
                    if let Err(e) = clipboard.set_text(final_msg) {
                        eprintln!("[ERROR] Could not copy to clipboard: {}", e);
                    } else {
                        eprintln!("[DONE] Message copied to clipboard. Press Cmd+V to paste.");
                    }
                }
            } else {
                eprintln!("[ERROR] Ollama API returned error: {}", res.status());
            }
        }
        Err(e) => {
            eprintln!(
                "[ERROR] Could not connect to Ollama: {}. Is the app running?",
                e
            );
        }
    }

    Ok(())
}
