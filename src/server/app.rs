use axum::{middleware, Router};
use axum::routing::{get, patch, post};
use axum_prometheus::PrometheusMetricLayer;
use tower_http::trace::TraceLayer;
use crate::auth::auth;
use crate::controllers::{create_link, get_link_statistic, health, redirect, update_link};
use crate::repositories::link_repository::LinkRepository;

pub fn app(repository: LinkRepository) -> Router {
    let (prometheus_layer, metric_handler) = PrometheusMetricLayer::pair();
    Router::new()
        .route("/create", post(create_link))
        .route("/:id/statistics", get(get_link_statistic))
        .route_layer(middleware::from_fn_with_state(repository.clone(), auth))
        .route(
            "/:id", 
            patch(update_link)
                .route_layer(middleware::from_fn_with_state(repository.clone(), auth))
                .get(redirect))
        .route("/metrics", get(|| async move {metric_handler.render()}))
        .route("/health", get(health))
        .layer(TraceLayer::new_for_http())
        .layer(prometheus_layer)
        .with_state(repository)
}