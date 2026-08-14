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
    epp::{session as epp_session, sessions, transaction, transactions},
    extensions::{catalog, list_zone_extensions, set_zone_extension},
    health::health,
    info::info,
    registrars::{create, get as get_registrar, list},
    zones::{
        create as create_zone, get as get_zone, list as list_zones, update as update_zone,
        update_contact_policy,
    },
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
        .route("/api/epp/sessions", get(sessions))
        .route("/api/epp/sessions/{id}", get(epp_session))
        .route("/api/epp/transactions", get(transactions))
        .route("/api/epp/transactions/{id}", get(transaction))
        .route("/api/registrars", get(list).post(create))
        .route("/api/zones", get(list_zones).post(create_zone))
        .route("/api/zones/{id}", get(get_zone).patch(update_zone))
        .route(
            "/api/zones/{id}/contact-policy",
            axum::routing::patch(update_contact_policy),
        )
        .route("/api/extensions/catalog", get(catalog))
        .route("/api/zones/{id}/extensions", get(list_zone_extensions))
        .route(
            "/api/zones/{id}/extensions/{extension_key}",
            axum::routing::patch(set_zone_extension),
        )
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

#[cfg(test)]
mod tests {
    use std::{net::SocketAddr, sync::Arc, time::Duration};

    use axum::{
        body::Body,
        http::{Request, StatusCode},
        response::Response,
    };
    use sqlx::PgPool;
    use tower::ServiceExt;

    use super::router;
    use crate::{app::AppState, config::Settings, domain::extension::ExtensionRegistry};

    fn settings() -> Settings {
        Settings {
            app_env: "test".to_owned(),
            admin_bind: "127.0.0.1:0".parse::<SocketAddr>().unwrap(),
            admin_tls_cert: String::new(),
            admin_tls_key: String::new(),
            frontend_dist: ".".to_owned(),
            epp_bind: "127.0.0.1:0".parse::<SocketAddr>().unwrap(),
            database_url: String::new(),
            epp_tls_cert: String::new(),
            epp_tls_key: String::new(),
            epp_client_ca: String::new(),
            contact_authinfo_key_hex: None,
            epp_read_timeout: Duration::from_secs(1),
            epp_tls_handshake_timeout: Duration::from_secs(1),
            epp_write_timeout: Duration::from_secs(1),
            epp_idle_timeout: None,
            epp_shutdown_grace_period: Duration::from_secs(1),
            epp_max_frame_size: 4096,
            epp_object_uris: Vec::new(),
            epp_extension_uris: Vec::new(),
            tcp_keepalive_idle: Duration::from_secs(1),
            tcp_keepalive_interval: Duration::from_secs(1),
            tcp_keepalive_retries: 1,
        }
    }

    async fn request(pool: PgPool, method: &str, uri: &str) -> Response {
        let state = Arc::new(AppState {
            db: pool,
            settings: settings(),
            extension_registry: Arc::new(ExtensionRegistry::empty()),
            contact_authinfo_cipher: None,
        });
        router(state)
            .oneshot(
                Request::builder()
                    .method(method)
                    .uri(uri)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap()
    }

    #[ignore = "requires PostgreSQL; run through just test-with-db"]
    #[sqlx::test(migrations = "../backend/migrations")]
    async fn protects_zone_api_and_keeps_unknown_api_as_json_404(pool: PgPool) {
        let response = request(pool.clone(), "GET", "/api/zones").await;
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        assert_eq!(response.headers()["x-content-type-options"], "nosniff");
        assert_eq!(response.headers()["x-frame-options"], "DENY");

        let response = request(pool, "GET", "/api/does-not-exist").await;
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        assert!(response.headers().get("content-type").is_none());
    }
}
