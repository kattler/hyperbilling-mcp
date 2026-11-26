use std::time::Duration;

use clap::Parser;

use crate::error::ConfigError;
use crate::hyperbilling::Credentials;

#[derive(Clone, Parser)]
#[command(name = "hyperbilling-mcp", version, about)]
pub struct Config {
    /// Port for the MCP endpoint, /healthz, and /readyz
    #[arg(long, env = "LISTEN_PORT", default_value_t = 8080)]
    pub listen_port: u16,

    /// Port for /metrics
    #[arg(long, env = "METRICS_PORT", default_value_t = 8081)]
    pub metrics_port: u16,

    /// HyperBilling Open API base URL
    #[arg(
        long,
        env = "HYPERBILLING_URL",
        default_value = "https://open.hyperapi.megazone.com"
    )]
    pub hyperbilling_url: String,

    /// Fallback X-Client-Id (HyperBilling login email) for callers that send no credential headers
    #[arg(long, env = "HYPERBILLING_CLIENT_ID", default_value = "")]
    pub hyperbilling_client_id: String,

    /// Fallback X-Client-Secret (Open API key) for callers that send no credential headers
    #[arg(
        long,
        env = "HYPERBILLING_CLIENT_SECRET",
        default_value = "",
        hide_env_values = true,
        hide_default_value = true
    )]
    pub hyperbilling_client_secret: String,

    /// Timeout of one HyperBilling API request in seconds
    #[arg(
        long,
        env = "HYPERBILLING_REQUEST_TIMEOUT_SECONDS",
        default_value_t = 60
    )]
    pub request_timeout_seconds: u64,

    /// Serve the documentation page at /docs
    #[arg(
        long,
        env = "DOCS_ENABLED",
        default_value_t = true,
        action = clap::ArgAction::Set
    )]
    pub docs_enabled: bool,

    /// Log level: trace, debug, info, warn, error
    #[arg(long, env = "LOG_LEVEL", default_value = "info")]
    pub log_level: String,

    /// Log format: json or text
    #[arg(long, env = "LOG_FORMAT", default_value = "json")]
    pub log_format: String,
}

impl Config {
    pub fn validate(&self) -> Result<(), ConfigError> {
        if !self.hyperbilling_url.starts_with("http://")
            && !self.hyperbilling_url.starts_with("https://")
        {
            return Err(ConfigError::InvalidUrl);
        }
        if self.hyperbilling_client_id.is_empty() != self.hyperbilling_client_secret.is_empty() {
            return Err(ConfigError::PartialCredentials);
        }
        if self.request_timeout_seconds == 0 {
            return Err(ConfigError::ZeroTimeout);
        }
        Ok(())
    }

    pub fn fallback_credentials(&self) -> Option<Credentials> {
        Credentials::new(
            &self.hyperbilling_client_id,
            &self.hyperbilling_client_secret,
        )
    }

    pub const fn request_timeout(&self) -> Duration {
        Duration::from_secs(self.request_timeout_seconds)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(flags: &[&str]) -> Config {
        Config::parse_from(std::iter::once(&"hyperbilling-mcp").chain(flags))
    }

    #[test]
    fn defaults_are_valid_without_credentials() {
        let cfg = parse(&[]);
        assert_eq!(cfg.validate(), Ok(()));
        assert_eq!(cfg.listen_port, 8080);
        assert_eq!(cfg.metrics_port, 8081);
        assert_eq!(cfg.request_timeout(), Duration::from_secs(60));
        assert!(cfg.fallback_credentials().is_none());
        assert!(cfg.docs_enabled);
        assert!(!parse(&["--docs-enabled", "false"]).docs_enabled);
    }

    #[test]
    fn fallback_credentials_need_both_values() {
        let cfg = parse(&["--hyperbilling-client-id", "user@example.com"]);
        assert_eq!(cfg.validate(), Err(ConfigError::PartialCredentials));

        let cfg = parse(&[
            "--hyperbilling-client-id",
            "user@example.com",
            "--hyperbilling-client-secret",
            "k",
        ]);
        assert_eq!(cfg.validate(), Ok(()));
        assert!(cfg.fallback_credentials().is_some());
    }

    #[test]
    fn rejects_bad_url_and_zero_timeout() {
        let cfg = parse(&["--hyperbilling-url", "open.example.com"]);
        assert_eq!(cfg.validate(), Err(ConfigError::InvalidUrl));

        let cfg = parse(&["--request-timeout-seconds", "0"]);
        assert_eq!(cfg.validate(), Err(ConfigError::ZeroTimeout));
    }
}
