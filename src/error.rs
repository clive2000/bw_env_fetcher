use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("GCP Secret Manager error: {0}")]
    GcpSecretManager(String),

    #[error("Bitwarden authentication failed: {0}")]
    BitwardenAuth(String),

    #[error("Bitwarden API error: {0}")]
    BitwardenApi(String),

    #[error("Invalid configuration: {0}")]
    #[allow(dead_code)]
    Config(String),

    #[error("File operation failed: {0}")]
    #[allow(dead_code)]
    FileOperation(String),

    #[error("Secret not found: {0}")]
    #[allow(dead_code)]
    SecretNotFound(String),

    #[error("Invalid UUID: {0}")]
    InvalidUuid(#[from] uuid::Error),

    #[error("UTF-8 conversion error: {0}")]
    Utf8Error(#[from] std::string::FromUtf8Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, AppError>;
