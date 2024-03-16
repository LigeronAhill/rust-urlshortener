
use axum::body::Body;
use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode};
use axum::Json;
use axum::response::Response;
use url::Url;
pub use health::health;
use crate::error::internal_error;
use crate::LinkRepository;
use crate::models::{CountedLinkStatistic, Link, LinkTarget};
mod health;


const DEFAULT_CACHE_CONTROL_HEADER_VALUE: &str = "public, max-age=300, s-maxage=300, stale-while-revalidate=300, stale-if-error=300";
type Result<T> = std::result::Result<T, (StatusCode, String)>;
static TIMEOUT: tokio::time::Duration = tokio::time::Duration::from_millis(300);


pub async fn redirect(State(repo): State<LinkRepository>, Path(requested_link): Path<String>, headers: HeaderMap) -> Result<Response> {
    let referer = headers.get("referer").map(|v| v.to_str().unwrap_or_default().to_string());
    let user_agent = headers.get("user-agent").map(|v| v.to_str().unwrap_or_default().to_string());
    
    let s = tokio::time::timeout(
        TIMEOUT, 
        repo.add_statistic(&requested_link, &referer, &user_agent))
            .await;
    
    match s {
        Ok(Err(e)) => tracing::error!("Saving statistic error: {:?}", e),
        Err(e) => tracing::error!("Saving statistic error: {:?}", e),
        _ => tracing::debug!("Saved statistic for link id {}, referer: {:?}, user agent: {:?}", requested_link, referer.unwrap_or_default(), user_agent.unwrap_or_default()),
    }
    
    tracing::debug!("Adding statistic for link id {}", requested_link);
    let link = tokio::time::timeout(
        TIMEOUT, 
        repo.get_link(&requested_link))
            .await
            .map_err(internal_error)?
            .map_err(internal_error)?
            .ok_or_else(|| "Not found".to_string())
            .map_err(|err| (StatusCode::NOT_FOUND, err))?; 
    
    tracing::debug!("Redirecting link id {} to {}", requested_link, link.target_url);
    Ok(Response::builder()
        .status(StatusCode::TEMPORARY_REDIRECT)
        .header("Location", link.target_url)
        .header("Cache-Control", DEFAULT_CACHE_CONTROL_HEADER_VALUE)
        .body(Body::empty())
        .expect("This response should always be constructable"))
}

pub async fn create_link(State(repo): State<LinkRepository>, Json(new_link): Json<LinkTarget>) -> Result<Json<Link>> {
    let url = Url::parse(&new_link.target_url).map_err(|_| ( StatusCode::CONFLICT, "Invalid URL".to_string()))?.to_string();
    let link = tokio::time::timeout(
        TIMEOUT, 
        repo.create_link(&url))
            .await
            .map_err(internal_error)?
            .map_err(internal_error)?;
    tracing::debug!("Created new link with id {} targeting {}", link.id, link.target_url);
    Ok(Json(link))
}

pub async fn update_link(State(repo): State<LinkRepository>, Path(link_id): Path<String>, Json(update_link): Json<LinkTarget>) -> Result<Json<Link>> {
    let url = Url::parse(&update_link.target_url)
        .map_err(|_| ( StatusCode::CONFLICT, "Invalid URL".to_string()))?
        .to_string();
    let link = tokio::time::timeout(
        TIMEOUT, 
        repo.update_link(&link_id, &url))
            .await
            .map_err(internal_error)?
            .map_err(internal_error)?;
    tracing::debug!("Updated link with id {} now targeting {}", link.id, link.target_url);
    Ok(Json(link))
}

pub async fn get_link_statistic(State(repo): State<LinkRepository>, Path(link_id): Path<String>) -> Result<Json<Vec<CountedLinkStatistic>>> {
    let statistic = tokio::time::timeout(
        TIMEOUT, 
        repo.get_link_statistic(&link_id))
            .await
            .map_err(internal_error)?
            .map_err(internal_error)?;
    tracing::debug!("Got statistic for link id {}", link_id);
    Ok(Json(statistic))
}