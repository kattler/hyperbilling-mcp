use std::time::Duration;

use prometheus_client::encoding::EncodeLabelSet;
use prometheus_client::encoding::text::encode;
use prometheus_client::metrics::counter::Counter;
use prometheus_client::metrics::family::Family;
use prometheus_client::metrics::histogram::Histogram;
use prometheus_client::metrics::info::Info;
use prometheus_client::registry::Registry;

pub const OUTCOME_OK: &str = "ok";
pub const OUTCOME_TOOL_ERROR: &str = "tool_error";
pub const OUTCOME_ERROR: &str = "error";

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
pub struct ToolOutcomeLabels {
    pub tool: String,
    pub outcome: String,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
pub struct ToolLabels {
    pub tool: String,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
pub struct UpstreamLabels {
    pub method: String,
    pub code: String,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
pub struct BuildInfoLabels {
    pub version: String,
    pub commit: String,
}

pub struct Metrics {
    registry: Registry,
    tool_calls: Family<ToolOutcomeLabels, Counter>,
    tool_duration: Family<ToolLabels, Histogram>,
    upstream_requests: Family<UpstreamLabels, Counter>,
}

fn duration_histogram() -> Histogram {
    Histogram::new([0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0, 30.0, 60.0])
}

impl Metrics {
    pub fn new(version: &str, commit: &str) -> Self {
        let mut registry = Registry::with_prefix("hyperbilling_mcp");
        let tool_calls = Family::<ToolOutcomeLabels, Counter>::default();
        let tool_duration =
            Family::<ToolLabels, Histogram>::new_with_constructor(duration_histogram);
        let upstream_requests = Family::<UpstreamLabels, Counter>::default();

        registry.register(
            "build",
            "Build version and commit of the running binary",
            Info::new(BuildInfoLabels {
                version: version.to_string(),
                commit: commit.to_string(),
            }),
        );
        registry.register(
            "tool_calls",
            "MCP tool calls by tool and outcome (ok, tool_error, error)",
            tool_calls.clone(),
        );
        registry.register(
            "tool_call_duration_seconds",
            "MCP tool call latency by tool, including the HyperBilling API request",
            tool_duration.clone(),
        );
        registry.register(
            "upstream_requests",
            "Requests sent to the HyperBilling API by method and status code (transport_error when no response arrived)",
            upstream_requests.clone(),
        );

        Self {
            registry,
            tool_calls,
            tool_duration,
            upstream_requests,
        }
    }

    pub fn tool_call(&self, tool: &str, outcome: &str, elapsed: Duration) {
        self.tool_calls
            .get_or_create(&ToolOutcomeLabels {
                tool: tool.to_string(),
                outcome: outcome.to_string(),
            })
            .inc();
        self.tool_duration
            .get_or_create(&ToolLabels {
                tool: tool.to_string(),
            })
            .observe(elapsed.as_secs_f64());
    }

    pub fn upstream_request(&self, method: &str, code: &str) {
        self.upstream_requests
            .get_or_create(&UpstreamLabels {
                method: method.to_string(),
                code: code.to_string(),
            })
            .inc();
    }

    pub fn encode(&self) -> Result<String, std::fmt::Error> {
        let mut out = String::new();
        encode(&mut out, &self.registry)?;
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encodes_series_with_prefix() {
        let metrics = Metrics::new("1.2.3", "abc");
        metrics.tool_call(
            "hyperbilling_search_billing",
            OUTCOME_OK,
            Duration::from_millis(20),
        );
        metrics.upstream_request("POST", "200");

        let out = metrics.encode().unwrap();
        assert!(out.contains("hyperbilling_mcp_build_info{version=\"1.2.3\",commit=\"abc\"} 1"));
        assert!(out.contains(
            "hyperbilling_mcp_tool_calls_total{tool=\"hyperbilling_search_billing\",outcome=\"ok\"} 1"
        ));
        assert!(out.contains(
            "hyperbilling_mcp_tool_call_duration_seconds_count{tool=\"hyperbilling_search_billing\"} 1"
        ));
        assert!(
            out.contains(
                "hyperbilling_mcp_upstream_requests_total{method=\"POST\",code=\"200\"} 1"
            )
        );
    }
}
