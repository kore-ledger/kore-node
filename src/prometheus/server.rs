use std::sync::{Arc, RwLock};

use axum::{routing::get, Extension, Json, Router};
use prometheus_client::{encoding::text::encode, registry::Registry};
use super::{common::State, errors::Errors};


pub async fn handler_prometheus_data(Extension(state): Extension<Arc<RwLock<State>>>) -> Result<Json<String>, Errors> {
  let state_read = state.read().map_err(|_| Errors::ErrorGetPrometheusData)?;
  let mut body = String::new();
  encode(&mut body, &state_read.registry).map_err(|_| Errors::ErrorGetPrometheusData)?;
  
  Ok(Json(body))
}

pub fn build_routes(registry: Registry) -> Router {
  let state = Arc::new(RwLock::new(State {
    registry: registry
  }));
  
  let endpoints = Router::new()
    .route("/metrics", get(handler_prometheus_data))
    .layer(Extension(state));

  Router::new().merge(endpoints)
}

pub fn run_prometheus(registry: Registry) {
  let routes = build_routes(registry);

  tokio::spawn(async move {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await.unwrap();
    axum::serve(listener, routes).await.unwrap();
  });
}