use crate::error::{AppError, Result};
use bitwarden::{
    auth::login::AccessTokenLoginRequest,
    secrets_manager::{
        secrets::{SecretIdentifiersByProjectRequest, SecretsGetRequest},
        ClientSecretsExt,
    },
    Client, ClientSettings, DeviceType,
};
use tracing::{debug, info};
use uuid::Uuid;

pub struct Secret {
    pub key: String,
    pub value: String,
}

pub struct BitwardenClient {
    client: Client,
}

impl BitwardenClient {
    pub fn new(identity_url: &str, api_url: &str) -> Self {
        let settings = ClientSettings {
            identity_url: identity_url.to_string(),
            api_url: api_url.to_string(),
            user_agent: "bw_env_fetcher".to_string(),
            device_type: DeviceType::SDK,
        };

        debug!(
            "Creating Bitwarden client with identity_url: {}, api_url: {}",
            identity_url, api_url
        );

        Self {
            client: Client::new(Some(settings)),
        }
    }

    pub async fn authenticate(&mut self, access_token: &str) -> Result<()> {
        debug!("Authenticating with Bitwarden using access token");

        let token_request = AccessTokenLoginRequest {
            access_token: access_token.to_string(),
            state_file: None,
        };

        self.client
            .auth()
            .login_access_token(&token_request)
            .await
            .map_err(|e| AppError::BitwardenAuth(format!("Authentication failed: {}", e)))?;

        info!("Successfully authenticated with Bitwarden");

        Ok(())
    }

    pub async fn fetch_secrets_by_project(&self, project_id: &str) -> Result<Vec<Secret>> {
        let project_uuid = Uuid::parse_str(project_id)?;

        debug!("Fetching secrets for project: {}", project_id);

        let request = SecretIdentifiersByProjectRequest {
            project_id: project_uuid,
        };

        let identifiers = self
            .client
            .secrets()
            .list_by_project(&request)
            .await
            .map_err(|e| AppError::BitwardenApi(format!("Failed to list secrets: {}", e)))?;

        if identifiers.data.is_empty() {
            info!("No secrets found in project");
            return Ok(Vec::new());
        }

        let secret_ids: Vec<Uuid> = identifiers.data.iter().map(|s| s.id).collect();
        debug!(
            "Found {} secret identifiers, fetching values",
            secret_ids.len()
        );

        let secrets_request = SecretsGetRequest { ids: secret_ids };

        let secrets_response = self
            .client
            .secrets()
            .get_by_ids(secrets_request)
            .await
            .map_err(|e| AppError::BitwardenApi(format!("Failed to fetch secrets: {}", e)))?;

        let secrets: Vec<Secret> = secrets_response
            .data
            .into_iter()
            .map(|s| Secret {
                key: s.key,
                value: s.value,
            })
            .collect();

        info!(
            "Successfully fetched {} secrets from Bitwarden",
            secrets.len()
        );

        Ok(secrets)
    }
}
