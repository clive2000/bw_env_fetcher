use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "bw_env_fetcher")]
#[command(
    author,
    version,
    about = "Fetch secrets from Bitwarden Secrets Manager and output as .env format"
)]
#[command(
    long_about = "This tool fetches a Bitwarden access token from Google Cloud Secret Manager, \
    then uses it to authenticate with Bitwarden Secrets Manager and retrieve all secrets \
    for a given project. By default, secrets are printed to stdout. Use --output to write to a file."
)]
pub struct Config {
    #[arg(
        long,
        env = "GCP_PROJECT_ID",
        help = "GCP project ID containing the Bitwarden access token secret"
    )]
    pub gcp_project: String,

    #[arg(
        long,
        env = "GCP_SECRET_NAME",
        help = "Name of the secret in GCP Secret Manager that contains the Bitwarden access token"
    )]
    pub gcp_secret_name: String,

    #[arg(
        long,
        env = "BW_PROJECT_ID",
        help = "Bitwarden project ID to fetch secrets from"
    )]
    pub project_id: String,

    #[arg(
        short,
        long,
        help = "Write to file instead of stdout (overwrites if exists)"
    )]
    pub output: Option<PathBuf>,

    #[arg(short, long, action = clap::ArgAction::Count, help = "Increase verbosity (-v, -vv, -vvv)")]
    pub verbose: u8,

    #[arg(
        long,
        env = "BW_IDENTITY_URL",
        default_value = "https://identity.bitwarden.com",
        help = "Bitwarden identity server URL"
    )]
    pub bw_identity_url: String,

    #[arg(
        long,
        env = "BW_API_URL",
        default_value = "https://api.bitwarden.com",
        help = "Bitwarden API server URL"
    )]
    pub bw_api_url: String,
}

impl Config {
    pub fn parse_args() -> Self {
        Config::parse()
    }
}
