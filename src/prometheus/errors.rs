use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};

// Errors
pub enum Errors {
    ErrorGetPrometheusData,
}

impl IntoResponse for Errors {
    fn into_response(self) -> Response {
        match self {
            Errors::ErrorGetPrometheusData => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json("Error: getting prometheus data."),
            )
                .into_response(),
        }
    }
}
