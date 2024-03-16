use axum::extract::{Request, State};
use axum::http::StatusCode;
use axum::middleware::Next;
use axum::response::IntoResponse;
use metrics::counter;
use sha3::{Sha3_256, Digest};
use crate::error::internal_error;
use crate::LinkRepository;

#[derive(Clone, Debug)]
pub struct Settings {
    #[allow(dead_code)]
    pub id: String,
    pub encrypted_global_api_key: String,
}

pub async fn auth(
    State(repo): State<LinkRepository>,
    req: Request,
    next: Next
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let labels = [("uri", format!("{}!", req.uri()))];
    let api_key = req
        .headers()
        .get("x-api-key")
        .map(|v| v.to_str().unwrap_or_default().to_string())
        .ok_or_else(|| {
            tracing::error!("Unauthorized call to API: No key header received");
            counter!("unauthenticated_calls_count", &labels);
            (StatusCode::UNAUTHORIZED, "Unauthorized".to_string())
        })?;
    let fetch_to = tokio::time::Duration::from_millis(300);
    let settings = tokio::time::timeout(fetch_to, repo.get_settings()).await.map_err(internal_error)?.map_err(internal_error)?;
    let mut hasher = Sha3_256::new();
    hasher.update(api_key.as_bytes());
    let provided_api_key = hasher.finalize();
    if settings.encrypted_global_api_key != format!("{provided_api_key:x}") {
        tracing::error!("Unauthorized call to API: Invalid key");
        counter!("unauthenticated_calls_count", &labels);
        return Err((StatusCode::UNAUTHORIZED, "Unauthorized".to_string()));
    }
    Ok(next.run(req).await)
}