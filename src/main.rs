use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

use rust_urlshortener::{Config, Result, Server};

#[tokio::main]
async fn main() -> Result<()> {
    let config = Config::from_env();
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| "rust_urlshortener=debug".into())
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
    let server = Server::new(config);
    server.start().await?;
    Ok(())
}