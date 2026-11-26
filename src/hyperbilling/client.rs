use std::sync::Arc;
use std::time::Duration;

use reqwest::Method;
use reqwest::header::{CONTENT_TYPE, HeaderValue};
use serde_json::Value;
use tracing::debug;

use super::Error;
use crate::observability::Metrics;

pub const CLIENT_ID_HEADER: &str = "x-client-id";
pub const CLIENT_SECRET_HEADER: &str = "x-client-secret";

const MAX_RESPONSE_BYTES: usize = 8 << 20;
const MAX_ERROR_DETAIL_CHARS: usize = 500;
const PROBE_TIMEOUT: Duration = Duration::from_secs(5);

/// No Debug: the secret must never reach a log line.
#[derive(Clone)]
pub struct Credentials {
    client_id: String,
    client_secret: String,
}

impl Credentials {
    pub fn new(client_id: &str, client_secret: &str) -> Option<Self> {
        (!client_id.is_empty() && !client_secret.is_empty()).then(|| Self {
            client_id: client_id.to_string(),
            client_secret: client_secret.to_string(),
        })
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum Reply {
    Body(String),
    NoContent,
}

pub struct Client {
    base_url: String,
    fallback: Option<Credentials>,
    http: reqwest::Client,
    timeout: Duration,
    metrics: Arc<Metrics>,
}

impl Client {
    pub fn new(
        base_url: &str,
        fallback: Option<Credentials>,
        timeout: Duration,
        metrics: Arc<Metrics>,
    ) -> Result<Self, Error> {
        let http = reqwest::Client::builder()
            .timeout(timeout)
            .build()
            .map_err(Error::Request)?;
        Ok(Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            fallback,
            http,
            timeout,
            metrics,
        })
    }

    /// Sends no credentials, so a check spends none of a client's hourly quota.
    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    /// Returns the answered status code.
    pub async fn probe(&self) -> Result<u16, Error> {
        let response = self
            .http
            .get(&self.base_url)
            .timeout(PROBE_TIMEOUT.min(self.timeout))
            .send()
            .await
            .map_err(Error::Request)?;
        let status = response.status().as_u16();
        if status >= 500 {
            return Err(Error::Api {
                status,
                message: "server error".to_string(),
            });
        }
        Ok(status)
    }

    pub async fn get(&self, caller: Option<&Credentials>, path: &str) -> Result<Reply, Error> {
        self.send(caller, Method::GET, path, None).await
    }

    pub async fn post(
        &self,
        caller: Option<&Credentials>,
        path: &str,
        body: &Value,
    ) -> Result<Reply, Error> {
        self.send(caller, Method::POST, path, Some(body)).await
    }

    async fn send(
        &self,
        caller: Option<&Credentials>,
        method: Method,
        path: &str,
        body: Option<&Value>,
    ) -> Result<Reply, Error> {
        let credentials = caller
            .or(self.fallback.as_ref())
            .ok_or(Error::MissingCredentials)?;
        let client_id =
            HeaderValue::from_str(&credentials.client_id).map_err(|_| Error::InvalidCredentials)?;
        let mut client_secret = HeaderValue::from_str(&credentials.client_secret)
            .map_err(|_| Error::InvalidCredentials)?;
        client_secret.set_sensitive(true);

        let mut request = self
            .http
            .request(method.clone(), format!("{}{path}", self.base_url))
            .header(CONTENT_TYPE, "application/json")
            .header(CLIENT_ID_HEADER, client_id)
            .header(CLIENT_SECRET_HEADER, client_secret);
        if let Some(body) = body {
            request = request.body(body.to_string());
        }

        let response = match request.send().await {
            Ok(response) => response,
            Err(err) => {
                self.metrics
                    .upstream_request(method.as_str(), "transport_error");
                return Err(Error::Request(err));
            }
        };
        let status = response.status().as_u16();
        self.metrics
            .upstream_request(method.as_str(), &status.to_string());
        debug!(method = %method, path, status, "hyperbilling request");

        if status == 204 {
            return Ok(Reply::NoContent);
        }
        let text = read_limited(response).await.map_err(Error::Request)?;
        if !(200..300).contains(&status) {
            return Err(Error::Api {
                status,
                message: api_message(status, &text, &credentials.client_secret),
            });
        }
        Ok(Reply::Body(text.trim().to_string()))
    }
}

async fn read_limited(mut response: reqwest::Response) -> Result<String, reqwest::Error> {
    let mut data: Vec<u8> = Vec::new();
    while let Some(chunk) = response.chunk().await? {
        let room = MAX_RESPONSE_BYTES - data.len();
        if chunk.len() >= room {
            data.extend_from_slice(&chunk[..room]);
            break;
        }
        data.extend_from_slice(&chunk);
    }
    Ok(String::from_utf8_lossy(&data).into_owned())
}

fn api_message(status: u16, body: &str, secret: &str) -> String {
    let hint = match status {
        401 => {
            "credentials rejected (header missing, key expired, or client id does not own the key)"
        }
        403 => "authenticated, but no linked account is visible to this client id",
        429 => "hourly limit of 500 requests exceeded, retry later",
        _ => "",
    };
    // The upstream 401 message echoes the submitted key, so it is never forwarded.
    let detail = if status == 401 {
        String::new()
    } else {
        upstream_detail(body, secret)
    };
    match (hint.is_empty(), detail.is_empty()) {
        (true, true) => "no detail".to_string(),
        (false, true) => hint.to_string(),
        (true, false) => detail,
        (false, false) => format!("{hint}: {detail}"),
    }
}

fn upstream_detail(body: &str, secret: &str) -> String {
    let field = |value: &Value, key: &str| {
        value
            .get(key)
            .and_then(Value::as_str)
            .filter(|s| !s.is_empty())
            .map(str::to_string)
    };
    let detail = serde_json::from_str::<Value>(body)
        .ok()
        .and_then(|value| {
            let parts: Vec<String> = ["error", "message"]
                .iter()
                .filter_map(|key| field(&value, key))
                .collect();
            (!parts.is_empty()).then(|| parts.join(": "))
        })
        .unwrap_or_else(|| body.trim().to_string());
    let detail: String = detail.chars().take(MAX_ERROR_DETAIL_CHARS).collect();
    if secret.is_empty() {
        detail
    } else {
        detail.replace(secret, "[redacted]")
    }
}

#[cfg(test)]
pub mod tests {
    use std::sync::Mutex;

    use axum::Router;
    use axum::body::Bytes;
    use axum::http::{HeaderMap, Method as HttpMethod, StatusCode, Uri};

    use super::*;

    #[derive(Clone, Debug, Default)]
    pub struct Seen {
        pub method: String,
        pub path: String,
        pub client_id: String,
        pub client_secret: String,
        pub content_type: String,
        pub body: String,
    }

    pub fn install_crypto_provider() {
        let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();
    }

    /// Serves `status` and `body` on every path and records the last request.
    pub async fn spawn_upstream(
        status: StatusCode,
        body: &'static str,
    ) -> (String, Arc<Mutex<Seen>>) {
        let seen: Arc<Mutex<Seen>> = Arc::default();
        let app = Router::new().fallback({
            let seen = seen.clone();
            move |method: HttpMethod, uri: Uri, headers: HeaderMap, bytes: Bytes| {
                let seen = seen.clone();
                async move {
                    let header = |name: &str| {
                        headers
                            .get(name)
                            .and_then(|v| v.to_str().ok())
                            .unwrap_or_default()
                            .to_string()
                    };
                    *seen.lock().unwrap() = Seen {
                        method: method.to_string(),
                        path: uri.path().to_string(),
                        client_id: header(CLIENT_ID_HEADER),
                        client_secret: header(CLIENT_SECRET_HEADER),
                        content_type: header("content-type"),
                        body: String::from_utf8_lossy(&bytes).into_owned(),
                    };
                    (status, body)
                }
            }
        });
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(async move {
            let _ = axum::serve(listener, app).await;
        });
        (format!("http://{addr}"), seen)
    }

    pub fn client(base_url: &str, fallback: Option<Credentials>) -> Client {
        install_crypto_provider();
        Client::new(
            base_url,
            fallback,
            Duration::from_secs(5),
            Arc::new(Metrics::new("test", "test")),
        )
        .unwrap()
    }

    #[tokio::test]
    async fn sends_fallback_credentials_and_json_body() {
        let (url, seen) = spawn_upstream(StatusCode::OK, r#" {"Results":[]} "#).await;
        let client = client(&url, Credentials::new("svc@example.com", "svc-key"));

        let reply = client
            .post(None, "/v1/search/tag", &serde_json::json!({"Filter": {}}))
            .await
            .unwrap();

        assert_eq!(reply, Reply::Body(r#"{"Results":[]}"#.to_string()));
        let seen = seen.lock().unwrap().clone();
        assert_eq!(seen.method, "POST");
        assert_eq!(seen.path, "/v1/search/tag");
        assert_eq!(seen.client_id, "svc@example.com");
        assert_eq!(seen.client_secret, "svc-key");
        assert_eq!(seen.content_type, "application/json");
        assert_eq!(seen.body, r#"{"Filter":{}}"#);
    }

    #[tokio::test]
    async fn caller_credentials_win_over_fallback() {
        let (url, seen) = spawn_upstream(StatusCode::OK, "[]").await;
        let client = client(&url, Credentials::new("svc@example.com", "svc-key"));
        let caller = Credentials::new("caller@example.com", "caller-key");

        client
            .get(caller.as_ref(), "/v1/search/linkedaccount")
            .await
            .unwrap();

        let seen = seen.lock().unwrap().clone();
        assert_eq!(seen.method, "GET");
        assert_eq!(seen.client_id, "caller@example.com");
        assert_eq!(seen.client_secret, "caller-key");
    }

    #[tokio::test]
    async fn missing_and_invalid_credentials_never_reach_upstream() {
        let (url, seen) = spawn_upstream(StatusCode::OK, "[]").await;

        let err = client(&url, None).get(None, "/x").await.unwrap_err();
        assert!(matches!(err, Error::MissingCredentials));

        let bad = Credentials::new("user@example.com", "line\nbreak");
        let err = client(&url, None)
            .get(bad.as_ref(), "/x")
            .await
            .unwrap_err();
        assert!(matches!(err, Error::InvalidCredentials));

        assert_eq!(seen.lock().unwrap().path, "");
    }

    #[tokio::test]
    async fn no_content_is_a_reply_not_an_error() {
        let (url, _) = spawn_upstream(StatusCode::NO_CONTENT, "").await;
        let client = client(&url, Credentials::new("svc@example.com", "svc-key"));
        assert_eq!(client.get(None, "/x").await.unwrap(), Reply::NoContent);
    }

    #[tokio::test]
    async fn unauthorized_body_is_dropped() {
        let (url, _) = spawn_upstream(
            StatusCode::UNAUTHORIZED,
            r#"{"status":401,"error":"Unauthorized","message":"invalid key svc-key"}"#,
        )
        .await;
        let client = client(&url, Credentials::new("svc@example.com", "svc-key"));

        let err = client.get(None, "/x").await.unwrap_err();
        let text = err.to_string();
        assert!(text.contains("401"));
        assert!(text.contains("credentials rejected"));
        assert!(!text.contains("svc-key"));
    }

    #[tokio::test]
    async fn transport_failure_is_a_request_error() {
        let client = client("http://127.0.0.1:1", Credentials::new("a@example.com", "k"));
        let err = client.get(None, "/x").await.unwrap_err();
        assert!(matches!(err, Error::Request(_)));
    }

    #[tokio::test]
    async fn probe_passes_on_any_answer_below_500_without_credentials() {
        for status in [
            StatusCode::OK,
            StatusCode::UNAUTHORIZED,
            StatusCode::NOT_FOUND,
        ] {
            let (url, seen) = spawn_upstream(status, "").await;
            let client = client(&url, Credentials::new("svc@example.com", "svc-key"));
            assert_eq!(client.probe().await.unwrap(), status.as_u16());
            let seen = seen.lock().unwrap().clone();
            assert_eq!(seen.method, "GET");
            assert_eq!(seen.client_id, "");
            assert_eq!(seen.client_secret, "");
        }
    }

    #[tokio::test]
    async fn probe_fails_on_server_error_and_transport_failure() {
        let (url, _) = spawn_upstream(StatusCode::BAD_GATEWAY, "").await;
        let err = client(&url, None).probe().await.unwrap_err();
        assert!(matches!(err, Error::Api { status: 502, .. }));

        let err = client("http://127.0.0.1:1", None)
            .probe()
            .await
            .unwrap_err();
        assert!(matches!(err, Error::Request(_)));
    }

    #[test]
    fn api_message_formats_and_redacts() {
        assert_eq!(
            api_message(
                400,
                r#"{"error":"Bad Request","message":"GroupBy max 5"}"#,
                "k"
            ),
            "Bad Request: GroupBy max 5"
        );
        assert_eq!(
            api_message(500, "boom secret-key boom", "secret-key"),
            "boom [redacted] boom"
        );
        assert_eq!(api_message(404, "", "k"), "no detail");
        assert_eq!(
            api_message(429, "", "k"),
            "hourly limit of 500 requests exceeded, retry later"
        );
        assert!(api_message(403, r#"{"message":"denied"}"#, "k").ends_with(": denied"));
        assert_eq!(api_message(502, &"x".repeat(900), "").len(), 500);
    }
}
