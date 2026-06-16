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
fn split_diff_into_file_blocks(diff: &str) -> Vec<(String, String)> {
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
fn extract_filename_from_header(header: &str) -> &str {
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

/// Parses a staged files `--name-status` list and constructs a formatted tree view.
pub fn build_tree_view(staged_output: &str) -> String {
    let mut root = TreeNode::default();

    for line in staged_output.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let parts: Vec<&str> = line.split('\t').collect();
        if parts.is_empty() {
            continue;
        }

        let status_code = parts[0];
        let (status_desc, path) = if status_code.starts_with('R') && parts.len() >= 3 {
            (format!("Renamed from {}", parts[1]), parts[2])
        } else if status_code.starts_with('C') && parts.len() >= 3 {
            (format!("Copied from {}", parts[1]), parts[2])
        } else if parts.len() >= 2 {
            (map_status(status_code), parts[1])
        } else {
            continue;
        };

        let segments: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
        let mut current = &mut root;
        for (i, &segment) in segments.iter().enumerate() {
            let is_last = i == segments.len() - 1;
            if is_last {
                current
                    .children
                    .entry(segment.to_string())
                    .or_insert_with(TreeNode::default)
                    .status = Some(status_desc.clone());
            } else {
                current = current
                    .children
                    .entry(segment.to_string())
                    .or_insert_with(TreeNode::default);
            }
        }
    }

    if root.children.is_empty() {
        return String::new();
    }

    format_tree_node(".", &root, "", true, true)
}

/// Truncates a single file diff block, keeping only the top `max_hunks` largest hunks (by affected lines).
/// Preserves the original file order of the kept hunks.
fn truncate_hunks_per_file(file_block: &str, max_hunks: usize) -> String {
    if max_hunks == 0 {
        return file_block.to_string();
    }

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

    if hunks.len() <= max_hunks {
        return file_block.to_string();
    }

    let mut indexed_hunks: Vec<(usize, String, usize)> = hunks
        .into_iter()
        .enumerate()
        .map(|(idx, (text, affected))| (idx, text, affected))
        .collect();

    indexed_hunks.sort_by(|a, b| b.2.cmp(&a.2));
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

#[cfg(test)]
mod tests {
    use super::extract_filename_from_header;
    use super::split_diff_into_file_blocks;
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use std::process::Command;
    use tempfile::tempdir;
    fn make_file_block(name: &str, lines: usize) -> String {
        let mut s = format!("diff --git a/{name} b/{name}\n--- a/{name}\n+++ b/{name}\n");
        for i in 0..lines {
            s.push_str(&format!("+line {i}\n"));
        }
        s
    }
    #[test]
    fn test_get_git_diff_no_staged() {
        let dir = tempdir().unwrap();
        let repo_path = dir.path();

        Command::new("git")
            .arg("init")
            .current_dir(repo_path)
            .output()
            .unwrap();

        let diff =
            get_git_diff_in_path(&["*.rs".to_string()], repo_path.to_str().unwrap()).unwrap();
        assert!(diff.is_empty());
    }

    #[test]
    fn test_get_git_diff_with_staged_table_driven() {
        struct TestCase {
            name: &'static str,
            filename: &'static str,
            content: &'static str,
            extension: &'static str,
            should_find: bool,
        }

        let cases = vec![
            TestCase {
                name: "find staged rust file",
                filename: "test.rs",
                content: "fn main() {}",
                extension: "*.rs",
                should_find: true,
            },
            TestCase {
                name: "exclude non-matching extension",
                filename: "test.txt",
                content: "hello",
                extension: "*.rs",
                should_find: false,
            },
        ];

        for case in cases {
            let dir = tempdir().unwrap();
            let repo_path = dir.path();

            Command::new("git")
                .arg("init")
                .current_dir(repo_path)
                .output()
                .unwrap();

            let file_path = repo_path.join(case.filename);
            let mut file = File::create(&file_path).unwrap();
            writeln!(file, "{}", case.content).unwrap();

            Command::new("git")
                .args(["add", case.filename])
                .current_dir(repo_path)
                .output()
                .unwrap();

            let diff =
                get_git_diff_in_path(&[case.extension.to_string()], repo_path.to_str().unwrap())
                    .unwrap();
            if case.should_find {
                assert!(!diff.is_empty(), "Failed case: {}", case.name);
                assert!(diff.contains(case.content), "Failed case: {}", case.name);
            } else {
                assert!(diff.is_empty(), "Failed case: {}", case.name);
            }
        }
    }

    #[test]
    fn test_get_git_diff_exclude_patterns() {
        let dir = tempdir().unwrap();
        let repo_path = dir.path();

        Command::new("git")
            .arg("init")
            .current_dir(repo_path)
            .output()
            .unwrap();

        // Create a lock file that should be excluded
        let lock_file_path = repo_path.join("package-lock.json");
        let mut lock_file = File::create(&lock_file_path).unwrap();
        writeln!(lock_file, "{{\"name\": \"test\"}}").unwrap();

        Command::new("git")
            .args(["add", "package-lock.json"])
            .current_dir(repo_path)
            .output()
            .unwrap();

        let diff =
            get_git_diff_in_path(&["*.json".to_string()], repo_path.to_str().unwrap()).unwrap();
        assert!(diff.is_empty(), "package-lock.json should be excluded");

        // Create a normal json file that should NOT be excluded
        let normal_file_path = repo_path.join("test.json");
        let mut normal_file = File::create(&normal_file_path).unwrap();
        writeln!(normal_file, "{{\"test\": true}}").unwrap();

        Command::new("git")
            .args(["add", "test.json"])
            .current_dir(repo_path)
            .output()
            .unwrap();

        let diff =
            get_git_diff_in_path(&["*.json".to_string()], repo_path.to_str().unwrap()).unwrap();
        assert!(!diff.is_empty(), "test.json should be included");
        assert!(diff.contains("test.json"));
    }

    #[test]
    fn test_get_git_diff_smoke() {
        // Just a smoke test to ensure it doesn't crash in the current repo
        let result = get_git_diff(&["*.rs".to_string()]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_extract_filename_from_header() {
        assert_eq!(
            extract_filename_from_header("diff --git a/src/main.rs b/src/main.rs"),
            "src/main.rs"
        );
        assert_eq!(
            extract_filename_from_header("diff --git a/foo.ts b/foo.ts"),
            "foo.ts"
        );
    }

    #[test]
    fn test_split_diff_into_file_blocks() {
        let block_a = make_file_block("a.rs", 1);
        let block_b = make_file_block("b.rs", 1);
        let diff = format!("{block_a}{block_b}");
        let blocks = split_diff_into_file_blocks(&diff);
        assert_eq!(blocks.len(), 2);
        assert!(blocks[0].0.contains("a.rs"));
        assert!(blocks[1].0.contains("b.rs"));
    }

    #[test]
    fn test_get_staged_files() {
        let dir = tempdir().unwrap();
        let repo_path = dir.path();

        Command::new("git")
            .arg("init")
            .current_dir(repo_path)
            .output()
            .unwrap();

        let file_path = repo_path.join("test.txt");
        File::create(&file_path).unwrap();

        Command::new("git")
            .args(["add", "test.txt"])
            .current_dir(repo_path)
            .output()
            .unwrap();

        let files = get_staged_files_in_path(repo_path.to_str().unwrap()).unwrap();
        assert!(files.contains("A\ttest.txt"));
    }

    #[test]
    fn test_build_tree_view_empty() {
        assert_eq!(build_tree_view(""), "");
        assert_eq!(build_tree_view("   \n  \n"), "");
    }

    #[test]
    fn test_build_tree_view_standard() {
        let input = "M\tsrc/main.rs\nA\tsrc/git.rs\nD\tCargo.toml\n";
        let tree = build_tree_view(input);

        let expected = ".\n├── Cargo.toml (Deleted)\n└── src/\n    ├── git.rs (Added)\n    └── main.rs (Modified)\n";
        assert_eq!(tree, expected);
    }

    #[test]
    fn test_build_tree_view_renamed_and_copied() {
        let input = "R100\told.rs\tnew.rs\nC085\torigin.rs\tcopy.rs\n";
        let tree = build_tree_view(input);

        let expected = ".\n├── copy.rs (Copied from origin.rs)\n└── new.rs (Renamed from old.rs)\n";
        assert_eq!(tree, expected);
    }

    #[test]
    fn test_truncate_hunks_per_file() {
        let file_diff = "diff --git a/a.rs b/a.rs\nindex 123..456\n--- a/a.rs\n+++ b/a.rs\n\
                         @@ -1,5 +1,5 @@\n-old1\n+new1\n\
                         @@ -10,10 +10,12 @@\n-old2\n-old22\n+new2\n+new22\n+new23\n\
                         @@ -30,5 +30,5 @@\n-old3\n+new3\n";

        // With max_hunks = 1, it should keep only the second hunk (which has 5 affected lines vs 2 in others)
        let truncated = truncate_hunks_per_file(file_diff, 1);
        assert!(truncated.contains("@@ -10,10 +10,12 @@"));
        assert!(!truncated.contains("@@ -1,5 +1,5 @@"));
        assert!(!truncated.contains("@@ -30,5 +30,5 @@"));

        // With max_hunks = 2, it should keep hunk 2 (5 affected) and hunk 1 or 3 (2 affected).
        // Since we sort back to original order, it should preserve relative positions.
        let truncated_2 = truncate_hunks_per_file(file_diff, 2);
        assert!(truncated_2.contains("@@ -1,5 +1,5 @@"));
        assert!(truncated_2.contains("@@ -10,10 +10,12 @@"));
        assert!(!truncated_2.contains("@@ -30,5 +30,5 @@"));
    }

    #[test]
    fn test_process_and_truncate_diff_modes() {
        let file_diff = "diff --git a/a.rs b/a.rs\nindex 123..456\n--- a/a.rs\n+++ b/a.rs\n\
                         @@ -1,5 +1,5 @@\n-old1\n+new1\n\
                         @@ -10,10 +10,12 @@\n-old2\n-old22\n+new2\n+new22\n+new23\n\
                         @@ -30,5 +30,5 @@\n-old3\n+new3\n";

        // Test hunk reduction mode
        let result = process_and_truncate_diff(file_diff, 10000, DiffReductionMode::Hunk, 1);
        assert!(result.contains("@@ -10,10 +10,12 @@"));
        assert!(!result.contains("@@ -1,5 +1,5 @@"));

        // Test file reduction mode (which does not do hunk truncation)
        let result_file = process_and_truncate_diff(file_diff, 10000, DiffReductionMode::File, 1);
        assert!(result_file.contains("@@ -1,5 +1,5 @@"));
        assert!(result_file.contains("@@ -10,10 +10,12 @@"));
        assert!(result_file.contains("@@ -30,5 +30,5 @@"));
    }

    #[test]
    fn test_get_git_diff_not_a_repo() {
        let dir = tempdir().unwrap();
        let result = get_git_diff_in_path(&["*.rs".to_string()], dir.path().to_str().unwrap());
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("git diff failed"));
    }
}
