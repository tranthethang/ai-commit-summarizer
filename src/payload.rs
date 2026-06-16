//! Payload preparation helper functions for ASUM.
//!
//! This module handles diff truncation, tree view building, and payload assembly.

use crate::config::{AsumConfig, DiffReductionMode};
use crate::git::{get_git_diff, get_staged_files, process_and_truncate_diff};
use anyhow::Context;
use tracing::{info, warn};

/// Get git diff of staged changes and perform smart truncation if it exceeds maximum limits.
pub fn get_truncated_diff(diff_text: &str, config: &AsumConfig) -> String {
    let max_diff_length = config.max_diff_length;
    if diff_text.len() > max_diff_length || config.diff_reduction_mode == DiffReductionMode::Hunk {
        let reduction_info = if config.diff_reduction_mode == DiffReductionMode::Hunk {
            format!(
                "applying hunk-level reduction (max {} hunks per file) and ",
                config.max_hunks_per_file
            )
        } else {
            "".to_string()
        };
        info!(
            "Diff is {} bytes, {}applying smart truncation to {} bytes...",
            diff_text.len(),
            reduction_info,
            max_diff_length
        );
        process_and_truncate_diff(
            diff_text,
            max_diff_length,
            config.diff_reduction_mode,
            config.max_hunks_per_file,
        )
    } else {
        diff_text.to_string()
    }
}

/// Construct a tree view representation of the staged files.
pub fn get_tree_view(is_fallback: bool, diff_text: &str, config: &AsumConfig) -> String {
    if config.enable_tree_view {
        let staged = if is_fallback {
            diff_text.to_string()
        } else {
            get_staged_files().unwrap_or_default()
        };
        if !staged.trim().is_empty() {
            crate::git::build_tree_view(&staged)
        } else {
            String::new()
        }
    } else {
        String::new()
    }
}

/// Assemble the final payload string combining tree view and diff text.
pub fn assemble_payload(is_fallback: bool, diff_text: String, tree_view: String) -> String {
    if !tree_view.is_empty() {
        if is_fallback {
            format!("[STAGED FILES TREE]\n{}", tree_view)
        } else {
            format!(
                "[STAGED FILES TREE]\n{}\n[INPUT DIFF]\n{}",
                tree_view, diff_text
            )
        }
    } else {
        diff_text
    }
}

/// Main entry point for preparing the prompt payload.
pub fn prepare_payload(config: &AsumConfig) -> anyhow::Result<String> {
    // 1. Extract the git diff of staged changes
    let mut diff_text = get_git_diff(&config.git_extensions).context("Failed to get git diff")?;

    let is_fallback = diff_text.is_empty();
    if is_fallback {
        warn!("No staged changes found in supported code files. Falling back to file list...");
        diff_text = get_staged_files().context("Failed to get staged files")?;
        if diff_text.is_empty() {
            return Ok(String::new());
        }
    }

    if !is_fallback {
        diff_text = get_truncated_diff(&diff_text, config);
    }

    let tree_view = get_tree_view(is_fallback, &diff_text, config);
    let payload = assemble_payload(is_fallback, diff_text, tree_view);

    Ok(payload)
}
