use std::error::Error;
use axum::http::StatusCode;
use metrics::counter;

pub type Result<T> = std::result::Result<T, Box<dyn Error>>;

pub fn internal_error<E:Error>(err: E) -> (StatusCode, String) {
    tracing::error!("{}", err);
    
    let labels = [("error", format!("{err}!"))];
    counter!("request_error", &labels);
    (StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
}