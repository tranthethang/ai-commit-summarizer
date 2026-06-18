# Repository Rules for AI Agents

This document contains rules, guidelines, and best practices for AI agents contributing to this codebase.

## Language and Localization

- **English Only**: All source code, documentation, inline comments, docstrings, commit messages, pull requests, and issue descriptions must be written in English.
- **Translation of Legacy Code**: If you encounter any comments, logs, or documentation written in other languages (such as Vietnamese), translate them to English.

## Documentation Guidelines

- **No Emojis**: Do not use emojis in `.md` (Markdown) files (including `README.md`, guides, or documentation in `docs/`). Emoji icons are inconsistent in style, color, and size across different platforms and editors. Use clean, text-based headers and lists instead.

## Coding Conventions for Rust

### Naming Conventions
Follow standard Rust naming conventions (RFC 430):
- **Types and Containers**: Use `PascalCase` for structs, enums, traits, unions, and type aliases (e.g., `GeminiSummarizer`, `ConfigError`).
- **Variables and Functions**: Use `snake_case` for local variables, function and method names, modules, struct fields, and macro names (e.g., `load_config`, `summarize_commit`).
- **Constants and Statics**: Use `SCREAMING_SNAKE_CASE` for constants and static variables (e.g., `DEFAULT_TIMEOUT_SECS`).
- **Type Parameters**: Use `PascalCase` with single upper-case letters (e.g., `T`, `U`) or descriptive names (e.g., `Key`, `Value`).
- **Acronyms**: Treat acronyms as words in `PascalCase` (e.g., `XmlHttpRequest` instead of `XMLHTTPRequest`, `VertexAi` instead of `VertexAI`).

### Formatting and Linting
- **Cargo Fmt**: Always format your code using `cargo fmt` before proposing/submitting changes. (Alternatively, run `make format` to run both formatting and clippy checks.)
- **Clippy**: Run `cargo clippy` to check for common mistakes and code improvements. Resolve all warnings before merging. (Alternatively, run `make format`.)
- **Allow Attributes**: Minimize the use of `#[allow(...)]`. When using it, always document why the lint warning is safely ignored.

### Error Handling
- **No Panic**: Avoid `unwrap()` and `expect()` in production/library code. Use them only in tests or when an invariant guarantees that a failure is impossible. If used, write a clear comment explaining why the invariant holds.
- **Error Propagation**: Prefer returning a `Result<T, E>` and propagating errors using the `?` operator.
- **Custom Error Types**: Use dedicated error types or crates (like `thiserror` or `anyhow` depending on the application context) to define domain-specific errors.

### Ownership, Borrowing, and Memory
- **Avoid Unnecessary Allocations**: Avoid calling `.clone()`, `.to_owned()`, or `.to_string()` unless it is necessary to transfer ownership. Prefer borrowing (`&str`, `&[T]`) over taking ownership of owned containers (`String`, `Vec<T>`) in function signatures where possible.
- **Smart Pointers**: Use `Rc`, `Arc`, `RefCell`, `Mutex`, and `Box` only when necessary for shared ownership, interior mutability, or dynamic dispatch.

### Testing
- **NO TESTS IN SRC**: NEVER write unit tests or integration tests (including `#[cfg(test)] mod tests`) inside any business logic files under the `./src` directory.
- **Isolated Testing**: Place both Unit and Integration tests exclusively in the `tests/` directory at the project root to keep the source directory clean.
- **Test Names**: Test function names should clearly describe the behavior being tested (e.g., `test_config_loading_fails_with_invalid_file`).

## Commenting Guidelines

- **Documentation Comments**: Use `///` for documenting public-facing structs, enums, traits, and functions. Use `//!` for module-level documentation. Write doc comments that describe the parameters, return values, errors, and panic conditions.
- **Inline Comments**: Use `//` for inline comments. Place them above the lines of code they describe, not inline on the same line, for better readability.
- **Focus on "Why" Not "What"**: Inline comments should explain the reasoning behind complex logic or why a specific approach was taken, rather than just repeating what the code does. If the code is complex, consider refactoring it for readability first.
- **Dead Code**: Do not leave commented-out blocks of code in the repository. Remove them and rely on version control (Git) to retrieve old code if needed.

## Developer Guides

- **Adding a New AI Provider**: For instructions on how to add and integrate a new AI provider, refer to the [Adding a New AI Provider Guide](file:///Users/thangtt/Documents/Github/ai-commit-summarizer/docs/adding-providers.md).


