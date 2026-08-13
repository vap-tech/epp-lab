use std::sync::Arc;

use axum::{Router, routing::get};

use crate::app::AppState;

use super::{
    certificates::{create as create_certificate, list as list_certificates},
    health::health,
    info::info,
    registrars::{create, get as get_registrar, list},
};

pub(crate) fn router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/api/health", get(health))
        .route("/api/info", get(info))
        .route("/api/registrars", get(list).post(create))
        .route("/api/registrars/{id}", get(get_registrar))
        .route(
            "/api/registrars/{id}/certificates",
            get(list_certificates).post(create_certificate),
        )
        .with_state(state)
}
