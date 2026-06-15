# Google Vertex AI Provider

Google Cloud Vertex AI is an enterprise platform for building machine learning applications. It allows you to use Gemini models securely within your GCP organization.

## Requirements

1. A Google Cloud Platform (GCP) Project with the **Vertex AI API** enabled.
2. The `gcloud` CLI tool installed and authenticated (`gcloud auth login`).

## Configuration

Update your `asum.toml` file to use Vertex AI:

```toml
[general]
active_provider = "vertexai"

[vertexai]
project_id = "your-gcp-project-id"
location = "us-central1"
model = "gemini-1.5-flash"
```

### Authentication

By default, if you do not provide an `access_token` in the config, `asum` will automatically run `gcloud auth print-access-token` behind the scenes to authenticate the request.

If you are running `asum` in an environment without `gcloud` (like a CI/CD pipeline), you can explicitly provide a static token:

```toml
[vertexai]
project_id = "your-gcp-project-id"
location = "us-central1"
model = "gemini-1.5-flash"
access_token = "ya29..."
```

## Custom URL

If you need to route requests through an enterprise gateway, you can override the default Vertex AI URL:

```toml
[vertexai]
project_id = "your-gcp-project-id"
location = "us-central1"
model = "gemini-1.5-flash"
url = "https://your-custom-proxy.com/v1/projects/...:generateContent"
```
