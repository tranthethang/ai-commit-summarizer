mod db;
mod git;
mod summarizer;

use crate::db::{get_config, init_db};
use crate::git::get_git_diff;
use crate::summarizer::get_summarizer;
use anyhow::Context;
use arboard::Clipboard;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize Database
    let pool = init_db().await.context("Failed to initialize database")?;

    // 1. Get git diff
    let mut diff_text = get_git_diff().context("Failed to get git diff")?;

    if diff_text.is_empty() {
        eprintln!("[WARN] No staged changes found in supported code files.");
        return Ok(());
    }

    // 2. Optimize diff size
    let max_diff_length: usize = get_config(&pool, "max_diff_length")
        .await
        .context("Failed to get max_diff_length from config")?
        .unwrap_or_else(|| "1000000".to_string())
        .parse()
        .unwrap_or(1000000);

    if diff_text.len() > max_diff_length {
        eprintln!(
            "[INFO] Diff is too large ({} bytes), truncating to {} bytes for AI...",
            diff_text.len(),
            max_diff_length
        );
        eprintln!(
            "[INFO] You can increase this limit by updating 'max_diff_length' in your config database."
        );
        diff_text = diff_text.chars().take(max_diff_length).collect();
    }

    eprintln!("[INFO] AI is analyzing your changes...");

    // 3. Get Summarizer (Strategy Pattern)
    let summarizer = get_summarizer(&pool)
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
