mod config;
mod git;
mod summarizer;

use crate::config::{AsumConfig, verify_toml};
use crate::git::get_git_diff;
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
                            std::process::exit(1);
                        }
                    }
                } else {
                    error!("asum.toml not found in the current directory.");
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
                error!("Unknown command: {}", args[1]);
                println!("\nUsage:");
                println!("  asum         Generate commit summary from staged changes");
                println!("  asum verify  Verify the syntax of asum.toml");
                println!("  asum help    Show this help message");
                std::process::exit(1);
            }
        }
    }

    // Load Configuration (prioritize local asum.toml, then ~/.asum/asum.toml)
    let config = AsumConfig::load().context("Failed to load configuration")?;

    // 1. Get git diff
    let mut diff_text = get_git_diff(&config.git_extensions).context("Failed to get git diff")?;

    if diff_text.is_empty() {
        warn!("No staged changes found in supported code files.");
        return Ok(());
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
        }
    }

    Ok(())
}
