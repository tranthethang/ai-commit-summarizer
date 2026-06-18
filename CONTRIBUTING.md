# Contributing to asum

Thank you for your interest in contributing to asum (AI Commit Summarizer)!

This document outlines the guidelines and best practices for developing and contributing to this project.

## Development Environment Setup

To get started, you need to have the Rust toolchain installed. The Minimum Supported Rust Version (MSRV) is 1.85.

1. Clone the repository:
   ```bash
   git clone https://github.com/tranthethang/ai-commit-summarizer.git
   cd ai-commit-summarizer
   ```

2. Build the project:
   ```bash
   make build
   ```

## Coding Conventions

Please adhere to the following rules when working on the codebase:

### Language
- All code, comments, documentation, commit messages, and pull request descriptions must be in English.

### Naming and Style
- Follow standard Rust naming conventions (RFC 430):
  - Structs, enums, traits, and type aliases: `PascalCase`.
  - Local variables, functions, methods, modules, and macro names: `snake_case`.
  - Constants and static variables: `SCREAMING_SNAKE_CASE`.
- Do not leave commented-out code. Use Git to track history.
- Minimise the use of `#[allow(...)]` attributes.

### Formatting and Linting
- Always format and lint your code using `make format` (which runs `cargo fmt` and `cargo clippy`) before submitting. Fix all warnings; the CI builds will fail if there are any formatting errors or clippy warnings.

### Error Handling
- Avoid calling `unwrap()` or `expect()` in production code.
- Prefer propagating errors using `Result<T, E>` and the `?` operator.

## Testing

Always run tests to ensure your changes do not break existing functionality. All unit and integration tests are isolated in the `tests/` directory at the project root.

You can run the tests concurrently using:

```bash
make test
```

If you prefer to run them sequentially, or if you want to isolate environment checks, you can use:

```bash
make test-sequential
```

## Pull Request Checklist

Before submitting a pull request, please make sure you have:
1. Formatted and linted the code with `make format`.
2. Verified all tests pass with `make test` (or `make test-sequential`).
3. Documented any public APIs with `///` doc comments.
4. Updated the CHANGELOG.md if you added new features or fixed bugs.
