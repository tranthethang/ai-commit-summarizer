//! Git utility module for ASUM.
//!
//! This module interacts with the Git CLI to retrieve staged changes
//! and file lists for AI analysis.

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

/// Smartly truncates a git diff to fit within `max_len` characters.
///
/// Strategy (Option A - whole-file granularity):
/// 1. Parse the diff into per-file blocks at `diff --git` boundaries.
/// 2. Add each block to the output as long as the cumulative length stays within `max_len`.
/// 3. If a block would exceed the budget, skip it entirely and record the filename as omitted.
/// 4. If the diff was truncated, append a footer:
///    `[TRUNCATED: N more file(s) not shown: file_a.rs, file_b.ts]`
///
/// If the diff fits entirely within the budget, it is returned unchanged.
pub fn smart_truncate_diff(diff: &str, max_len: usize) -> String {
    // If the diff already fits, return early without any allocation.
    if diff.len() <= max_len {
        return diff.to_string();
    }

    let blocks = split_diff_into_file_blocks(diff);
    let mut output = String::new();
    let mut omitted: Vec<String> = Vec::new();

    for (header, block) in &blocks {
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

    // Safety fallback: if even the first file was larger than max_len,
    // output may still be empty (only footer). Return the footer so the AI
    // at least knows what was staged.
    output
}

#[cfg(test)]
mod tests {
    use super::extract_filename_from_header;
    use super::smart_truncate_diff;
    use super::split_diff_into_file_blocks;
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use std::process::Command;
    use tempfile::tempdir;

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

    // ---------- smart_truncate_diff tests ----------

    fn make_file_block(name: &str, lines: usize) -> String {
        let mut s = format!("diff --git a/{name} b/{name}\n--- a/{name}\n+++ b/{name}\n");
        for i in 0..lines {
            s.push_str(&format!("+line {i}\n"));
        }
        s
    }

    #[test]
    fn test_smart_truncate_diff_fits_entirely() {
        let diff = make_file_block("main.rs", 5);
        let result = smart_truncate_diff(&diff, 10_000);
        assert_eq!(result, diff, "Diff that fits should be returned unchanged");
    }

    #[test]
    fn test_smart_truncate_diff_empty() {
        let result = smart_truncate_diff("", 1000);
        assert_eq!(result, "");
    }

    #[test]
    fn test_smart_truncate_diff_single_file_too_large() {
        // A single file that exceeds the budget entirely -> only footer emitted.
        let diff = make_file_block("huge.rs", 100);
        let result = smart_truncate_diff(&diff, 10);
        assert!(
            result.contains("[TRUNCATED:"),
            "Should contain TRUNCATED footer"
        );
        assert!(
            result.contains("huge.rs"),
            "Footer should name the omitted file"
        );
    }

    #[test]
    fn test_smart_truncate_diff_multiple_files_partial() {
        let block_a = make_file_block("alpha.rs", 2);
        let block_b = make_file_block("beta.ts", 100); // too large
        let block_c = make_file_block("gamma.go", 100); // also too large
        let diff = format!("{block_a}{block_b}{block_c}");

        // Budget: just enough for block_a
        let budget = block_a.len() + 50;
        let result = smart_truncate_diff(&diff, budget);

        assert!(result.contains("alpha.rs"), "alpha.rs should be included");
        assert!(!result.contains("+line 0\n+line 1\n") || result.contains("alpha.rs"));
        assert!(
            result.contains("[TRUNCATED:"),
            "Should have TRUNCATED footer"
        );
        assert!(
            result.contains("beta.ts"),
            "beta.ts should be in omitted list"
        );
        assert!(
            result.contains("gamma.go"),
            "gamma.go should be in omitted list"
        );
        assert!(
            result.contains("2 more file(s)"),
            "Footer should count 2 omitted files"
        );
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
}
