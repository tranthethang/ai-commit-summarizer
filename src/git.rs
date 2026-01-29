use std::process::Command;

pub fn get_git_diff() -> anyhow::Result<String> {
    let output = Command::new("git")
        .args([
            "diff",
            "--cached",
            "--",
            "*.java",
            "*.php",
            "*.js",
            "*.jsx",
            "*.ts",
            "*.tsx",
            "*.scss",
            "*.css",
            "*.rs",
            "*.py",
            "*.go",
            "*.c",
            "*.cpp",
            ":(exclude)*-lock.json",
            ":(exclude)package-lock.json",
            ":(exclude)pnpm-lock.yaml",
            ":(exclude)*.min.js",
        ])
        .output()?;

    let diff_text = String::from_utf8_lossy(&output.stdout).to_string();
    Ok(diff_text)
}
