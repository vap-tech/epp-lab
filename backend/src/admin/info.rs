use axum::{Json, extract::State};
use serde::Serialize;
use std::sync::Arc;

use crate::app::AppState;

#[derive(Serialize)]
pub(crate) struct InfoResponse {
    name: &'static str,
    version: &'static str,
    epp_bind: String,
    environment: String,
}

pub(crate) async fn info(State(state): State<Arc<AppState>>) -> Json<InfoResponse> {
    Json(InfoResponse {
        name: "epp-registry-simulator",
        version: env!("CARGO_PKG_VERSION"),
        epp_bind: state.settings.epp_bind.to_string(),
        environment: state.settings.app_env.clone(),
    })
}
