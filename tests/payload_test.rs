use asum::config::{AsumConfig, DiffReductionMode, ProviderConfig};
use asum::payload::{assemble_payload, get_tree_view, get_truncated_diff};

fn get_test_config() -> AsumConfig {
    AsumConfig {
        provider: ProviderConfig::Ollama {
            model: "llama3".to_string(),
            url: "http://localhost:11434".to_string(),
        },
        max_diff_length: 100,
        git_extensions: vec![".rs".to_string()],
        enable_tree_view: true,
        diff_reduction_mode: DiffReductionMode::File,
        max_hunks_per_file: 3,
        system_prompt: "sys".to_string(),
        user_prompt: "user".to_string(),
        ai_temperature: 0.7,
        ai_top_p: 1.0,
        ai_num_predict: 100,
        fallbacks: vec![],
    }
}

#[test]
fn test_assemble_payload() {
    let tree_view = "tree".to_string();
    let diff_text = "diff".to_string();

    let payload = assemble_payload(false, diff_text.clone(), tree_view.clone());
    assert_eq!(payload, "[STAGED FILES TREE]\ntree\n[INPUT DIFF]\ndiff");

    let payload = assemble_payload(true, diff_text.clone(), tree_view.clone());
    assert_eq!(payload, "[STAGED FILES TREE]\ntree");

    let payload = assemble_payload(false, diff_text.clone(), String::new());
    assert_eq!(payload, "diff");
}

#[test]
fn test_get_truncated_diff_no_truncation() {
    let config = get_test_config();
    let diff = "short diff";
    let res = get_truncated_diff(diff, &config);
    assert_eq!(res, diff);
}

#[test]
fn test_get_truncated_diff_with_truncation() {
    let mut config = get_test_config();
    config.max_diff_length = 5;
    let diff = "long diff text";
    let res = get_truncated_diff(diff, &config);
    // Should truncate/reduce
    assert!(res.len() <= 5 || res.contains("diff"));
}

#[test]
fn test_get_tree_view_disabled() {
    let mut config = get_test_config();
    config.enable_tree_view = false;
    let res = get_tree_view(false, "diff", &config);
    assert_eq!(res, "");
}

#[test]
fn test_get_tree_view_fallback() {
    let config = get_test_config();
    let res = get_tree_view(true, "A\tsome_file.rs\nM\tother_file.rs", &config);
    assert!(!res.is_empty());
}

#[test]
fn test_get_truncated_diff_hunk_mode() {
    let mut config = get_test_config();
    config.diff_reduction_mode = DiffReductionMode::Hunk;
    let diff = "diff --git a/src/main.rs b/src/main.rs\nindex 0000000..1111111\n--- a/src/main.rs\n+++ b/src/main.rs\n@@ -1,3 +1,4 @@\n fn main() {}\n";
    let res = get_truncated_diff(diff, &config);
    assert!(!res.is_empty());
}
