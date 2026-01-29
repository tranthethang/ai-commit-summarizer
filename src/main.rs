mod config;
mod db;
mod git;
mod summarizer;

use crate::config::{AsumConfig, verify_toml};
use crate::db::init_db;
use crate::git::get_git_diff;
use crate::summarizer::get_summarizer;
use anyhow::Context;
use arboard::Clipboard;
use std::env;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args: Vec<String> = env::args().collect();

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
                            eprintln!("[ERROR] asum.toml syntax error: {}", e);
                            std::process::exit(1);
                        }
                    }
                } else {
                    eprintln!("[ERROR] asum.toml not found in the current directory.");
                    std::process::exit(1);
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
                eprintln!("[ERROR] Unknown command: {}", args[1]);
                println!("\nUsage:");
                println!("  asum         Generate commit summary from staged changes");
                println!("  asum verify  Verify the syntax of asum.toml");
                println!("  asum help    Show this help message");
                std::process::exit(1);
            }
        }
    }

    // Initialize Database
    let pool = init_db().await.context("Failed to initialize database")?;

    // Load Configuration (prioritize asum.toml)
    let config = AsumConfig::load(&pool)
        .await
        .context("Failed to load configuration")?;

    // 1. Get git diff
    let mut diff_text = get_git_diff().context("Failed to get git diff")?;

    if diff_text.is_empty() {
        eprintln!("[WARN] No staged changes found in supported code files.");
        return Ok(());
    }

    // 2. Optimize diff size
    let max_diff_length = config.max_diff_length;

    if diff_text.len() > max_diff_length {
        eprintln!(
            "[INFO] Diff is too large ({} bytes), truncating to {} bytes for AI...",
            diff_text.len(),
            max_diff_length
        );
        eprintln!(
            "[INFO] You can increase this limit by updating 'max_diff_length' in your config."
        );
        diff_text = diff_text.chars().take(max_diff_length).collect();
    }

    eprintln!("[INFO] AI is analyzing your changes...");

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
                    eprintln!("[ERROR] Could not copy to clipboard: {}", e);
                } else {
                    eprintln!("[DONE] Message copied to clipboard. Press Cmd+V to paste.");
                }
            }
        }
        Err(e) => {
            eprintln!("[ERROR] Summarization failed: {}", e);
        }
    }

    Ok(())
}
