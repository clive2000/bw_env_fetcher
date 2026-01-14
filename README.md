# bw_env_fetcher

A CLI tool that fetches secrets from [Bitwarden Secrets Manager](https://bitwarden.com/products/secrets-manager/) and outputs them in `.env` format. It retrieves the Bitwarden access token from Google Cloud Secret Manager, making it ideal for CI/CD pipelines and automated deployments.

## Installation

### Quick Install (macOS/Linux)

```bash
curl -fsSL https://raw.githubusercontent.com/clive2000/bw_env_fetcher/main/install.sh | bash
```

This installs to `~/.local/bin`. If needed, add it to your PATH:

```bash
# bash
echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.bashrc

# zsh
echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.zshrc

# fish
fish_add_path $HOME/.local/bin
```

### From Source

```bash
cargo install --git https://github.com/clive2000/bw_env_fetcher
```

### Download Binary

Download the latest release from [Releases](https://github.com/clive2000/bw_env_fetcher/releases).

## Prerequisites

1. **GCP Authentication**: Configure Application Default Credentials
   ```bash
   gcloud auth application-default login
   ```

2. **Bitwarden Access Token**: Store your Bitwarden Secrets Manager access token in GCP Secret Manager
   ```bash
   echo -n "0.your-access-token:key" | gcloud secrets create BITWARDEN_SM_API_TOKEN --data-file=-
   ```

3. **Bitwarden Project ID**: Find it in Bitwarden web vault under Secrets Manager → Projects

## Usage

### Basic Usage (stdout)

```bash
bw_env_fetcher \
  --gcp-project my-gcp-project \
  --gcp-secret-name BITWARDEN_SM_API_TOKEN \
  --project-id 00000000-0000-0000-0000-000000000000
```

Output:
```
DATABASE_URL=postgres://localhost:5432/mydb
API_KEY=secret123
```

### Write to File

```bash
bw_env_fetcher \
  --gcp-project my-gcp-project \
  --gcp-secret-name BITWARDEN_SM_API_TOKEN \
  --project-id 00000000-0000-0000-0000-000000000000 \
  --output .env
```

### Using Environment Variables

```bash
export GCP_PROJECT_ID=my-gcp-project
export GCP_SECRET_NAME=BITWARDEN_SM_API_TOKEN
export BW_PROJECT_ID=00000000-0000-0000-0000-000000000000

bw_env_fetcher
```

### Pipe to Another Command

```bash
bw_env_fetcher --gcp-project my-project --gcp-secret-name TOKEN --project-id UUID | source /dev/stdin
```

## Options

| Option | Env Variable | Description |
|--------|--------------|-------------|
| `--gcp-project` | `GCP_PROJECT_ID` | GCP project ID containing the Bitwarden access token |
| `--gcp-secret-name` | `GCP_SECRET_NAME` | Name of the secret in GCP Secret Manager |
| `--project-id` | `BW_PROJECT_ID` | Bitwarden project ID to fetch secrets from |
| `-o, --output` | - | Write to file instead of stdout |
| `-v, --verbose` | - | Increase verbosity (-v, -vv, -vvv) |
| `--bw-identity-url` | `BW_IDENTITY_URL` | Bitwarden identity server URL (default: https://identity.bitwarden.com) |
| `--bw-api-url` | `BW_API_URL` | Bitwarden API server URL (default: https://api.bitwarden.com) |

## Use Cases

### CI/CD Pipeline

```yaml
# GitHub Actions example
- name: Fetch secrets
  run: |
    bw_env_fetcher \
      --gcp-project ${{ vars.GCP_PROJECT }} \
      --gcp-secret-name BITWARDEN_TOKEN \
      --project-id ${{ vars.BW_PROJECT_ID }} \
      --output .env
```

### Docker

```dockerfile
RUN curl -fsSL https://raw.githubusercontent.com/clive2000/bw_env_fetcher/main/install.sh | bash
```

### Self-hosted Bitwarden

```bash
bw_env_fetcher \
  --gcp-project my-project \
  --gcp-secret-name TOKEN \
  --project-id UUID \
  --bw-identity-url https://identity.mybitwarden.com \
  --bw-api-url https://api.mybitwarden.com
```

## How It Works

1. Fetches Bitwarden access token from GCP Secret Manager
2. Authenticates with Bitwarden Secrets Manager
3. Retrieves all secrets from the specified project
4. Outputs secrets in `.env` format (stdout or file)

## License

MIT
