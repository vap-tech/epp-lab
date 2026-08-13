use axum::{Json, extract::State, http::StatusCode};
use serde::Serialize;
use std::sync::Arc;

use crate::app::AppState;

#[derive(Serialize)]
pub(crate) struct HealthResponse {
    status: &'static str,
    database: &'static str,
}

pub(crate) async fn health(
    State(state): State<Arc<AppState>>,
) -> (StatusCode, Json<HealthResponse>) {
    let database_ok = sqlx::query("SELECT 1").execute(&state.db).await.is_ok();
    let response = HealthResponse {
        status: if database_ok { "ok" } else { "degraded" },
        database: if database_ok { "ok" } else { "unavailable" },
    };
    (
        if database_ok {
            StatusCode::OK
        } else {
            StatusCode::SERVICE_UNAVAILABLE
        },
        Json(response),
    )
}
