mod bitwarden;
mod config;
mod env_writer;
mod error;
mod gcp;

use crate::bitwarden::BitwardenClient;
use crate::config::Config;
use crate::env_writer::EnvWriter;
use crate::error::AppError;
use crate::gcp::GcpSecretClient;
use colored::Colorize;
use tracing::{error, info, Level};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() {
    let config = Config::parse_args();

    init_logging(config.verbose);

    if let Err(e) = run(config).await {
        print_error(&e);
        std::process::exit(1);
    }
}

async fn run(config: Config) -> Result<(), AppError> {
    info!("Starting bw_env_fetcher");

    eprintln!(
        "{} Fetching Bitwarden access token from GCP Secret Manager...",
        "→".blue()
    );

    let gcp_client = GcpSecretClient::new(config.gcp_project.clone()).await?;
    let access_token = gcp_client.get_secret(&config.gcp_secret_name).await?;

    eprintln!(
        "{} Successfully retrieved access token from GCP",
        "✓".green()
    );

    eprintln!("{} Authenticating with Bitwarden...", "→".blue());

    let mut bw_client = BitwardenClient::new(&config.bw_identity_url, &config.bw_api_url);
    bw_client.authenticate(&access_token).await?;

    eprintln!("{} Successfully authenticated with Bitwarden", "✓".green());

    eprintln!(
        "{} Fetching secrets for project {}...",
        "→".blue(),
        &config.project_id
    );

    let secrets = bw_client
        .fetch_secrets_by_project(&config.project_id)
        .await?;

    if secrets.is_empty() {
        eprintln!("{} No secrets found in the project", "!".yellow());
        return Ok(());
    }

    eprintln!("{} Found {} secrets", "✓".green(), secrets.len());

    match &config.output {
        Some(path) => {
            eprintln!("{} Writing secrets to {}...", "→".blue(), path.display());
            EnvWriter::write_to_file(&secrets, path)?;
            eprintln!(
                "{} Successfully wrote {} environment variables to {}",
                "✓".green(),
                secrets.len(),
                path.display()
            );
        }
        None => {
            EnvWriter::write_to_stdout(&secrets)?;
        }
    }

    Ok(())
}

fn init_logging(verbosity: u8) {
    let level = match verbosity {
        0 => Level::WARN,
        1 => Level::INFO,
        2 => Level::DEBUG,
        _ => Level::TRACE,
    };

    let filter = EnvFilter::from_default_env().add_directive(level.into());

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .with_writer(std::io::stderr)
        .init();
}

fn print_error(error: &AppError) {
    error!("{}", error);
    eprintln!("{} {}", "Error:".red().bold(), error);

    match error {
        AppError::GcpSecretManager(_) => {
            eprintln!(
                "\n{}: Make sure you have authenticated with GCP:",
                "Hint".yellow()
            );
            eprintln!("  gcloud auth application-default login");
            eprintln!("  # or set GOOGLE_APPLICATION_CREDENTIALS");
        }
        AppError::BitwardenAuth(_) => {
            eprintln!(
                "\n{}: Make sure you're using a valid Machine Account access token.",
                "Hint".yellow()
            );
            eprintln!("  Generate one at: Organization Settings → Machine Accounts");
        }
        AppError::Config(msg) if msg.contains("project") => {
            eprintln!(
                "\n{}: The project ID should be a valid UUID.",
                "Hint".yellow()
            );
            eprintln!("  Find it in Bitwarden: Secrets Manager → Projects");
        }
        _ => {}
    }
}
