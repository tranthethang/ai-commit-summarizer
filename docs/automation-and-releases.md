# CI/CD Automation and Releases

This document describes the automated build and release pipeline for `asum`. The automation is built using **GitHub Actions**.

## Overview

The repository uses a rolling release strategy. Every time new code is pushed or merged into the `main` branch, a GitHub Actions workflow is triggered. This workflow cross-compiles the Rust project for multiple platforms and publishes the resulting binaries to a `latest` release tag on GitHub Releases.

## Supported Platforms

Currently, the pipeline compiles binaries for:
- `x86_64-unknown-linux-gnu` (Linux x86_64)
- `x86_64-apple-darwin` (macOS Intel)
- `aarch64-apple-darwin` (macOS Apple Silicon)

## Workflow Details (`release.yml`)

1. **Trigger:** The workflow runs on the `push` event for the `main` branch.
2. **Build Matrix:** A matrix strategy is used to run parallel jobs on `ubuntu-latest` and `macos-latest` runners, targeting the required Rust architectures.
3. **Compilation:** `cargo build --release` is executed.
4. **Packaging:** The compiled binaries are packed into `.tar.gz` archives (e.g., `asum-x86_64-unknown-linux-gnu.tar.gz`).
5. **Publishing:** 
   - An intermediate job collects all the artifacts from the matrix build.
   - Using the GitHub CLI (`gh`), it deletes the existing `latest` tag and release.
   - It uses `softprops/action-gh-release` to create a new `latest` release and attaches the newly compiled `.tar.gz` files as assets.

## Installation Scripts

The `install.sh` script provides a user-friendly mechanism for downloading and installing the pre-compiled binary.
- It detects the user's OS (`uname -s`) and architecture (`uname -m`).
- It constructs the URL to the corresponding asset in the `latest` GitHub release.
- It uses `curl` to download and extract the tarball.
- Finally, it copies the binary to `/usr/local/bin` (using `sudo`).

The `uninstall.sh` simply removes the binary from `/usr/local/bin`.

## Maintenance

If you need to support additional platforms (e.g., Windows), you can:
1. Update the `matrix` under `jobs.build.strategy` in `.github/workflows/release.yml` with the new OS runner and Rust target.
2. Ensure the packaging steps handle `.exe` files or different archive formats if required (e.g., `.zip` for Windows).
3. Update `install.sh`'s OS detection logic if the new platform uses a bash-compatible shell. For Windows native, consider adding a `.ps1` installation script instead.
