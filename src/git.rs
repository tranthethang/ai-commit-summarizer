use std::process::Command;

pub fn get_git_diff(extensions: &[String]) -> anyhow::Result<String> {
    get_git_diff_in_path(extensions, ".")
}

pub fn get_git_diff_in_path(extensions: &[String], path: &str) -> anyhow::Result<String> {
    let mut args = vec!["diff", "--cached", "--"];
    for ext in extensions {
        args.push(ext);
    }
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

pub fn get_staged_files() -> anyhow::Result<String> {
    get_staged_files_in_path(".")
}

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

#[cfg(test)]
mod tests {
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
