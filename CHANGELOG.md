# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- CI workflow targeting GitHub Actions to automatically build, lint, and test the codebase.
- CONTRIBUTING.md outlining development guidelines and code conventions.
- Support for CLI argument parsing and validation using the clap crate.
- Enum representation for Provider and DiffReductionMode config fields to enforce type-safety.
- Timeout limits (120s) for outbound HTTP request clients to avoid infinite hangs.

### Changed
- Moved Gemini API authentication key from URL query parameters to x-goog-api-key HTTP header.
- Extracted duplicate response-post-processing and verbose logging logic into a shared helper module.
- Restructured AsumConfig with a ProviderConfig enum to remove optional provider fields.
- Replaced unwrap_or_default calls in provider instantiation with explicit configuration checks.

### Fixed
- Fixed typo in installation script.
- Added exit status checks on git command invocations.
- Refactored tests mutating global environment variables to prevent race conditions.
