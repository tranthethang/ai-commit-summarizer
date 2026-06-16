use asum::config::DiffReductionMode;
use asum::git::*;
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

    let diff = get_git_diff_in_path(&["*.rs".to_string()], repo_path.to_str().unwrap()).unwrap();
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

        let diff = get_git_diff_in_path(&[case.extension.to_string()], repo_path.to_str().unwrap())
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

    let diff = get_git_diff_in_path(&["*.json".to_string()], repo_path.to_str().unwrap()).unwrap();
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

    let diff = get_git_diff_in_path(&["*.json".to_string()], repo_path.to_str().unwrap()).unwrap();
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
