pub mod app;


use crate::{Config, LinkRepository, Result};
use crate::app::app;

pub struct Server {
    config: Config,
}
impl Server {
    pub fn new(config: Config) -> Self {
        Self { config }
    }
    pub async fn start(&self) -> Result<()> {
        let addr = format!("{}:{}", self.config.host, self.config.port);
        let listener = tokio::net::TcpListener::bind(&addr).await?;
        tracing::debug!("listening on {}", listener.local_addr()?);
        let repository = LinkRepository::new(&self.config.db_url).await?;
        
        let app = app(repository);
        
        axum::serve(listener, app).await?;
        Ok(())
    }
}