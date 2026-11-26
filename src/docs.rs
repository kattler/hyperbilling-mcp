//! The embedded documentation pages: a React bundle built from `web/`, served
//! with the registered tool definitions injected as JSON.

use std::sync::Arc;

use axum::Router;
use axum::extract::{RawQuery, State};
use axum::http::{HeaderMap, header};
use axum::response::{IntoResponse, Redirect, Response};
use axum::routing::{MethodRouter, get};
use serde::Serialize;
use serde_json::Value;

use crate::mcp::args::RecentView;
use crate::mcp::server::ToolDoc;
use crate::observability::UpstreamHealth;

const SHELL: &str = include_str!("docs/shell.html");
const SCRIPT: &str = include_str!("../web/dist/app.js");
const STYLESHEET: &str = include_str!("../web/dist/app.css");
const CONTENT_SECURITY_POLICY: &str = "default-src 'none'; script-src 'self'; style-src 'self'; connect-src 'self'; base-uri 'none'; frame-ancestors 'none'";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Locale {
    En,
    Ko,
}

impl Locale {
    const ALL: [Self; 2] = [Self::En, Self::Ko];

    const fn code(self) -> &'static str {
        match self {
            Self::En => "en",
            Self::Ko => "ko",
        }
    }

    fn from_tag(tag: &str) -> Option<Self> {
        let primary = tag.trim().split(['-', '_']).next()?.to_ascii_lowercase();
        Self::ALL
            .into_iter()
            .find(|locale| locale.code() == primary)
    }

    /// An explicit `?lang=` wins, then the first supported `Accept-Language`
    /// entry, then English.
    fn negotiate(query: Option<&str>, accept_language: Option<&str>) -> Self {
        let from_query = query
            .into_iter()
            .flat_map(|query| query.split('&'))
            .find_map(|pair| pair.strip_prefix("lang="))
            .and_then(Self::from_tag);
        let from_header = || {
            accept_language?
                .split(',')
                .filter_map(|entry| entry.split(';').next())
                .find_map(Self::from_tag)
        };
        from_query.or_else(from_header).unwrap_or(Self::En)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Page {
    Guide,
    Api,
}

impl Page {
    const ALL: [Self; 2] = [Self::Guide, Self::Api];

    const fn code(self) -> &'static str {
        match self {
            Self::Guide => "guide",
            Self::Api => "api",
        }
    }

    /// Relative, so the pages keep working behind a path prefix.
    const fn assets(self) -> &'static str {
        match self {
            Self::Guide => "docs/",
            Self::Api => "",
        }
    }
}

pub struct Pages(Vec<(Page, Locale, String)>);

impl Pages {
    pub fn render(version: &str, built: &str, tools: &[ToolDoc]) -> Arc<Self> {
        let pages = Page::ALL.into_iter().flat_map(|page| {
            Locale::ALL
                .into_iter()
                .map(move |locale| (page, locale, render(page, locale, version, built, tools)))
        });
        Arc::new(Self(pages.collect()))
    }

    fn get(&self, page: Page, locale: Locale) -> &str {
        self.0
            .iter()
            .find(|(p, l, _)| *p == page && *l == locale)
            .map_or("", |(_, _, html)| html)
    }
}

pub fn router(pages: Arc<Pages>, health: Arc<UpstreamHealth>) -> Router {
    Router::new()
        .route("/", get(|| async { Redirect::temporary("/docs") }))
        .route("/docs", page(Page::Guide))
        .route("/docs/api", page(Page::Api))
        .route(
            "/docs/status",
            get(move || {
                let health = health.clone();
                async move { status(&health) }
            }),
        )
        .route(
            "/docs/app.js",
            get(|| async { asset("text/javascript; charset=utf-8", SCRIPT) }),
        )
        .route(
            "/docs/app.css",
            get(|| async { asset("text/css; charset=utf-8", STYLESHEET) }),
        )
        .with_state(pages)
}

fn page(page: Page) -> MethodRouter<Arc<Pages>> {
    get(
        move |State(pages): State<Arc<Pages>>, RawQuery(query): RawQuery, headers: HeaderMap| async move {
            serve(page, &pages, query.as_deref(), &headers)
        },
    )
}

fn serve(page: Page, pages: &Pages, query: Option<&str>, headers: &HeaderMap) -> Response {
    let accept_language = headers
        .get(header::ACCEPT_LANGUAGE)
        .and_then(|value| value.to_str().ok());
    let locale = Locale::negotiate(query, accept_language);
    (
        [
            (header::CONTENT_TYPE, "text/html; charset=utf-8"),
            (header::CONTENT_LANGUAGE, locale.code()),
            (header::VARY, "Accept-Language"),
            (header::CONTENT_SECURITY_POLICY, CONTENT_SECURITY_POLICY),
            (header::X_CONTENT_TYPE_OPTIONS, "nosniff"),
            (header::CACHE_CONTROL, "no-cache"),
        ],
        pages.get(page, locale).to_string(),
    )
        .into_response()
}

fn status(health: &UpstreamHealth) -> Response {
    let body = serde_json::to_string(&health.snapshot()).expect("status snapshot serializes");
    (
        [
            (header::CONTENT_TYPE, "application/json"),
            (header::X_CONTENT_TYPE_OPTIONS, "nosniff"),
            (header::CACHE_CONTROL, "no-store"),
        ],
        body,
    )
        .into_response()
}

fn asset(content_type: &'static str, body: &'static str) -> Response {
    (
        [
            (header::CONTENT_TYPE, content_type),
            (header::X_CONTENT_TYPE_OPTIONS, "nosniff"),
            (header::CACHE_CONTROL, "no-cache"),
        ],
        body,
    )
        .into_response()
}

#[derive(Serialize)]
struct Payload<'a> {
    page: &'static str,
    locale: &'static str,
    version: &'a str,
    built: &'a str,
    tools: Vec<Tool<'a>>,
    views: Vec<View>,
}

#[derive(Serialize)]
struct View {
    name: &'static str,
    path: &'static str,
}

#[derive(Serialize)]
struct Tool<'a> {
    name: &'a str,
    description: &'a str,
    method: &'a str,
    path: &'a str,
    arguments: Vec<Argument>,
}

#[derive(Serialize)]
struct Argument {
    name: String,
    kind: String,
    description: String,
    required: bool,
    values: Vec<String>,
}

fn render(page: Page, locale: Locale, version: &str, built: &str, tools: &[ToolDoc]) -> String {
    let payload = Payload {
        page: page.code(),
        locale: locale.code(),
        version,
        built,
        tools: tools
            .iter()
            .map(|tool| {
                let (method, path) = tool.upstream.split_once(' ').unwrap_or(("", tool.upstream));
                Tool {
                    name: &tool.name,
                    description: &tool.description,
                    method,
                    path,
                    arguments: arguments(&tool.schema),
                }
            })
            .collect(),
        views: RecentView::ALL
            .into_iter()
            .map(|view| View {
                name: view.name(),
                path: view.path(),
            })
            .collect(),
    };
    // An unescaped `</script>` in a description would end the data block early.
    let data = serde_json::to_string(&payload)
        .expect("docs payload serializes")
        .replace('<', "\\u003c");
    SHELL
        .replace("{{lang}}", locale.code())
        .replace("{{assets}}", page.assets())
        .replace("{{data}}", &data)
}

/// Required arguments first, each group in schema order.
fn arguments(schema: &serde_json::Map<String, Value>) -> Vec<Argument> {
    let strings = |value: Option<&Value>| -> Vec<String> {
        value
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .filter_map(Value::as_str)
            .map(str::to_string)
            .collect()
    };
    let required = strings(schema.get("required"));
    let mut arguments: Vec<Argument> = schema
        .get("properties")
        .and_then(Value::as_object)
        .into_iter()
        .flatten()
        .map(|(name, property)| Argument {
            name: name.clone(),
            kind: kind(property),
            description: property
                .get("description")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string(),
            required: required.contains(name),
            values: strings(property.get("enum")),
        })
        .collect();
    arguments.sort_by_key(|argument| !argument.required);
    arguments
}

fn kind(property: &Value) -> String {
    let name = match property.get("type") {
        Some(Value::String(name)) => name.as_str(),
        Some(Value::Array(names)) => names
            .iter()
            .filter_map(Value::as_str)
            .find(|name| *name != "null")
            .unwrap_or("any"),
        _ => "any",
    };
    if name == "array" {
        let item = property
            .get("items")
            .and_then(|items| items.get("type"))
            .and_then(Value::as_str)
            .unwrap_or("any");
        return format!("{item}[]");
    }
    name.to_string()
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, BTreeSet};
    use std::path::Path;

    use serde_json::json;

    use super::*;
    use crate::hyperbilling::client::tests::client;
    use crate::mcp::Server;
    use crate::observability::Metrics;

    type Catalog = BTreeMap<String, String>;

    fn catalog(locale: Locale) -> Catalog {
        let source = match locale {
            Locale::En => include_str!("../web/src/locales/en.json"),
            Locale::Ko => include_str!("../web/src/locales/ko.json"),
        };
        serde_json::from_str(source).expect("locale catalog is valid JSON")
    }

    fn tool(name: &str, upstream: &'static str, schema: Value) -> ToolDoc {
        ToolDoc {
            name: name.to_string(),
            description: "Lists <things> & more</script>".to_string(),
            upstream,
            schema: Arc::new(serde_json::from_value(schema).unwrap()),
        }
    }

    fn real_tools() -> Vec<ToolDoc> {
        let metrics = Arc::new(Metrics::new("test", "test"));
        Server::new(Arc::new(client("http://127.0.0.1:1", None)), metrics).tool_docs()
    }

    /// The JSON the page reads back out of its data block.
    fn payload(page: &str) -> Value {
        let (_, rest) = page
            .split_once(r#"<script id="docs-data" type="application/json">"#)
            .expect("page carries a data block");
        let (data, _) = rest.split_once("</script>").unwrap();
        serde_json::from_str(data).expect("data block is valid JSON")
    }

    /// Keys a tool definition can supply are optional in a catalog. Every
    /// other key is page or UI text and must exist in each locale.
    fn page_keys(catalog: &Catalog) -> BTreeSet<String> {
        catalog
            .keys()
            .filter(|key| !key.starts_with("tool.") && !key.starts_with("arg."))
            .cloned()
            .collect()
    }

    fn web_sources() -> String {
        let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("web/src");
        std::fs::read_dir(dir)
            .unwrap()
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .filter(|path| path.extension().is_some_and(|ext| ext == "tsx"))
            .map(|path| std::fs::read_to_string(path).unwrap())
            .collect()
    }

    #[test]
    fn injects_tools_and_arguments_as_escaped_json() {
        let page = render(
            Page::Guide,
            Locale::En,
            "1.2.3",
            "2026-01-02",
            &[
                tool(
                    "hyperbilling_search_billing",
                    "POST /v1/search/billing",
                    json!({
                        "type": "object",
                        "required": ["view"],
                        "properties": {
                            "accounts": {"type": "array", "items": {"type": "string"}, "description": "IDs"},
                            "filter": {"type": ["object", "null"]},
                            "view": {"type": "string", "enum": ["latest", "last-3-months"], "description": "a \"fixed\" report"}
                        }
                    }),
                ),
                tool(
                    "hyperbilling_list_linked_accounts",
                    "GET /v1/search/linkedaccount",
                    json!({"type": "object"}),
                ),
            ],
        );

        assert!(!page.contains("{{"));
        assert!(page.contains(r#"<html lang="en">"#));
        assert!(page.contains(r#"<link rel="stylesheet" href="docs/app.css">"#));
        assert!(page.contains(r#"<script type="module" src="docs/app.js"></script>"#));
        assert_eq!(page.matches("</script>").count(), 2);

        let data = payload(&page);
        assert_eq!(data["locale"], "en");
        assert_eq!(data["page"], "guide");
        assert_eq!(data["version"], "1.2.3");
        assert_eq!(data["built"], "2026-01-02");
        let search = &data["tools"][0];
        assert_eq!(search["name"], "hyperbilling_search_billing");
        assert_eq!(search["description"], "Lists <things> & more</script>");
        assert_eq!(search["method"], "POST");
        assert_eq!(search["path"], "/v1/search/billing");
        let names: Vec<_> = search["arguments"]
            .as_array()
            .unwrap()
            .iter()
            .map(|argument| argument["name"].as_str().unwrap())
            .collect();
        assert_eq!(names, ["view", "accounts", "filter"]);
        assert_eq!(search["arguments"][0]["required"], true);
        assert_eq!(
            search["arguments"][0]["values"],
            json!(["latest", "last-3-months"])
        );
        assert_eq!(search["arguments"][0]["description"], "a \"fixed\" report");
        assert_eq!(search["arguments"][1]["kind"], "string[]");
        assert_eq!(search["arguments"][2]["kind"], "object");
        assert_eq!(data["tools"][1]["method"], "GET");
        assert_eq!(data["tools"][1]["arguments"], json!([]));
    }

    #[test]
    fn every_locale_covers_every_page_key_and_the_page_uses_each() {
        let english = page_keys(&catalog(Locale::En));
        assert!(!english.is_empty());
        assert_eq!(
            page_keys(&catalog(Locale::Ko)),
            english,
            "Korean page keys differ from English"
        );
        let sources = web_sources();
        let unused: Vec<_> = english
            .iter()
            .filter(|key| !sources.contains(&format!("\"{key}\"")))
            .collect();
        assert!(
            unused.is_empty(),
            "catalog keys the page never reads: {unused:?}"
        );
    }

    #[test]
    fn korean_translates_every_tool_and_argument() {
        let korean = catalog(Locale::Ko);
        let tools = real_tools();
        let mut expected = BTreeSet::new();
        for tool in &tools {
            expected.insert(format!("tool.{}.description", tool.name));
            for argument in arguments(&tool.schema) {
                expected.insert(format!("arg.{}.{}", tool.name, argument.name));
            }
        }
        let translated: BTreeSet<String> = korean
            .keys()
            .filter(|key| key.starts_with("tool.") || key.starts_with("arg."))
            .cloned()
            .collect();
        assert_eq!(translated, expected, "ko.json must match the tool surface");
        assert!(
            catalog(Locale::En)
                .keys()
                .all(|key| !key.starts_with("tool.") && !key.starts_with("arg.")),
            "English tool text comes from the tool definitions"
        );
    }

    #[test]
    fn page_carries_the_real_tool_definitions() {
        let tools = real_tools();
        let page = render(Page::Guide, Locale::Ko, "1.2.3", "2026-01-02", &tools);
        assert!(page.contains(r#"<html lang="ko">"#));
        let data = payload(&page);
        assert_eq!(data["locale"], "ko");
        let rendered = data["tools"].as_array().unwrap();
        assert_eq!(rendered.len(), tools.len());
        for (rendered, tool) in rendered.iter().zip(&tools) {
            assert_eq!(rendered["name"], tool.name);
            assert_eq!(rendered["description"], tool.description);
        }
    }

    #[test]
    fn api_page_loads_assets_from_its_own_directory_and_lists_the_views() {
        let page = render(Page::Api, Locale::En, "1.2.3", "2026-01-02", &real_tools());
        assert!(page.contains(r#"<link rel="stylesheet" href="app.css">"#));
        assert!(page.contains(r#"<script type="module" src="app.js"></script>"#));
        assert!(!page.contains("{{"));
        let data = payload(&page);
        assert_eq!(data["page"], "api");
        let views = data["views"].as_array().unwrap();
        assert_eq!(views.len(), RecentView::ALL.len());
        assert_eq!(views[4]["name"], "latest-from-last-month-daily");
        assert_eq!(
            views[4]["path"],
            "/v2/search/billing/lastest-from-last-month-daily"
        );
    }

    #[test]
    fn setup_prompt_keeps_the_key_out_of_the_chat() {
        for locale in Locale::ALL {
            let prompt = &catalog(locale)["connect.prompt"];
            assert!(prompt.contains("- URL: {{origin}}/mcp"));
            assert!(prompt.contains("X-Client-Secret: ${HYPERBILLING_API_KEY}"));
            assert!(prompt.contains("hyperbilling_list_linked_accounts"));
        }
    }

    #[test]
    fn serves_the_bundle_under_a_policy_without_inline_code() {
        let script = asset("text/javascript; charset=utf-8", SCRIPT);
        assert_eq!(
            script.headers()[header::CONTENT_TYPE],
            "text/javascript; charset=utf-8"
        );
        assert_eq!(script.headers()[header::X_CONTENT_TYPE_OPTIONS], "nosniff");
        assert!(SCRIPT.contains("hyperbilling_list_linked_accounts"));
        assert!(STYLESHEET.contains("prefers-color-scheme"));
        assert!(CONTENT_SECURITY_POLICY.contains("script-src 'self'"));
        assert!(CONTENT_SECURITY_POLICY.contains("style-src 'self'"));
        assert!(CONTENT_SECURITY_POLICY.contains("connect-src 'self'"));
        assert!(!CONTENT_SECURITY_POLICY.contains("unsafe-inline"));
    }

    #[test]
    fn negotiates_query_then_header_then_english() {
        let pick = Locale::negotiate;
        assert_eq!(pick(None, None), Locale::En);
        assert_eq!(pick(Some("lang=ko"), Some("en-US")), Locale::Ko);
        assert_eq!(pick(Some("x=1&lang=EN"), Some("ko")), Locale::En);
        assert_eq!(
            pick(Some("lang=fr"), Some("ko-KR,ko;q=0.9,en;q=0.8")),
            Locale::Ko
        );
        assert_eq!(pick(None, Some("fr-FR, en-GB;q=0.8, ko;q=0.5")), Locale::En);
        assert_eq!(pick(None, Some("ja, zh_CN")), Locale::En);
    }

    #[test]
    fn kind_falls_back_to_any() {
        assert_eq!(kind(&json!({})), "any");
        assert_eq!(kind(&json!({"type": ["null"]})), "any");
        assert_eq!(kind(&json!({"type": "array"})), "any[]");
    }
}
