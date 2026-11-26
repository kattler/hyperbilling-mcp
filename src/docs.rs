//! The embedded documentation page, rendered once per locale at startup from
//! the registered tool definitions and the message catalogs in `docs/locales`.

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::sync::Arc;

use axum::Router;
use axum::extract::{RawQuery, State};
use axum::http::{HeaderMap, header};
use axum::response::{IntoResponse, Redirect, Response};
use axum::routing::get;
use serde_json::Value;

use crate::mcp::server::ToolDoc;

const PAGE: &str = include_str!("docs/page.html");
// The vendor's wordmark, kept byte for byte as published. `logo` adapts it.
const LOGO: &str = include_str!("docs/hyperbilling-logo.svg");
// The page has one inline stylesheet and loads nothing else.
const CONTENT_SECURITY_POLICY: &str =
    "default-src 'none'; style-src 'unsafe-inline'; base-uri 'none'; frame-ancestors 'none'";

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

    /// Shown in the switcher in its own language, whatever the page locale.
    const fn label(self) -> &'static str {
        match self {
            Self::En => "English",
            Self::Ko => "한국어",
        }
    }

    const fn catalog(self) -> &'static str {
        match self {
            Self::En => include_str!("docs/locales/en.json"),
            Self::Ko => include_str!("docs/locales/ko.json"),
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

/// `page.html` and UI strings by key. Values are trusted markup compiled into
/// the binary. `tool.*` and `arg.*` entries translate text that otherwise
/// comes from the tool definitions, which stay English for the agent.
struct Catalog(BTreeMap<String, String>);

impl Catalog {
    fn load(locale: Locale) -> Self {
        Self(serde_json::from_str(locale.catalog()).expect("embedded locale catalog is valid JSON"))
    }

    fn text<'a>(&'a self, key: &'a str) -> &'a str {
        self.0.get(key).map_or(key, String::as_str)
    }

    fn translated<'a>(&'a self, key: &str, source: &'a str) -> &'a str {
        self.0.get(key).map_or(source, String::as_str)
    }
}

pub struct Pages {
    en: String,
    ko: String,
}

impl Pages {
    pub fn render(version: &str, tools: &[ToolDoc]) -> Arc<Self> {
        Arc::new(Self {
            en: render(Locale::En, version, tools),
            ko: render(Locale::Ko, version, tools),
        })
    }

    fn get(&self, locale: Locale) -> &str {
        match locale {
            Locale::En => &self.en,
            Locale::Ko => &self.ko,
        }
    }
}

pub fn router(pages: Arc<Pages>) -> Router {
    Router::new()
        .route("/", get(|| async { Redirect::temporary("/docs") }))
        .route("/docs", get(serve))
        .with_state(pages)
}

async fn serve(
    State(pages): State<Arc<Pages>>,
    RawQuery(query): RawQuery,
    headers: HeaderMap,
) -> Response {
    let accept_language = headers
        .get(header::ACCEPT_LANGUAGE)
        .and_then(|value| value.to_str().ok());
    let locale = Locale::negotiate(query.as_deref(), accept_language);
    (
        [
            (header::CONTENT_TYPE, "text/html; charset=utf-8"),
            (header::CONTENT_LANGUAGE, locale.code()),
            (header::VARY, "Accept-Language"),
            (header::CONTENT_SECURITY_POLICY, CONTENT_SECURITY_POLICY),
            (header::X_CONTENT_TYPE_OPTIONS, "nosniff"),
            (header::CACHE_CONTROL, "no-cache"),
        ],
        pages.get(locale).to_string(),
    )
        .into_response()
}

/// The published file is white on a fixed canvas with wide margins. Inline it
/// cropped to the drawing and in the text color, so it reads in both themes.
fn logo() -> String {
    LOGO.trim()
        .replace(
            r#"width="580" height="150" viewBox="0 0 580 150""#,
            r#"class="logo" viewBox="62 33 457 84" role="img" aria-label="HyperBilling""#,
        )
        .replace(r#"fill="white""#, r#"fill="currentColor""#)
}

fn render(locale: Locale, version: &str, tools: &[ToolDoc]) -> String {
    let catalog = Catalog::load(locale);
    let mut nav = String::new();
    let mut body = String::new();
    for tool in tools {
        let name = escape(&tool.name);
        // Every tool shares the product prefix, which only eats sidebar width.
        let label = escape(
            tool.name
                .strip_prefix("hyperbilling_")
                .unwrap_or(&tool.name),
        );
        let _ = writeln!(nav, r##"  <a class="mono" href="#{name}">{label}</a>"##);
        render_tool(&mut body, &catalog, tool);
    }
    let mut page = PAGE
        .replace("{{lang}}", locale.code())
        .replace("{{version}}", &escape(version))
        .replace("{{switcher}}", &switcher(locale, &catalog))
        .replace("{{logo}}", &logo())
        .replace("{{nav}}", nav.trim_end())
        .replace("{{tools}}", body.trim_end());
    for (key, value) in &catalog.0 {
        page = page.replace(&format!("{{{{t:{key}}}}}"), value);
    }
    // Catalog text may itself carry this placeholder, so it resolves last.
    page.replace("{{tool_count}}", &tools.len().to_string())
}

/// A `details` element, so the menu opens without script.
fn switcher(current: Locale, catalog: &Catalog) -> String {
    let mut out = format!(
        r#"<details class="lang"><summary aria-label="{}" lang="{}">{}</summary><div class="menu">"#,
        escape(catalog.text("ui.language")),
        current.code(),
        current.label(),
    );
    for locale in Locale::ALL {
        let code = locale.code();
        let state = if locale == current {
            r#" class="on" aria-current="true""#
        } else {
            ""
        };
        let _ = write!(
            out,
            r#"<a href="?lang={code}" hreflang="{code}" lang="{code}"{state}>{}</a>"#,
            locale.label()
        );
    }
    out.push_str("</div></details>");
    out
}

fn render_tool(out: &mut String, catalog: &Catalog, tool: &ToolDoc) {
    let name = escape(&tool.name);
    let (method, path) = tool.upstream.split_once(' ').unwrap_or(("", tool.upstream));
    let description = catalog.translated(
        &format!("tool.{}.description", tool.name),
        &tool.description,
    );
    let _ = write!(
        out,
        r##"    <article class="tool" id="{name}">
      <header><h3>{name}<a class="anchor" href="#{name}" aria-label="{permalink}">#</a></h3></header>
      <div class="upstream"><span class="method {class}">{method}</span><code>{path}</code></div>
      <p>{description}</p>
"##,
        permalink = escape(catalog.text("ui.permalink")),
        class = method.to_ascii_lowercase(),
        method = escape(method),
        path = escape(path),
        description = escape(description),
    );

    let arguments = arguments(&tool.schema);
    if arguments.is_empty() {
        let _ = writeln!(
            out,
            r#"      <p class="none">{}</p>"#,
            escape(catalog.text("ui.no_arguments"))
        );
    } else {
        let _ = writeln!(
            out,
            "      <div class=\"table\"><table>\n        <thead><tr><th>{}</th><th></th><th>{}</th></tr></thead>\n        <tbody>",
            escape(catalog.text("ui.argument")),
            escape(catalog.text("ui.description")),
        );
        for argument in arguments {
            render_argument(out, catalog, &tool.name, &argument);
        }
        out.push_str("        </tbody>\n      </table></div>\n");
    }
    out.push_str("    </article>\n");
}

fn render_argument(out: &mut String, catalog: &Catalog, tool: &str, argument: &Argument) {
    let (class, key) = if argument.required {
        ("req", "ui.required")
    } else {
        ("opt", "ui.optional")
    };
    let description = catalog.translated(
        &format!("arg.{tool}.{}", argument.name),
        &argument.description,
    );
    let _ = write!(
        out,
        r#"          <tr><td><span class="name">{name}</span><br><span class="type">{kind}</span></td><td><span class="{class}">{presence}</span></td><td>{description}"#,
        name = escape(&argument.name),
        kind = escape(&argument.kind),
        presence = escape(catalog.text(key)),
        description = escape(description),
    );
    if !argument.values.is_empty() {
        out.push_str(r#"<div class="values">"#);
        for value in &argument.values {
            let _ = write!(out, "<code>{}</code>", escape(value));
        }
        out.push_str("</div>");
    }
    out.push_str("</td></tr>\n");
}

struct Argument {
    name: String,
    kind: String,
    description: String,
    required: bool,
    values: Vec<String>,
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

fn escape(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for c in text.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            _ => out.push(c),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use serde_json::json;

    use super::*;
    use crate::hyperbilling::client::tests::client;
    use crate::mcp::Server;
    use crate::observability::Metrics;

    fn tool(name: &str, upstream: &'static str, schema: Value) -> ToolDoc {
        ToolDoc {
            name: name.to_string(),
            description: "Lists <things> & more".to_string(),
            upstream,
            schema: Arc::new(serde_json::from_value(schema).unwrap()),
        }
    }

    fn real_tools() -> Vec<ToolDoc> {
        let metrics = Arc::new(Metrics::new("test", "test"));
        Server::new(Arc::new(client("http://127.0.0.1:1", None)), metrics).tool_docs()
    }

    fn template_keys() -> BTreeSet<String> {
        PAGE.split("{{t:")
            .skip(1)
            .map(|rest| rest.split("}}").next().unwrap().to_string())
            .collect()
    }

    /// Keys a tool definition can supply are optional in a catalog. Every
    /// other key is page or UI text and must exist in each locale.
    fn page_keys(catalog: &Catalog) -> BTreeSet<String> {
        catalog
            .0
            .keys()
            .filter(|key| !key.starts_with("tool.") && !key.starts_with("arg."))
            .cloned()
            .collect()
    }

    #[test]
    fn renders_tools_arguments_and_escapes_text() {
        let page = render(
            Locale::En,
            "1.2.3",
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
        assert!(page.contains(r#"<meta name="color-scheme" content="dark light">"#));
        assert!(page.contains("@media (prefers-color-scheme: light)"));
        assert!(page.contains("v1.2.3"));
        assert!(page.contains(
            r##"<a class="mono" href="#hyperbilling_search_billing">search_billing</a>"##
        ));
        assert!(page.contains(r#"<article class="tool" id="hyperbilling_list_linked_accounts">"#));
        assert!(
            page.contains(
                r#"<span class="method post">POST</span><code>/v1/search/billing</code>"#
            )
        );
        assert!(page.contains(r#"<span class="method get">GET</span>"#));
        assert!(page.contains("Lists &lt;things&gt; &amp; more"));
        assert!(page.contains("a &quot;fixed&quot; report"));
        assert!(page.contains(r#"<span class="type">string[]</span>"#));
        assert!(page.contains(r#"<span class="type">object</span>"#));
        assert!(page.contains("<code>last-3-months</code>"));
        assert!(page.contains("No arguments."));
        assert!(page.contains(
            "<h3>hyperbilling_search_billing<a class=\"anchor\" href=\"#hyperbilling_search_billing\" aria-label=\"Link to this section\">#</a></h3>"
        ));
        assert_eq!(page.matches(r#"<a class="anchor""#).count(), 10);
        assert!(page.contains("2 tools, nothing cached"));
        assert!(page.contains(r#"<div class="flow" role="img" aria-label="Architecture:"#));
        assert!(page.contains(r#"<details class="arch" id="arch-direct">"#));
        assert!(page.contains(r#"<details class="arch" id="arch-litellm">"#));
        let required = page.find(r#"<span class="name">view</span>"#).unwrap();
        let optional = page.find(r#"<span class="name">accounts</span>"#).unwrap();
        assert!(required < optional, "required arguments come first");
    }

    #[test]
    fn every_locale_covers_every_template_and_ui_key() {
        let template = template_keys();
        assert!(!template.is_empty());
        let english = page_keys(&Catalog::load(Locale::En));
        for locale in Locale::ALL {
            let keys = page_keys(&Catalog::load(locale));
            assert_eq!(keys, english, "{locale:?} page keys differ from English");
            let missing: Vec<_> = template.difference(&keys).collect();
            assert!(
                missing.is_empty(),
                "{locale:?} lacks template keys {missing:?}"
            );
            let unused: Vec<_> = keys
                .difference(&template)
                .filter(|key| !key.starts_with("ui."))
                .collect();
            assert!(unused.is_empty(), "{locale:?} has unused keys {unused:?}");
        }
    }

    #[test]
    fn korean_translates_every_tool_and_argument() {
        let catalog = Catalog::load(Locale::Ko);
        let tools = real_tools();
        let mut expected = BTreeSet::new();
        for tool in &tools {
            expected.insert(format!("tool.{}.description", tool.name));
            for argument in arguments(&tool.schema) {
                expected.insert(format!("arg.{}.{}", tool.name, argument.name));
            }
        }
        let translated: BTreeSet<String> = catalog
            .0
            .keys()
            .filter(|key| key.starts_with("tool.") || key.starts_with("arg."))
            .cloned()
            .collect();
        assert_eq!(translated, expected, "ko.json must match the tool surface");

        let page = render(Locale::Ko, "1.2.3", &tools);
        assert!(!page.contains("{{"));
        assert!(page.contains(r#"<html lang="ko">"#));
        assert!(page.contains("사용 제한"));
        assert!(page.contains(r#"<span class="req">필수</span>"#));
        for tool in &tools {
            assert!(
                !page.contains(&escape(&tool.description)),
                "{} is untranslated",
                tool.name
            );
        }
    }

    #[test]
    fn english_page_uses_the_tool_definitions() {
        let tools = real_tools();
        let page = render(Locale::En, "1.2.3", &tools);
        assert!(!page.contains("{{"));
        for tool in &tools {
            assert!(page.contains(&escape(&tool.description)));
        }
        assert!(page.contains(r#"<a href="?lang=en" hreflang="en" lang="en" class="on" aria-current="true">English</a>"#));
        assert!(page.contains(r#"<a href="?lang=ko" hreflang="ko" lang="ko">한국어</a>"#));
        assert!(page.contains(
            r#"<details class="lang"><summary aria-label="Language" lang="en">English</summary>"#
        ));
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
        assert_eq!(Catalog::load(Locale::En).text("no.such.key"), "no.such.key");
    }

    #[test]
    fn logo_is_cropped_and_follows_the_text_color() {
        let logo = logo();
        assert!(logo.starts_with(
            r#"<svg class="logo" viewBox="62 33 457 84" role="img" aria-label="HyperBilling""#
        ));
        assert!(logo.contains(r#"fill="currentColor""#));
        assert!(!logo.contains(r#"fill="white""#));
        assert!(!logo.contains("<script"));
        let page = render(Locale::En, "1.2.3", &[]);
        assert_eq!(page.matches(r#"<svg class="logo""#).count(), 3);
        assert!(page.contains("not affiliated with or endorsed by MegazoneCloud"));
    }

    #[test]
    fn kind_falls_back_to_any() {
        assert_eq!(kind(&json!({})), "any");
        assert_eq!(kind(&json!({"type": ["null"]})), "any");
        assert_eq!(kind(&json!({"type": "array"})), "any[]");
        assert_eq!(escape("it's"), "it&#39;s");
    }
}
