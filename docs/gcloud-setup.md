# Google Cloud CLI (gcloud) Setup Guide

This guide explains how to install and authenticate the Google Cloud CLI (`gcloud`) on your system to use the Google Vertex AI provider with `asum`.

## Installation

You can install the Google Cloud CLI using Homebrew (recommended for macOS) or the official interactive installer.

### Method 1: Using Homebrew (macOS)

1. Update Homebrew and install the `gcloud-cli` cask:
   ```bash
   brew update && brew install --cask gcloud-cli
   ```

2. Add the `gcloud` binaries to your shell's search path. Open your shell profile file (e.g., `~/.zshrc` or `~/.bash_profile`) and add the following configuration:
   ```bash
   export PATH="$(brew --prefix)/share/google-cloud-sdk/bin:$PATH"
   ```

3. Reload your shell configuration:
   ```bash
   source ~/.zshrc
   ```

### Method 2: Official Interactive Installer

If you are not using Homebrew, or if you are on a different operating system:

1. Determine your processor type (e.g., Apple Silicon or Intel for macOS).
2. Download the appropriate archive from the official [Google Cloud SDK Install Page](https://cloud.google.com/sdk/docs/install).
3. Extract the downloaded archive to a directory of your choice (e.g., your home directory).
4. Run the installation script in your terminal:
   ```bash
   ./google-cloud-sdk/install.sh
   ```
5. Follow the interactive prompts to configure your path and enable shell command completion. Restart your terminal after completion.

---

## Verification

To verify that the Google Cloud CLI is successfully installed, run:

```bash
gcloud --version
```

This command should output the installed version details of the SDK.

---

## Authentication for Vertex AI

To allow `asum` to query Google Vertex AI models, you need to authenticate your local environment.

### 1. Initialize and Log In

Log in to your Google Account associated with your GCP project:

```bash
gcloud auth login
```

This will open a browser window asking you to authorize the Google Cloud SDK.

### 2. Configure Your Default Project

Set the default Google Cloud project that contains your Vertex AI resources:

```bash
gcloud config set project YOUR_GCP_PROJECT_ID
```

Replace `YOUR_GCP_PROJECT_ID` with your actual GCP project ID.

### 3. Authenticate Application Default Credentials (ADC)

For applications like `asum` to make API calls using your credentials, authenticate Application Default Credentials (ADC):

```bash
gcloud auth application-default login
```

This is the most critical step to ensure that `gcloud auth print-access-token` works seamlessly behind the scenes.
