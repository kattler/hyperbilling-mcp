//! MCP server that proxies the Megazone HyperBilling Open API as read-only tools.

mod app;
mod config;
mod docs;
mod error;
mod hyperbilling;
mod mcp;
mod observability;

pub use app::start;
