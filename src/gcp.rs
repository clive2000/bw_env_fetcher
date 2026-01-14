use crate::error::{AppError, Result};
use base64::Engine;
use reqwest::Client;
use serde::Deserialize;
use std::path::PathBuf;
use std::time::Duration;
use tracing::{debug, info, warn};

const GCP_TIMEOUT_SECS: u64 = 30;
const SECRET_MANAGER_BASE_URL: &str = "https://secretmanager.googleapis.com/v1";

#[derive(Deserialize)]
struct AdcCredentials {
    client_id: String,
    client_secret: String,
    refresh_token: String,
}

#[derive(Deserialize)]
struct TokenResponse {
    access_token: String,
}

#[derive(Deserialize)]
struct SecretPayload {
    data: String,
}

#[derive(Deserialize)]
struct SecretAccessResponse {
    payload: SecretPayload,
}

pub struct GcpSecretClient {
    client: Client,
    project_id: String,
    access_token: String,
}

impl GcpSecretClient {
    pub async fn new(project_id: String) -> Result<Self> {
        debug!(
            "Initializing GCP Secret Manager client for project: {}",
            project_id
        );

        let client = Client::builder()
            .timeout(Duration::from_secs(GCP_TIMEOUT_SECS))
            .build()
            .map_err(|e| {
                AppError::GcpSecretManager(format!("Failed to create HTTP client: {}", e))
            })?;

        let access_token = Self::get_access_token(&client).await?;

        info!("GCP Secret Manager client initialized successfully");

        Ok(Self {
            client,
            project_id,
            access_token,
        })
    }

    pub async fn get_secret(&self, secret_name: &str) -> Result<String> {
        let url = format!(
            "{}/projects/{}/secrets/{}/versions/latest:access",
            SECRET_MANAGER_BASE_URL, self.project_id, secret_name
        );

        debug!("Fetching secret from: {}", url);

        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.access_token))
            .send()
            .await
            .map_err(|e| AppError::GcpSecretManager(format!("HTTP request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(AppError::GcpSecretManager(format!(
                "API returned {}: {}",
                status, body
            )));
        }

        let secret_response: SecretAccessResponse = response
            .json()
            .await
            .map_err(|e| AppError::GcpSecretManager(format!("Failed to parse response: {}", e)))?;

        let decoded = base64::engine::general_purpose::STANDARD
            .decode(&secret_response.payload.data)
            .map_err(|e| AppError::GcpSecretManager(format!("Failed to decode secret: {}", e)))?;

        let secret_value = String::from_utf8(decoded)?;

        info!("Successfully retrieved secret: {}", secret_name);

        Ok(secret_value)
    }

    async fn get_access_token(client: &Client) -> Result<String> {
        let adc_path = Self::find_adc_path()?;
        debug!("Using ADC from: {:?}", adc_path);

        let adc_content = std::fs::read_to_string(&adc_path).map_err(|e| {
            AppError::GcpSecretManager(format!(
                "Failed to read ADC file at {:?}: {}. Run: gcloud auth application-default login",
                adc_path, e
            ))
        })?;

        let credentials: AdcCredentials = serde_json::from_str(&adc_content)
            .map_err(|e| AppError::GcpSecretManager(format!("Failed to parse ADC file: {}", e)))?;

        let token_response = client
            .post("https://oauth2.googleapis.com/token")
            .form(&[
                ("client_id", credentials.client_id.as_str()),
                ("client_secret", credentials.client_secret.as_str()),
                ("refresh_token", credentials.refresh_token.as_str()),
                ("grant_type", "refresh_token"),
            ])
            .send()
            .await
            .map_err(|e| AppError::GcpSecretManager(format!("Token refresh failed: {}", e)))?;

        if !token_response.status().is_success() {
            let status = token_response.status();
            let body = token_response.text().await.unwrap_or_default();
            return Err(AppError::GcpSecretManager(format!(
                "Token refresh returned {}: {}",
                status, body
            )));
        }

        let token: TokenResponse = token_response.json().await.map_err(|e| {
            AppError::GcpSecretManager(format!("Failed to parse token response: {}", e))
        })?;

        debug!("Successfully obtained access token");
        Ok(token.access_token)
    }

    fn find_adc_path() -> Result<PathBuf> {
        if let Ok(path) = std::env::var("GOOGLE_APPLICATION_CREDENTIALS") {
            let path = PathBuf::from(path);
            if path.exists() {
                return Ok(path);
            }
            warn!(
                "GOOGLE_APPLICATION_CREDENTIALS points to non-existent file: {:?}",
                path
            );
        }

        let default_path = dirs_next::home_dir()
            .map(|h| h.join(".config/gcloud/application_default_credentials.json"))
            .ok_or_else(|| {
                AppError::GcpSecretManager("Could not determine home directory".to_string())
            })?;

        if default_path.exists() {
            return Ok(default_path);
        }

        Err(AppError::GcpSecretManager(
            "No GCP credentials found. Run: gcloud auth application-default login".to_string(),
        ))
    }
}
