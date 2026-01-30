use std::process::Command;

pub fn get_git_diff(extensions: &[String]) -> anyhow::Result<String> {
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

    let output = Command::new("git").args(args).output()?;

    let diff_text = String::from_utf8_lossy(&output.stdout).to_string();
    Ok(diff_text)
}
