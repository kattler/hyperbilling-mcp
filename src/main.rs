#[tokio::main]
async fn main() -> anyhow::Result<()> {
    hyperbilling_mcp::start().await
}
