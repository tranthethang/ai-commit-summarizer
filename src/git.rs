//! Git utility module for ASUM.
//!
//! This module interacts with the Git CLI to retrieve staged changes
//! and file lists for AI analysis.

use crate::config::DiffReductionMode;
use std::collections::BTreeMap;
use std::process::Command;

/// Retrieves the git diff of staged changes for the specified file extensions in the current directory.
pub fn get_git_diff(extensions: &[String]) -> anyhow::Result<String> {
    get_git_diff_in_path(extensions, ".")
}

/// Retrieves the git diff of staged changes for the specified file extensions in a specific directory.
/// It excludes common lock files and minified scripts to keep the diff clean.
pub fn get_git_diff_in_path(extensions: &[String], path: &str) -> anyhow::Result<String> {
    let mut args = vec!["diff", "--cached", "--"];
    // Add file patterns to include based on configuration
    for ext in extensions {
        args.push(ext);
    }
    // Explicitly exclude generated or binary-like files that aren't useful for summaries
    args.extend([
        ":(exclude)*-lock.json",
        ":(exclude)package-lock.json",
        ":(exclude)pnpm-lock.yaml",
        ":(exclude)*.min.js",
    ]);

    let output = Command::new("git").args(args).current_dir(path).output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("git diff failed: {}", stderr.trim());
    }

    let diff_text = String::from_utf8_lossy(&output.stdout).to_string();
    Ok(diff_text)
}

/// Retrieves a list of staged files and their status in the current directory.
pub fn get_staged_files() -> anyhow::Result<String> {
    get_staged_files_in_path(".")
}

/// Retrieves a list of staged files and their status in a specific directory.
/// This is used as a fallback when no code diff is available.
pub fn get_staged_files_in_path(path: &str) -> anyhow::Result<String> {
    let args = vec![
        "diff",
        "--cached",
        "--name-status",
        "--",
        ":(exclude)*-lock.json",
        ":(exclude)package-lock.json",
        ":(exclude)pnpm-lock.yaml",
        ":(exclude)*.min.js",
    ];
    let output = Command::new("git").args(args).current_dir(path).output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("git diff failed: {}", stderr.trim());
    }

    let files_text = String::from_utf8_lossy(&output.stdout).to_string();
    Ok(files_text)
}

/// Splits a unified diff string into per-file blocks, keyed on the `diff --git a/... b/...` header line.
/// Returns a vector of `(header_line, full_block_text)` pairs in order of appearance.
pub fn split_diff_into_file_blocks(diff: &str) -> Vec<(String, String)> {
    let mut blocks: Vec<(String, String)> = Vec::new();
    let mut current_header = String::new();
    let mut current_lines: Vec<&str> = Vec::new();

    for line in diff.lines() {
        if line.starts_with("diff --git ") {
            if !current_header.is_empty() {
                blocks.push((current_header.clone(), current_lines.join("\n") + "\n"));
            }
            current_header = line.to_string();
            current_lines = vec![line];
        } else {
            current_lines.push(line);
        }
    }

    if !current_header.is_empty() {
        blocks.push((current_header, current_lines.join("\n") + "\n"));
    }

    blocks
}

/// Extracts the destination file path from a `diff --git` header line.
/// e.g. `diff --git a/src/main.rs b/src/main.rs` -> `src/main.rs`
pub fn extract_filename_from_header(header: &str) -> &str {
    // The destination side is after " b/"
    if let Some(pos) = header.rfind(" b/") {
        return &header[pos + 3..];
    }
    header
}

#[derive(Debug, Default)]
struct TreeNode {
    status: Option<String>,
    children: BTreeMap<String, TreeNode>,
}

fn map_status(status_code: &str) -> String {
    match status_code {
        "A" => "Added".to_string(),
        "M" => "Modified".to_string(),
        "D" => "Deleted".to_string(),
        "R" => "Renamed".to_string(),
        "C" => "Copied".to_string(),
        "U" => "Updated but unmerged".to_string(),
        s => s.to_string(),
    }
}

fn format_tree_node(
    name: &str,
    node: &TreeNode,
    prefix: &str,
    is_last: bool,
    is_root: bool,
) -> String {
    let mut result = String::new();

    if !is_root {
        result.push_str(prefix);
        if is_last {
            result.push_str("└── ");
        } else {
            result.push_str("├── ");
        }

        if let Some(ref status) = node.status {
            result.push_str(&format!("{} ({})\n", name, status));
        } else {
            result.push_str(&format!("{}/\n", name));
        }
    } else {
        result.push_str(".\n");
    }

    let next_prefix = if is_root {
        "".to_string()
    } else if is_last {
        format!("{}    ", prefix)
    } else {
        format!("{}│   ", prefix)
    };

    let child_count = node.children.len();
    for (i, (child_name, child_node)) in node.children.iter().enumerate() {
        let child_is_last = i == child_count - 1;
        result.push_str(&format_tree_node(
            child_name,
            child_node,
            &next_prefix,
            child_is_last,
            false,
        ));
    }

    result
}

fn parse_status_line(line: &str) -> Option<(String, &str)> {
    let parts: Vec<&str> = line.split('\t').collect();
    if parts.is_empty() {
        return None;
    }
    let status_code = parts[0];
    if status_code.starts_with('R') && parts.len() >= 3 {
        Some((format!("Renamed from {}", parts[1]), parts[2]))
    } else if status_code.starts_with('C') && parts.len() >= 3 {
        Some((format!("Copied from {}", parts[1]), parts[2]))
    } else if parts.len() >= 2 {
        Some((map_status(status_code), parts[1]))
    } else {
        None
    }
}

fn insert_path_into_tree(root: &mut TreeNode, path: &str, status_desc: String) {
    let segments: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
    let mut current = root;
    for (i, &segment) in segments.iter().enumerate() {
        let is_last = i == segments.len() - 1;
        if is_last {
            current
                .children
                .entry(segment.to_string())
                .or_default()
                .status = Some(status_desc.clone());
        } else {
            current = current.children.entry(segment.to_string()).or_default();
        }
    }
}

/// Parses a staged files `--name-status` list and constructs a formatted tree view.
pub fn build_tree_view(staged_output: &str) -> String {
    let mut root = TreeNode::default();
    for line in staged_output.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if let Some((status_desc, path)) = parse_status_line(line) {
            insert_path_into_tree(&mut root, path, status_desc);
        }
    }
    if root.children.is_empty() {
        return String::new();
    }
    format_tree_node(".", &root, "", true, true)
}

fn parse_hunks(file_block: &str) -> (Vec<&str>, Vec<(String, usize)>) {
    let mut header_lines = Vec::new();
    let mut hunks: Vec<(String, usize)> = Vec::new();
    let mut current_hunk = String::new();
    let mut current_hunk_affected = 0;
    let mut parsing_hunks = false;

    for line in file_block.lines() {
        if line.starts_with("@@ ") {
            parsing_hunks = true;
            if !current_hunk.is_empty() {
                hunks.push((current_hunk.clone(), current_hunk_affected));
            }
            current_hunk = line.to_string() + "\n";
            current_hunk_affected = 0;
        } else if !parsing_hunks {
            header_lines.push(line);
        } else {
            current_hunk.push_str(line);
            current_hunk.push('\n');
            if line.starts_with('+') || line.starts_with('-') {
                current_hunk_affected += 1;
            }
        }
    }

    if !current_hunk.is_empty() {
        hunks.push((current_hunk, current_hunk_affected));
    }

    (header_lines, hunks)
}

/// Truncates a single file diff block, keeping only the top `max_hunks` largest hunks (by affected lines).
/// Preserves the original file order of the kept hunks.
pub fn truncate_hunks_per_file(file_block: &str, max_hunks: usize) -> String {
    if max_hunks == 0 {
        return file_block.to_string();
    }

    let (header_lines, hunks) = parse_hunks(file_block);

    if hunks.len() <= max_hunks {
        return file_block.to_string();
    }

    let mut indexed_hunks: Vec<(usize, String, usize)> = hunks
        .into_iter()
        .enumerate()
        .map(|(idx, (text, affected))| (idx, text, affected))
        .collect();

    indexed_hunks.sort_by_key(|b| std::cmp::Reverse(b.2));
    indexed_hunks.truncate(max_hunks);
    indexed_hunks.sort_by_key(|h| h.0);

    let mut reconstructed = header_lines.join("\n") + "\n";
    for (_, hunk_text, _) in indexed_hunks {
        reconstructed.push_str(&hunk_text);
    }
    reconstructed
}

/// Processes a git diff string, optionally reducing it by keeping only the top hunks per file,
/// and then truncates the result to fit within `max_len` characters.
pub fn process_and_truncate_diff(
    diff: &str,
    max_len: usize,
    mode: DiffReductionMode,
    max_hunks: usize,
) -> String {
    if diff.is_empty() {
        return diff.to_string();
    }

    let blocks = split_diff_into_file_blocks(diff);

    let processed_blocks: Vec<(String, String)> = if mode == DiffReductionMode::Hunk {
        blocks
            .into_iter()
            .map(|(header, block)| {
                let truncated_block = truncate_hunks_per_file(&block, max_hunks);
                (header, truncated_block)
            })
            .collect()
    } else {
        blocks
    };

    let mut reconstructed_diff = String::new();
    for (_, block) in &processed_blocks {
        reconstructed_diff.push_str(block);
    }

    if reconstructed_diff.len() <= max_len {
        return reconstructed_diff;
    }

    let mut output = String::new();
    let mut omitted: Vec<String> = Vec::new();

    for (header, block) in &processed_blocks {
        if output.len() + block.len() <= max_len {
            output.push_str(block);
        } else {
            omitted.push(extract_filename_from_header(header).to_string());
        }
    }

    if !omitted.is_empty() {
        let footer = format!(
            "\n[TRUNCATED: {} more file(s) not shown: {}]",
            omitted.len(),
            omitted.join(", ")
        );
        output.push_str(&footer);
    }

    output
}
