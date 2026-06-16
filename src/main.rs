//! ASUM - AI Commit Summarizer
//!
//! This tool automatically generates professional commit messages based on staged changes
//! using AI providers like Google Gemini or local Ollama instances.

use anyhow::Context;
use arboard::Clipboard;
use asum::config::{AsumConfig, verify_toml};
use asum::payload::prepare_payload;
use asum::summarizer::get_summarizer;
use tracing::{error, info, warn};
use tracing_subscriber::{EnvFilter, fmt, prelude::*};

use clap::{Parser, Subcommand};

/// Command-line arguments for asum.
#[derive(Parser, Debug)]
#[command(
    name = "asum",
    version,
    about = "AI Commit Summarizer - Generate professional commit messages using AI"
)]
pub struct Cli {
    /// Enable verbose output (print prompts and raw API responses)
    #[arg(short, long, global = true)]
    pub verbose: bool,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

/// Available subcommands for asum.
#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Verify the syntax and completeness of asum.toml
    Verify,
}

/// Entry point of the application.
/// Sets up logging and parses command line arguments to run the app.
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging directory at ~/.asum/logs
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

    let cli = Cli::parse();
    run_app(cli).await
}

/// Core logic for processing command line arguments and executing commands.
pub async fn run_app(cli: Cli) -> anyhow::Result<()> {
    if let Some(Commands::Verify) = cli.command {
        return handle_verify_command();
    }

    // Load Configuration (prioritize local asum.toml, then ~/.asum/asum.toml)
    let config = AsumConfig::load().context("Failed to load configuration")?;

    let payload = prepare_payload(&config)?;
    if payload.is_empty() {
        warn!("No staged changes found.");
        return Ok(());
    }

    info!("AI is analyzing your changes...");

    // 4. Initialize the AI summarizer based on the active provider (e.g., Gemini, Ollama)
    let summarizer = get_summarizer(config, cli.verbose)
        .await
        .context("Failed to get summarizer")?;

    // 5. Request the AI to generate a commit message based on the payload
    match summarizer.summarize(&payload).await {
        Ok(final_msg) => {
            println!("{}", final_msg);
            handle_output(final_msg);
        }
        Err(e) => {
            error!("Summarization failed: {}", e);
            return Err(e);
        }
    }

    Ok(())
}

fn handle_verify_command() -> anyhow::Result<()> {
    // Find local config path
    let local_path = std::path::Path::new("asum.toml");
    let global_path = home::home_dir().map(|mut p| {
        p.push(".asum");
        p.push("asum.toml");
        p
    });

    let config_path = if local_path.exists() {
        Some(local_path.to_path_buf())
    } else if let Some(ref p) = global_path {
        if p.exists() { Some(p.clone()) } else { None }
    } else {
        None
    };

    if let Some(path) = config_path {
        match verify_toml(&path) {
            Ok(_) => {
                println!("[OK] {} syntax is valid.", path.display());
                Ok(())
            }
            Err(e) => {
                error!("{} syntax error: {}", path.display(), e);
                Err(anyhow::anyhow!("{} syntax error: {}", path.display(), e))
            }
        }
    } else {
        error!("Configuration file 'asum.toml' not found locally or in ~/.asum/asum.toml");
        Err(anyhow::anyhow!("asum.toml not found"))
    }
}

fn handle_output(final_msg: String) {
    if let Ok(mut clipboard) = Clipboard::new() {
        if let Err(e) = clipboard.set_text(final_msg) {
            error!("Could not copy to clipboard: {}", e);
        } else {
            info!("Message copied to clipboard. Press Cmd+V to paste.");
        }
    }
}
