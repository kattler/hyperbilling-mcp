#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum ConfigError {
    #[error("HYPERBILLING_URL must start with http:// or https://")]
    InvalidUrl,
    #[error("HYPERBILLING_CLIENT_ID and HYPERBILLING_CLIENT_SECRET must be set together")]
    PartialCredentials,
    #[error("HYPERBILLING_REQUEST_TIMEOUT_SECONDS must be greater than 0")]
    ZeroTimeout,
}
