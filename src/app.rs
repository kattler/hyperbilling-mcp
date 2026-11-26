use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use axum::Router;
use clap::Parser;
use rmcp::transport::StreamableHttpService;
use rmcp::transport::streamable_http_server::StreamableHttpServerConfig;
use rmcp::transport::streamable_http_server::session::local::LocalSessionManager;
use tokio::net::TcpListener;
use tokio_util::sync::CancellationToken;
use tracing::{info, warn};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{EnvFilter, fmt};

use crate::config::Config;
use crate::docs;
use crate::hyperbilling::Client;
use crate::hyperbilling::Error;
use crate::mcp::Server;
use crate::observability::server::{health_router, metrics_router, serve};
use crate::observability::{Metrics, Readiness, UpstreamHealth};

const STARTUP_FAILURE_LOG_EVERY: u32 = 15;
const STARTUP_CHECK_INTERVAL: Duration = Duration::from_secs(2);
const UPSTREAM_CHECK_INTERVAL: Duration = Duration::from_secs(15);

pub async fn start() -> Result<()> {
    let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();
    let cfg = Config::parse();
    init_tracing(&cfg.log_level, &cfg.log_format);
    cfg.validate().context("configuration")?;

    let shutdown = CancellationToken::new();
    tokio::spawn({
        let shutdown = shutdown.clone();
        async move {
            wait_for_signal().await;
            info!("shutdown signal received");
            shutdown.cancel();
        }
    });

    let listener = bind(cfg.listen_port).await?;
    let metrics_listener = bind(cfg.metrics_port).await?;
    run(&cfg, listener, metrics_listener, shutdown).await
}

async fn bind(port: u16) -> Result<TcpListener> {
    TcpListener::bind(("0.0.0.0", port))
        .await
        .with_context(|| format!("bind port {port}"))
}

async fn run(
    cfg: &Config,
    listener: TcpListener,
    metrics_listener: TcpListener,
    shutdown: CancellationToken,
) -> Result<()> {
    let metrics = Arc::new(Metrics::new(
        env!("CARGO_PKG_VERSION"),
        env!("BUILD_COMMIT"),
    ));
    let fallback = cfg.fallback_credentials();
    info!(
        version = env!("CARGO_PKG_VERSION"),
        commit = env!("BUILD_COMMIT"),
        built = env!("BUILD_DATE"),
        hyperbilling_url = cfg.hyperbilling_url,
        fallback_credentials = fallback.is_some(),
        listen_port = cfg.listen_port,
        metrics_port = cfg.metrics_port,
        docs = cfg.docs_enabled,
        "starting hyperbilling-mcp"
    );
    let client = Client::new(
        &cfg.hyperbilling_url,
        fallback,
        cfg.request_timeout(),
        metrics.clone(),
    )
    .context("build HyperBilling client")?;
    let client = Arc::new(client);
    let readiness = Readiness::default();
    let health = Arc::new(UpstreamHealth::default());
    tokio::spawn(watch_upstream(
        client.clone(),
        readiness.clone(),
        health.clone(),
        metrics.clone(),
        STARTUP_CHECK_INTERVAL,
        UPSTREAM_CHECK_INTERVAL,
        shutdown.clone(),
    ));
    let server = Server::new(client, metrics.clone());

    let mut app = Router::new();
    if cfg.docs_enabled {
        let pages = docs::Pages::render(
            env!("CARGO_PKG_VERSION"),
            env!("BUILD_DATE"),
            &server.tool_docs(),
        );
        app = app.merge(docs::router(pages, health));
    }
    let app = app.merge(mcp_router(server, &shutdown));
    tokio::try_join!(
        serve(listener, app, shutdown.clone()),
        serve(
            metrics_listener,
            health_router(readiness).merge(metrics_router(metrics)),
            shutdown
        ),
    )
    .context("http server")?;
    Ok(())
}

async fn watch_upstream(
    client: Arc<Client>,
    readiness: Readiness,
    health: Arc<UpstreamHealth>,
    metrics: Arc<Metrics>,
    startup_interval: Duration,
    interval: Duration,
    shutdown: CancellationToken,
) {
    let url = client.base_url();
    let started = Instant::now();
    let mut attempts: u32 = 0;
    let mut last = None;
    info!(
        url,
        retry_seconds = startup_interval.as_secs_f64(),
        "not ready, waiting for the first answer from the HyperBilling API"
    );
    loop {
        let sent = Instant::now();
        let result = tokio::select! {
            () = shutdown.cancelled() => return,
            result = client.probe() => result,
        };
        let reachable = result.is_ok();
        let status = match &result {
            Ok(status) | Err(Error::Api { status, .. }) => Some(*status),
            Err(_) => None,
        };
        health.record(reachable, status, sent.elapsed());
        metrics.upstream_reachable(reachable);
        match result {
            Ok(status) if readiness.mark_ready() => info!(
                url,
                status,
                attempts = attempts + 1,
                waited_seconds = started.elapsed().as_secs_f64(),
                "HyperBilling API reachable, ready until the process exits"
            ),
            Ok(status) if last == Some(false) => {
                info!(url, status, "HyperBilling API reachable again");
            }
            Err(err) if !readiness.is_ready() => {
                attempts += 1;
                if attempts == 1 || attempts.is_multiple_of(STARTUP_FAILURE_LOG_EVERY) {
                    warn!(
                        url,
                        error = %err,
                        attempts,
                        waited_seconds = started.elapsed().as_secs_f64(),
                        "HyperBilling API unreachable, staying not ready"
                    );
                }
            }
            Err(err) if last != Some(false) => warn!(
                url,
                error = %err,
                "HyperBilling API unreachable, staying ready and failing tool calls"
            ),
            Ok(_) | Err(_) => {}
        }
        last = Some(reachable);

        let delay = if readiness.is_ready() {
            interval
        } else {
            startup_interval
        };
        tokio::select! {
            () = shutdown.cancelled() => return,
            () = tokio::time::sleep(delay) => {}
        }
    }
}

fn mcp_router(server: Arc<Server>, shutdown: &CancellationToken) -> Router {
    let mut config = StreamableHttpServerConfig::default().disable_allowed_hosts();
    config.cancellation_token = shutdown.child_token();
    let service = StreamableHttpService::new(
        move || Ok(server.clone()),
        Arc::new(LocalSessionManager::default()),
        config,
    );
    Router::new().route_service("/mcp", service)
}

fn init_tracing(level: &str, format: &str) {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(level));
    let registry = tracing_subscriber::registry().with(filter);
    if format.eq_ignore_ascii_case("text") {
        registry.with(fmt::layer().with_target(false)).init();
    } else {
        registry
            .with(fmt::layer().json().flatten_event(true))
            .init();
    }
}

async fn wait_for_signal() {
    #[cfg(unix)]
    {
        use tokio::signal::unix::{SignalKind, signal};
        let mut term = signal(SignalKind::terminate()).expect("install SIGTERM handler");
        tokio::select! {
            _ = tokio::signal::ctrl_c() => {}
            _ = term.recv() => {}
        }
    }
    #[cfg(not(unix))]
    {
        let _ = tokio::signal::ctrl_c().await;
    }
}

#[cfg(test)]
mod tests {
    use axum::http::StatusCode;
    use serde_json::{Value, json};

    use super::*;
    use crate::hyperbilling::client::tests::{install_crypto_provider, spawn_upstream};

    const ACCEPT: &str = "application/json, text/event-stream";

    /// The JSON-RPC message of a streamable HTTP reply, which is either plain
    /// JSON or one SSE `data:` line.
    fn rpc(body: &str) -> Value {
        let payload = body
            .lines()
            .filter_map(|line| line.strip_prefix("data:"))
            .map(str::trim)
            .find(|data| data.starts_with('{'))
            .unwrap_or(body);
        serde_json::from_str(payload).unwrap_or_else(|_| panic!("not JSON-RPC: {body}"))
    }

    #[tokio::test]
    async fn serves_tools_over_streamable_http_until_shutdown() {
        install_crypto_provider();
        let (upstream, seen) = spawn_upstream(StatusCode::OK, r#"["123456789012"]"#).await;
        let cfg = Config::parse_from(["hyperbilling-mcp", "--hyperbilling-url", &upstream]);
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let metrics_listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let base = format!("http://{}", listener.local_addr().unwrap());
        let metrics_base = format!("http://{}", metrics_listener.local_addr().unwrap());
        let shutdown = CancellationToken::new();
        let handle = tokio::spawn({
            let shutdown = shutdown.clone();
            async move { run(&cfg, listener, metrics_listener, shutdown).await }
        });

        let http = reqwest::Client::new();
        let post = |session: Option<String>, body: Value| {
            let mut request = http
                .post(format!("{base}/mcp"))
                .header("accept", ACCEPT)
                .header("content-type", "application/json")
                .header("x-client-id", "caller@example.com")
                .header("x-client-secret", "caller-key")
                .body(body.to_string());
            if let Some(session) = session {
                request = request.header("mcp-session-id", session);
            }
            request.send()
        };

        let response = post(
            None,
            json!({"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {
                "protocolVersion": "2025-03-26",
                "capabilities": {},
                "clientInfo": {"name": "test", "version": "0"}
            }}),
        )
        .await
        .unwrap();
        assert_eq!(response.status(), 200);
        let session = response
            .headers()
            .get("mcp-session-id")
            .map(|v| v.to_str().unwrap().to_string());
        let init = rpc(&response.text().await.unwrap());
        assert_eq!(init["result"]["serverInfo"]["name"], "hyperbilling-mcp");

        post(
            session.clone(),
            json!({"jsonrpc": "2.0", "method": "notifications/initialized"}),
        )
        .await
        .unwrap();

        let response = post(
            session.clone(),
            json!({"jsonrpc": "2.0", "id": 2, "method": "tools/call", "params": {
                "name": "hyperbilling_list_linked_accounts", "arguments": {}
            }}),
        )
        .await
        .unwrap();
        let called = rpc(&response.text().await.unwrap());
        assert_eq!(
            called["result"]["content"][0]["text"],
            r#"["123456789012"]"#
        );
        let seen = seen.lock().unwrap().clone();
        assert_eq!(seen.client_id, "caller@example.com");
        assert_eq!(seen.client_secret, "caller-key");

        wait_for_status(&format!("{metrics_base}/readyz"), 200).await;
        for path in ["/healthz", "/readyz"] {
            let internal = reqwest::get(format!("{metrics_base}{path}")).await.unwrap();
            assert_eq!(internal.status(), 200, "{path}");
            let routed = reqwest::get(format!("{base}{path}")).await.unwrap();
            assert_eq!(routed.status(), 404, "{path} stays off the routed port");
        }
        let metrics = reqwest::get(format!("{metrics_base}/metrics"))
            .await
            .unwrap()
            .text()
            .await
            .unwrap();
        assert!(
            metrics.contains(
                "hyperbilling_mcp_upstream_requests_total{method=\"GET\",code=\"200\"} 1"
            )
        );
        assert!(metrics.contains("hyperbilling_mcp_upstream_reachable 1"));

        shutdown.cancel();
        handle.await.unwrap().unwrap();
    }

    async fn wait_for_status(url: &str, status: u16) {
        for _ in 0..200 {
            if reqwest::get(url).await.unwrap().status() == status {
                return;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
        panic!("{url} never answered {status}");
    }

    #[tokio::test]
    async fn readiness_latches_after_the_first_reachable_check() {
        use std::sync::atomic::{AtomicBool, Ordering};

        install_crypto_provider();
        let upstream = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let url = format!("http://{}", upstream.local_addr().unwrap());
        let down = Arc::new(AtomicBool::new(true));
        tokio::spawn({
            let down = down.clone();
            let app = Router::new().fallback(move || {
                let down = down.clone();
                async move {
                    if down.load(Ordering::Relaxed) {
                        StatusCode::SERVICE_UNAVAILABLE
                    } else {
                        StatusCode::TOO_MANY_REQUESTS
                    }
                }
            });
            async move { axum::serve(upstream, app).await }
        });

        let metrics = Arc::new(Metrics::new("test", "test"));
        let client =
            Arc::new(Client::new(&url, None, Duration::from_secs(5), metrics.clone()).unwrap());
        let readiness = Readiness::default();
        let health = Arc::new(UpstreamHealth::default());
        let shutdown = CancellationToken::new();
        let handle = tokio::spawn(watch_upstream(
            client,
            readiness.clone(),
            health.clone(),
            metrics.clone(),
            Duration::from_millis(10),
            Duration::from_millis(10),
            shutdown.clone(),
        ));
        let wait_for_gauge = |value: u8| {
            let metrics = metrics.clone();
            let series = format!("hyperbilling_mcp_upstream_reachable {value}");
            async move {
                for _ in 0..200 {
                    if metrics.encode().unwrap().contains(&series) {
                        return;
                    }
                    tokio::time::sleep(Duration::from_millis(10)).await;
                }
                panic!("never saw {series}");
            }
        };

        tokio::time::sleep(Duration::from_millis(50)).await;
        assert!(!readiness.is_ready());

        down.store(false, Ordering::Relaxed);
        wait_for_gauge(1).await;
        assert!(readiness.is_ready());

        down.store(true, Ordering::Relaxed);
        wait_for_gauge(0).await;
        tokio::time::sleep(Duration::from_millis(50)).await;
        assert!(readiness.is_ready());

        let snapshot = health.snapshot();
        assert!(snapshot.ready_since.is_some());
        assert_eq!(snapshot.checks.first().unwrap().status, Some(503));
        assert!(
            snapshot
                .checks
                .iter()
                .any(|check| check.status == Some(429))
        );
        assert!(!snapshot.checks.last().unwrap().reachable);

        shutdown.cancel();
        handle.await.unwrap();
    }

    type Running = (
        String,
        CancellationToken,
        tokio::task::JoinHandle<Result<()>>,
    );

    async fn spawn(flags: &[&str]) -> Running {
        install_crypto_provider();
        let cfg = Config::parse_from(
            [
                "hyperbilling-mcp",
                "--hyperbilling-url",
                "http://127.0.0.1:1",
            ]
            .iter()
            .chain(flags),
        );
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let metrics_listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let base = format!("http://{}", listener.local_addr().unwrap());
        let shutdown = CancellationToken::new();
        let handle = tokio::spawn({
            let shutdown = shutdown.clone();
            async move { run(&cfg, listener, metrics_listener, shutdown).await }
        });
        (base, shutdown, handle)
    }

    #[tokio::test]
    async fn serves_docs_in_both_locales() {
        let (base, shutdown, handle) = spawn(&[]).await;

        let docs = reqwest::get(format!("{base}/")).await.unwrap();
        assert_eq!(docs.url().path(), "/docs");
        assert_eq!(docs.status(), 200);
        assert!(
            docs.headers()["content-type"]
                .to_str()
                .unwrap()
                .starts_with("text/html")
        );
        assert!(docs.headers().contains_key("content-security-policy"));
        assert_eq!(docs.headers()["content-language"], "en");
        let docs = docs.text().await.unwrap();
        assert!(docs.contains(r#""name":"hyperbilling_get_recent_billing""#));
        assert!(docs.contains("latest-by-region"));
        assert!(!docs.contains("{{"));

        let korean = reqwest::get(format!("{base}/docs?lang=ko")).await.unwrap();
        assert_eq!(korean.headers()["content-language"], "ko");
        assert!(korean.text().await.unwrap().contains(r#""locale":"ko""#));
        for (path, content_type, text) in [
            (
                "/docs/app.js",
                "text/javascript; charset=utf-8",
                "사용 제한",
            ),
            (
                "/docs/app.css",
                "text/css; charset=utf-8",
                "prefers-color-scheme",
            ),
        ] {
            let asset = reqwest::get(format!("{base}{path}")).await.unwrap();
            assert_eq!(asset.status(), 200);
            assert_eq!(asset.headers()["content-type"], content_type);
            assert!(asset.text().await.unwrap().contains(text));
        }
        let api = reqwest::get(format!("{base}/docs/api?lang=ko"))
            .await
            .unwrap();
        assert_eq!(api.status(), 200);
        assert_eq!(api.headers()["content-language"], "ko");
        assert!(api.text().await.unwrap().contains(r#""page":"api""#));

        let status = reqwest::get(format!("{base}/docs/status")).await.unwrap();
        assert_eq!(status.status(), 200);
        assert_eq!(status.headers()["cache-control"], "no-store");
        let status: Value = serde_json::from_str(&status.text().await.unwrap()).unwrap();
        assert!(status["ready_since"].is_null());
        assert!(status["checks"].is_array());

        let negotiated = reqwest::Client::new()
            .get(format!("{base}/docs"))
            .header("accept-language", "ko-KR,ko;q=0.9,en;q=0.8")
            .send()
            .await
            .unwrap();
        assert_eq!(negotiated.headers()["content-language"], "ko");
        assert_eq!(negotiated.headers()["vary"], "Accept-Language");

        shutdown.cancel();
        handle.await.unwrap().unwrap();
    }

    #[tokio::test]
    async fn docs_can_be_disabled() {
        let (base, shutdown, handle) = spawn(&["--docs-enabled", "false"]).await;

        for path in ["/", "/docs", "/docs/api", "/docs/status", "/docs/app.js"] {
            let response = reqwest::get(format!("{base}{path}")).await.unwrap();
            assert_eq!(response.status(), 404, "{path}");
        }

        shutdown.cancel();
        handle.await.unwrap().unwrap();
    }
}
