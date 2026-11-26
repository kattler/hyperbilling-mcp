#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(
        "no HyperBilling credentials: send the X-Client-Id and X-Client-Secret headers, or configure the server fallback"
    )]
    MissingCredentials,
    #[error("a HyperBilling credential is not a valid HTTP header value")]
    InvalidCredentials,
    #[error("HyperBilling API returned {status}: {message}")]
    Api { status: u16, message: String },
    #[error("HyperBilling API request failed: {0}")]
    Request(#[source] reqwest::Error),
}
