use asum::shell::{
    escape_posix_single_quoted, is_affirmative_confirm, should_commit_after_confirm,
};

#[test]
fn test_escape_posix_single_quoted_empty_string() {
    assert_eq!(escape_posix_single_quoted(""), "''");
}

#[test]
fn test_escape_posix_single_quoted_plain_text() {
    assert_eq!(
        escape_posix_single_quoted("fix(ui): correct button alignment"),
        "'fix(ui): correct button alignment'"
    );
}

#[test]
fn test_escape_posix_single_quoted_single_quote() {
    assert_eq!(
        escape_posix_single_quoted("fix: don't break"),
        "'fix: don'\\''t break'"
    );
}

#[test]
fn test_escape_posix_single_quoted_double_quotes_and_specials() {
    let msg = "feat: add \"quotes\", $HOME, `cmd`, and bang!";
    let escaped = escape_posix_single_quoted(msg);
    assert_eq!(escaped, "'feat: add \"quotes\", $HOME, `cmd`, and bang!'");
}

#[test]
fn test_escape_posix_single_quoted_newlines_in_body() {
    let msg = "feat(auth): implement oauth2 login flow\n\n- Add Google support.\n";
    let escaped = escape_posix_single_quoted(msg);
    assert_eq!(
        escaped,
        "'feat(auth): implement oauth2 login flow\n\n- Add Google support.\n'"
    );
}

#[test]
fn test_escape_posix_single_quoted_multiple_single_quotes() {
    assert_eq!(escape_posix_single_quoted("a'b'c"), "'a'\\''b'\\''c'");
}

#[test]
fn test_is_affirmative_confirm_defaults_and_y() {
    assert!(is_affirmative_confirm(""));
    assert!(is_affirmative_confirm("\n"));
    assert!(is_affirmative_confirm("  "));
    assert!(is_affirmative_confirm("y"));
    assert!(is_affirmative_confirm("Y"));
    assert!(is_affirmative_confirm(" y\n"));
}

#[test]
fn test_is_affirmative_confirm_rejects_no_and_other() {
    assert!(!is_affirmative_confirm("n"));
    assert!(!is_affirmative_confirm("N"));
    assert!(!is_affirmative_confirm("no"));
    assert!(!is_affirmative_confirm("yes"));
    assert!(!is_affirmative_confirm("maybe"));
}

#[test]
fn test_should_commit_after_confirm_eof_is_abort() {
    assert!(!should_commit_after_confirm("", 0));
    assert!(!should_commit_after_confirm("y", 0));
}

#[test]
fn test_should_commit_after_confirm_enter_and_y() {
    assert!(should_commit_after_confirm("\n", 1));
    assert!(should_commit_after_confirm("y\n", 2));
    assert!(should_commit_after_confirm("Y\n", 2));
    assert!(!should_commit_after_confirm("n\n", 2));
}
