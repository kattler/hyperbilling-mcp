use std::sync::Arc;
use std::time::Instant;

use rmcp::handler::server::tool::{parse_json_object, schema_for_input};
use rmcp::model::{
    CallToolRequestParams, CallToolResponse, CallToolResult, ContentBlock, ErrorData,
    Implementation, JsonObject, ListToolsResult, PaginatedRequestParams, ProtocolVersion,
    ServerCapabilities, ServerConfig, Tool, ToolAnnotations,
};
use rmcp::service::RequestContext;
use rmcp::{RoleServer, ServerHandler};
use schemars::JsonSchema;
use serde_json::Value;

use super::args::{LinkedAccountsArgs, NoArgs, RecentBillingArgs, SearchBillingArgs};
use super::coerce::{coerce_arguments, strip_defaults};
use crate::hyperbilling::client::{CLIENT_ID_HEADER, CLIENT_SECRET_HEADER};
use crate::hyperbilling::{Client, Credentials, Error, Reply};
use crate::observability::Metrics;
use crate::observability::metrics::{OUTCOME_ERROR, OUTCOME_OK, OUTCOME_TOOL_ERROR};

const INSTRUCTIONS: &str = "Read-only AWS billing data from Megazone HyperBilling. Call hyperbilling_list_linked_accounts first to learn the account IDs, hyperbilling_list_tag_keys before grouping by a tag, then hyperbilling_search_billing for a custom query or hyperbilling_get_recent_billing for a fixed recent report. A search returns at most 1000 rows: pass a non-empty response NextDataToken back as next_data_token until it comes back empty. The API allows 500 requests per hour.";

type ToolResult = Result<CallToolResult, ErrorData>;

pub struct ToolDoc {
    pub name: String,
    pub description: String,
    pub upstream: &'static str,
    pub schema: Arc<JsonObject>,
}

pub struct Server {
    client: Arc<Client>,
    metrics: Arc<Metrics>,
    tools: Vec<Tool>,
}

impl Server {
    pub fn new(client: Arc<Client>, metrics: Arc<Metrics>) -> Arc<Self> {
        let mut tools = Self::tool_definitions();
        tools.sort_by(|a, b| a.name.cmp(&b.name));
        Arc::new(Self {
            client,
            metrics,
            tools,
        })
    }

    pub fn tool_docs(&self) -> Vec<ToolDoc> {
        self.tools
            .iter()
            .map(|tool| ToolDoc {
                name: tool.name.to_string(),
                description: tool.description.as_deref().unwrap_or_default().to_string(),
                upstream: Self::upstream(&tool.name),
                schema: tool.input_schema.clone(),
            })
            .collect()
    }

    fn tool(&self, name: &str) -> Option<&Tool> {
        self.tools.iter().find(|tool| tool.name == name)
    }

    pub async fn call(
        &self,
        caller: Option<&Credentials>,
        name: &str,
        arguments: Option<JsonObject>,
    ) -> ToolResult {
        let start = Instant::now();
        let result = self
            .dispatch(caller, name, arguments.unwrap_or_default())
            .await;
        let outcome = match &result {
            Err(_) => OUTCOME_ERROR,
            Ok(r) if r.is_error == Some(true) => OUTCOME_TOOL_ERROR,
            Ok(_) => OUTCOME_OK,
        };
        // An unknown name comes from the caller, so it must not become a label value.
        let label = self.tool(name).map_or("unknown", |tool| tool.name.as_ref());
        self.metrics.tool_call(label, outcome, start.elapsed());
        result
    }

    async fn list_linked_accounts(
        &self,
        caller: Option<&Credentials>,
        _args: NoArgs,
    ) -> ToolResult {
        reply(self.client.get(caller, "/v1/search/linkedaccount").await)
    }

    async fn list_tag_keys(
        &self,
        caller: Option<&Credentials>,
        args: LinkedAccountsArgs,
    ) -> ToolResult {
        let body = args.into_tag_body();
        reply(self.client.post(caller, "/v1/search/tag", &body).await)
    }

    async fn search_billing(
        &self,
        caller: Option<&Credentials>,
        args: SearchBillingArgs,
    ) -> ToolResult {
        let body = args.into_body();
        reply(self.client.post(caller, "/v1/search/billing", &body).await)
    }

    async fn get_recent_billing(
        &self,
        caller: Option<&Credentials>,
        args: RecentBillingArgs,
    ) -> ToolResult {
        reply(
            self.client
                .post(caller, args.view.path(), &args.body())
                .await,
        )
    }
}

/// API rejections are tool errors the model can read and correct. Only a
/// transport failure fails the MCP call.
fn reply(result: Result<Reply, Error>) -> ToolResult {
    match result {
        Ok(Reply::Body(text)) if !text.is_empty() => {
            Ok(CallToolResult::success(vec![ContentBlock::text(text)]))
        }
        Ok(_) => Ok(CallToolResult::success(vec![ContentBlock::text(
            "no billing data matches this query",
        )])),
        Err(err @ Error::Request(_)) => Err(ErrorData::internal_error(err.to_string(), None)),
        Err(err) => Ok(CallToolResult::error(vec![ContentBlock::text(
            err.to_string(),
        )])),
    }
}

fn input_schema<T: JsonSchema + std::any::Any>() -> Arc<JsonObject> {
    let schema = schema_for_input::<T>().expect("tool input schema must have root type object");
    let mut value = Value::Object(schema.as_ref().clone());
    strip_defaults(&mut value);
    match value {
        Value::Object(object) => Arc::new(object),
        _ => schema,
    }
}

macro_rules! tool_surface {
    ($( $name:literal, $args:ty, $handler:ident, $upstream:literal, $desc:literal; )*) => {
        impl Server {
            fn upstream(name: &str) -> &'static str {
                match name {
                    $( $name => $upstream, )*
                    _ => "",
                }
            }

            fn tool_definitions() -> Vec<Tool> {
                vec![$(
                    Tool::new($name, $desc, input_schema::<$args>())
                        .annotate(ToolAnnotations::new().read_only(true))
                ),*]
            }

            async fn dispatch(
                &self,
                caller: Option<&Credentials>,
                name: &str,
                mut arguments: JsonObject,
            ) -> ToolResult {
                if let Some(tool) = self.tool(name) {
                    coerce_arguments(tool.input_schema.as_ref(), &mut arguments);
                }
                match name {
                    $( $name => self.$handler(caller, parse_json_object::<$args>(arguments)?).await, )*
                    _ => Err(ErrorData::invalid_params("tool not found", None)),
                }
            }
        }
    };
}

tool_surface! {
    "hyperbilling_list_linked_accounts", NoArgs, list_linked_accounts, "GET /v1/search/linkedaccount",
        "List the AWS linked accounts whose billing data the caller can query. Call this first: every other tool needs linked account IDs.";
    "hyperbilling_list_tag_keys", LinkedAccountsArgs, list_tag_keys, "POST /v1/search/tag",
        "List the cost allocation tag keys of each given linked account (user:Team, aws:eks:namespace). Prefix a key with TAG: to group by it in hyperbilling_search_billing.";
    "hyperbilling_search_billing", SearchBillingArgs, search_billing, "POST /v1/search/billing",
        "Query AWS billing data with a custom period, grouping and measures. Returns Results rows (GroupBy values and their Value) and, when more than 1000 rows match, a NextDataToken to pass back as next_data_token.";
    "hyperbilling_get_recent_billing", RecentBillingArgs, get_recent_billing, "POST /v2/search/billing/{view}",
        "Get a fixed recent billing report for the given linked accounts, where the view sets the period and the grouping. Cheaper to call than hyperbilling_search_billing when one of the views fits.";
}

fn caller_credentials(context: &RequestContext<RoleServer>) -> Option<Credentials> {
    let parts = context.extensions.get::<http::request::Parts>()?;
    credentials_from_headers(&parts.headers)
}

fn credentials_from_headers(headers: &http::HeaderMap) -> Option<Credentials> {
    let value = |name: &str| headers.get(name).and_then(|v| v.to_str().ok());
    Credentials::new(value(CLIENT_ID_HEADER)?, value(CLIENT_SECRET_HEADER)?)
}

impl ServerHandler for Server {
    fn get_info(&self) -> ServerConfig {
        let mut server_info = Implementation::new("hyperbilling-mcp", env!("CARGO_PKG_VERSION"));
        server_info.title = Some("Megazone HyperBilling".to_string());
        let mut info = ServerConfig::new(ServerCapabilities::builder().enable_tools().build());
        info.protocol_version = ProtocolVersion::default();
        info.server_info = server_info;
        info.instructions = Some(INSTRUCTIONS.to_string());
        info
    }

    fn list_tools(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> impl Future<Output = Result<ListToolsResult, ErrorData>> + Send + '_ {
        std::future::ready(Ok(ListToolsResult {
            tools: self.tools.clone(),
            ..Default::default()
        }))
    }

    fn get_tool(&self, name: &str) -> Option<Tool> {
        self.tool(name).cloned()
    }

    async fn call_tool(
        &self,
        request: CallToolRequestParams,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResponse, ErrorData> {
        let caller = caller_credentials(&context);
        self.call(caller.as_ref(), &request.name, request.arguments)
            .await
            .map(CallToolResponse::from)
    }
}

#[cfg(test)]
mod tests {
    use axum::http::StatusCode;
    use serde_json::json;

    use super::*;
    use crate::hyperbilling::client::tests::{client, spawn_upstream};

    fn server(url: &str) -> (Arc<Server>, Arc<Metrics>) {
        let metrics = Arc::new(Metrics::new("test", "test"));
        let client = client(url, Credentials::new("svc@example.com", "svc-key"));
        (Server::new(Arc::new(client), metrics.clone()), metrics)
    }

    fn arguments(value: Value) -> JsonObject {
        serde_json::from_value(value).unwrap()
    }

    fn text_of(result: &CallToolResult) -> String {
        match result.content.first() {
            Some(ContentBlock::Text(text)) => text.text.clone(),
            other => panic!("expected text content, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn tools_are_prefixed_described_and_sorted() {
        let (server, _) = server("http://127.0.0.1:1");
        let names: Vec<&str> = server.tools.iter().map(|t| t.name.as_ref()).collect();
        assert_eq!(
            names,
            [
                "hyperbilling_get_recent_billing",
                "hyperbilling_list_linked_accounts",
                "hyperbilling_list_tag_keys",
                "hyperbilling_search_billing",
            ]
        );
        for tool in &server.tools {
            assert!(tool.description.as_ref().is_some_and(|d| !d.is_empty()));
            assert_eq!(
                tool.annotations.as_ref().and_then(|a| a.read_only_hint),
                Some(true)
            );
            let schema = serde_json::to_string(tool.input_schema.as_ref()).unwrap();
            assert!(!schema.contains("$ref"), "{} schema uses $ref", tool.name);
        }
        assert!(server.get_tool("hyperbilling_search_billing").is_some());
        let docs = server.tool_docs();
        assert_eq!(docs.len(), 4);
        assert_eq!(docs[0].upstream, "POST /v2/search/billing/{view}");
        assert!(docs.iter().all(|doc| !doc.upstream.is_empty()));
        assert_eq!(Server::upstream("no_such_tool"), "");
        assert!(server.get_info().instructions.is_some());
    }

    #[tokio::test]
    async fn linked_accounts_is_a_get() {
        let (url, seen) = spawn_upstream(StatusCode::OK, r#"["123456789012"]"#).await;
        let (server, metrics) = server(&url);

        let result = server
            .call(None, "hyperbilling_list_linked_accounts", None)
            .await
            .unwrap();

        assert_ne!(result.is_error, Some(true));
        assert_eq!(text_of(&result), r#"["123456789012"]"#);
        let seen = seen.lock().unwrap().clone();
        assert_eq!(seen.method, "GET");
        assert_eq!(seen.path, "/v1/search/linkedaccount");
        assert_eq!(seen.client_id, "svc@example.com");
        assert!(metrics.encode().unwrap().contains(
            "tool_calls_total{tool=\"hyperbilling_list_linked_accounts\",outcome=\"ok\"} 1"
        ));
    }

    #[tokio::test]
    async fn search_billing_posts_the_filter_with_stringified_arguments() {
        let (url, seen) = spawn_upstream(StatusCode::OK, r#"{"Results":[]}"#).await;
        let (server, _) = server(&url);

        let result = server
            .call(
                None,
                "hyperbilling_search_billing",
                Some(arguments(json!({
                    "linked_accounts": "123456789012",
                    "granularity": "MONTHLY",
                    "start": "2026-09-01",
                    "end": "2026-09-30",
                    "group_by": "[\"SERVICE_CODE\"]",
                    "result": ["USAGE_COST"]
                }))),
            )
            .await
            .unwrap();

        assert_ne!(result.is_error, Some(true));
        let seen = seen.lock().unwrap().clone();
        assert_eq!(seen.method, "POST");
        assert_eq!(seen.path, "/v1/search/billing");
        assert_eq!(
            serde_json::from_str::<Value>(&seen.body).unwrap(),
            json!({
                "Filter": {
                    "LinkedAccount": ["123456789012"],
                    "Granularity": "MONTHLY",
                    "TimePeriod": {"Start": "2026-09-01", "End": "2026-09-30"}
                },
                "GroupBy": ["SERVICE_CODE"],
                "Result": ["USAGE_COST"]
            })
        );
    }

    #[tokio::test]
    async fn tag_keys_and_recent_billing_post_the_account_filter() {
        let (url, seen) = spawn_upstream(StatusCode::OK, "{}").await;
        let (server, _) = server(&url);
        let caller = Credentials::new("caller@example.com", "caller-key");

        server
            .call(
                caller.as_ref(),
                "hyperbilling_list_tag_keys",
                Some(arguments(json!({"linked_accounts": ["123456789012"]}))),
            )
            .await
            .unwrap();
        let tag = seen.lock().unwrap().clone();
        assert_eq!(tag.path, "/v1/search/tag");
        assert_eq!(tag.client_id, "caller@example.com");
        assert_eq!(
            serde_json::from_str::<Value>(&tag.body).unwrap(),
            json!({"LinkedAccount": ["123456789012"]})
        );

        server
            .call(
                None,
                "hyperbilling_get_recent_billing",
                Some(arguments(
                    json!({"linked_accounts": ["123456789012"], "view": "latest-by-region"}),
                )),
            )
            .await
            .unwrap();
        let recent = seen.lock().unwrap().clone();
        assert_eq!(recent.path, "/v2/search/billing/latest/region");
        assert_eq!(
            serde_json::from_str::<Value>(&recent.body).unwrap(),
            json!({"Filter": {"LinkedAccount": ["123456789012"]}})
        );
    }

    #[tokio::test]
    async fn no_content_reads_as_an_empty_result() {
        let (url, _) = spawn_upstream(StatusCode::NO_CONTENT, "").await;
        let (server, _) = server(&url);
        let result = server
            .call(None, "hyperbilling_list_linked_accounts", None)
            .await
            .unwrap();
        assert_ne!(result.is_error, Some(true));
        assert_eq!(text_of(&result), "no billing data matches this query");
    }

    #[tokio::test]
    async fn api_rejection_is_a_tool_error_without_the_key() {
        let (url, _) = spawn_upstream(
            StatusCode::UNAUTHORIZED,
            r#"{"error":"Unauthorized","message":"bad key svc-key"}"#,
        )
        .await;
        let (server, metrics) = server(&url);

        let result = server
            .call(None, "hyperbilling_list_linked_accounts", None)
            .await
            .unwrap();

        assert_eq!(result.is_error, Some(true));
        let text = text_of(&result);
        assert!(text.contains("401"));
        assert!(!text.contains("svc-key"));
        assert!(
            metrics
                .encode()
                .unwrap()
                .contains("outcome=\"tool_error\"} 1")
        );
    }

    #[tokio::test]
    async fn bad_calls_fail_at_the_mcp_level() {
        let (server, metrics) = server("http://127.0.0.1:1");

        assert!(server.call(None, "no_such_tool", None).await.is_err());
        assert!(
            server
                .call(
                    None,
                    "hyperbilling_get_recent_billing",
                    Some(arguments(
                        json!({"linked_accounts": ["123456789012"], "view": "yearly"})
                    )),
                )
                .await
                .is_err()
        );
        assert!(
            server
                .call(None, "hyperbilling_list_linked_accounts", None)
                .await
                .is_err()
        );

        let out = metrics.encode().unwrap();
        assert!(out.contains("tool_calls_total{tool=\"unknown\",outcome=\"error\"} 1"));
        assert!(!out.contains("no_such_tool"));
    }

    #[test]
    fn caller_credentials_need_both_headers() {
        let mut headers = http::HeaderMap::new();
        headers.insert(CLIENT_ID_HEADER, "caller@example.com".parse().unwrap());
        assert!(credentials_from_headers(&headers).is_none());
        headers.insert(CLIENT_SECRET_HEADER, "caller-key".parse().unwrap());
        assert!(credentials_from_headers(&headers).is_some());
    }
}
