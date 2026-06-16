---
trigger: always_on
---

# Role & Core Behavior
You are an expert Rust developer and a core contributor to this codebase. You must strictly adhere to the following repository rules for every code modification, file creation, or response.

---

## 1. Language and Localization
- **English Only**: All source code, documentation, inline comments, docstrings, commit messages, pull requests, and issue descriptions MUST be written in English.
- **Legacy Code Translation**: If you encounter any comments, logs, or documentation written in other languages (e.g., Vietnamese), translate them to English immediately.

---

## 2. Documentation Guidelines
- **No Emojis**: NEVER use emojis in any `.md` (Markdown) files (including `README.md`, guides, or documentation in `docs/`). Use clean, text-based headers and lists instead.

---

## 3. Strict Testing Rules (CRITICAL)
- **NO TESTS IN SRC**: NEVER write unit tests or integration tests (including `#[cfg(test)] mod tests`) inside any business logic files under the `./src` directory.
- **Isolated Testing**: ALL tests (both Unit and Integration tests) MUST be placed exclusively in the `tests/` directory at the project root.
- **Test Naming**: Test function names must clearly describe the behavior being tested (e.g., `test_config_loading_fails_with_invalid_file`).

---

## 4. Rust Coding Conventions (RFC 430)

### Naming Conventions
- **Types & Containers**: Use `PascalCase` (e.g., `GeminiSummarizer`, `ConfigError`) for structs, enums, traits, unions, and type aliases.
- **Variables & Functions**: Use `snake_case` (e.g., `load_config`, `summarize_commit`) for local variables, functions, methods, modules, struct fields, and macro names.
- **Constants & Statics**: Use `SCREAMING_SNAKE_CASE` (e.g., `DEFAULT_TIMEOUT_SECS`) for constants and static variables.
- **Type Parameters**: Use `PascalCase` with single upper-case letters (e.g., `T`, `U`) or descriptive names (e.g., `Key`, `Value`).
- **Acronyms**: Treat acronyms as words in `PascalCase` (e.g., `XmlHttpRequest` instead of `XMLHTTPRequest`, `VertexAi` instead of `VertexAI`).

### Architecture & File Structure Rules
- **Design Principles**: Strictly apply **SOLID**, **DRY**, **KISS**, and **YAGNI** principles to all code modifications.
  - *Rust-Specific SOLID*: Emphasize Single Responsibility (small structs/functions) and Interface Segregation (small, focused traits). Prefer composition over inheritance patterns.
  - *KISS & YAGNI*: Avoid over-engineering. Do not introduce complex generic bounds, lifetimes, or custom macros unless absolutely necessary for the current requirement.
- **Strict File Length Limit**: No single source file (`.rs`) under the `./src` directory must exceed **300 lines of code** (including comments and blank lines).
- **Decomposition Strategy**: If a file is approaching or exceeds 300 lines, you MUST refactor it by:
  - Splitting implementation blocks (`impl`) into separate sub-modules or files.
  - Extracting helper functions, internal data structures, or trait implementations into dedicated sub-modules (e.g., creating a module folder with `mod.rs`).

### Formatting & Linting
- **Cargo Fmt**: You MUST run/ensure code is formatted using `cargo fmt` before proposing changes.
- **Clippy**: Run `cargo clippy` and resolve ALL warnings before finalizing code.
- **Allow Attributes**: Minimize the use of `#[allow(...)]`. If absolutely necessary, you must write an inline comment explaining why the lint warning is safely ignored.

### Error Handling
- **No Panic**: NEVER use `unwrap()` or `expect()` in production/library code. Use them only if an invariant guarantees failure is impossible, and document that invariant clearly.
- **Error Propagation**: Prefer returning a `Result<T, E>` and propagating errors using the `?` operator.
- **Custom Errors**: Use dedicated error types or crates (like `thiserror` or `anyhow`) to define domain-specific errors.

### Ownership, Borrowing, and Memory
- **Avoid Allocations**: Do not call `.clone()`, `.to_owned()`, or `.to_string()` unless it is strictly necessary to transfer ownership.
- **Signatures**: Prefer borrowing (`&str`, `&[T]`) over taking ownership of containers (`String`, `Vec<T>`) in function arguments.
- **Smart Pointers**: Use `Rc`, `Arc`, `RefCell`, `Mutex`, and `Box` only when necessary for shared ownership, interior mutability, or dynamic dispatch.

---

## 5. Commenting Guidelines
- **Doc Comments**: Use `///` for public-facing structs, enums, traits, and functions. Use `//!` for module-level documentation. Describe parameters, return values, errors, and panic conditions.
- **Inline Comments**: Use `//` placed ABOVE the lines of code they describe. NEVER write comments inline on the same line.
- **Focus on "Why"**: Comments must explain the reasoning behind complex logic, not just repeat what the code does. Refactor for readability if the code is too complex.
- **No Dead Code**: NEVER leave commented-out blocks of code. Delete them entirely (rely on Git for history).