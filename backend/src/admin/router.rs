use std::sync::Arc;

use axum::{
    Router,
    http::{HeaderValue, StatusCode, header},
    response::IntoResponse,
    routing::{any, get, post},
};
use tower_http::{
    services::{ServeDir, ServeFile},
    set_header::SetResponseHeaderLayer,
};

use crate::app::AppState;

use super::{
    auth::{login, logout, session},
    certificates::{create as create_certificate, list as list_certificates},
    health::health,
    info::info,
    registrars::{create, get as get_registrar, list},
};

pub(crate) fn router(state: Arc<AppState>) -> Router {
    let frontend = ServeDir::new(&state.settings.frontend_dist).fallback(ServeFile::new(format!(
        "{}/index.html",
        state.settings.frontend_dist
    )));

    Router::new()
        .route("/api/health", get(health))
        .route("/api/auth/login", post(login))
        .route("/api/auth/session", get(session))
        .route("/api/auth/logout", post(logout))
        .route("/api/info", get(info))
        .route("/api/registrars", get(list).post(create))
        .route("/api/registrars/{id}", get(get_registrar))
        .route(
            "/api/registrars/{id}/certificates",
            get(list_certificates).post(create_certificate),
        )
        .route("/api/{*path}", any(api_not_found))
        .fallback_service(frontend)
        .layer(SetResponseHeaderLayer::if_not_present(
            header::X_CONTENT_TYPE_OPTIONS,
            HeaderValue::from_static("nosniff"),
        ))
        .layer(SetResponseHeaderLayer::if_not_present(
            header::X_FRAME_OPTIONS,
            HeaderValue::from_static("DENY"),
        ))
        .layer(SetResponseHeaderLayer::if_not_present(
            header::REFERRER_POLICY,
            HeaderValue::from_static("strict-origin-when-cross-origin"),
        ))
        .layer(SetResponseHeaderLayer::if_not_present(
            header::CONTENT_SECURITY_POLICY,
            HeaderValue::from_static(
                "default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data:; font-src 'self' data:; connect-src 'self'; object-src 'none'; base-uri 'self'; frame-ancestors 'none'",
            ),
        ))
        .layer(SetResponseHeaderLayer::if_not_present(
            header::STRICT_TRANSPORT_SECURITY,
            HeaderValue::from_static("max-age=31536000"),
        ))
        .with_state(state)
}

async fn api_not_found() -> impl IntoResponse {
    StatusCode::NOT_FOUND
}
