use std::sync::Arc;

use axum::Router;
use axum::extract::State;
use axum::http::{StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use tokio::net::TcpListener;
use tokio_util::sync::CancellationToken;

use super::{Metrics, Readiness};

pub fn health_router(readiness: Readiness) -> Router {
    Router::new()
        .route("/healthz", get(ok))
        .route("/readyz", get(ready))
        .with_state(readiness)
}

async fn ok() -> StatusCode {
    StatusCode::OK
}

async fn ready(State(readiness): State<Readiness>) -> (StatusCode, &'static str) {
    if readiness.is_ready() {
        (StatusCode::OK, "ok")
    } else {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            "HyperBilling API has not been reached yet",
        )
    }
}

pub fn metrics_router(metrics: Arc<Metrics>) -> Router {
    Router::new()
        .route("/metrics", get(serve_metrics))
        .with_state(metrics)
}

async fn serve_metrics(State(metrics): State<Arc<Metrics>>) -> Response {
    match metrics.encode() {
        Ok(body) => (
            [(
                header::CONTENT_TYPE,
                "application/openmetrics-text; version=1.0.0; charset=utf-8",
            )],
            body,
        )
            .into_response(),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("encode metrics: {err}"),
        )
            .into_response(),
    }
}

pub async fn serve(
    listener: TcpListener,
    router: Router,
    shutdown: CancellationToken,
) -> std::io::Result<()> {
    axum::serve(listener, router)
        .with_graceful_shutdown(async move { shutdown.cancelled().await })
        .await
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;
    use crate::observability::metrics::OUTCOME_OK;

    #[tokio::test]
    async fn serves_health_and_metrics_until_shutdown() {
        let metrics = Arc::new(Metrics::new("test", "test"));
        metrics.tool_call("hyperbilling_search_billing", OUTCOME_OK, Duration::ZERO);
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let base = format!("http://{}", listener.local_addr().unwrap());
        let shutdown = CancellationToken::new();
        let readiness = Readiness::default();
        let handle = tokio::spawn(serve(
            listener,
            health_router(readiness.clone()).merge(metrics_router(metrics)),
            shutdown.clone(),
        ));
        let status = |path: &'static str| {
            let url = format!("{base}{path}");
            async move { reqwest::get(url).await.unwrap().status() }
        };

        assert_eq!(status("/healthz").await, 200);
        assert_eq!(status("/readyz").await, 503);
        readiness.mark_ready();
        assert_eq!(status("/readyz").await, 200);
        assert_eq!(status("/healthz").await, 200);
        let body = reqwest::get(format!("{base}/metrics"))
            .await
            .unwrap()
            .text()
            .await
            .unwrap();
        assert!(body.contains("hyperbilling_mcp_tool_calls_total"));

        shutdown.cancel();
        handle.await.unwrap().unwrap();
    }
}
