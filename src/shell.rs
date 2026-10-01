//! Shell escaping and confirm helpers for paste-safe / force-commit flows.

/// Escapes `s` as a POSIX single-quoted string suitable for pasting after
/// `git commit -m ` in bash, zsh, or other POSIX shells.
///
/// Single quotes inside the message are encoded as `'\''` (end quote,
/// escaped literal quote, reopen quote).
pub fn escape_posix_single_quoted(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('\'');
    for ch in s.chars() {
        if ch == '\'' {
            out.push_str("'\\''");
        } else {
            out.push(ch);
        }
    }
    out.push('\'');
    out
}

/// Interprets a confirm prompt answer for force-commit.
///
/// Empty input (Enter), `y`, and `Y` mean yes. Everything else means no.
/// Callers must not treat stdin EOF (`read_line` returning 0 bytes) as empty
/// Enter — use [`should_commit_after_confirm`] instead.
pub fn is_affirmative_confirm(answer: &str) -> bool {
    let trimmed = answer.trim();
    trimmed.is_empty() || trimmed.eq_ignore_ascii_case("y")
}

/// Decides whether to proceed after `stdin().read_line(...)`.
///
/// `bytes_read == 0` is EOF and always aborts, so a closed/piped-empty stdin
/// cannot silently default to yes.
pub fn should_commit_after_confirm(answer: &str, bytes_read: usize) -> bool {
    bytes_read > 0 && is_affirmative_confirm(answer)
}
