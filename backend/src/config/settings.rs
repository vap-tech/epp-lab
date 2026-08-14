use std::{env, net::SocketAddr, time::Duration};

use anyhow::{Context, Result};

#[derive(Clone, Debug)]
pub struct Settings {
    pub app_env: String,
    pub admin_bind: SocketAddr,
    pub admin_tls_cert: String,
    pub admin_tls_key: String,
    pub frontend_dist: String,
    pub epp_bind: SocketAddr,
    pub database_url: String,
    pub epp_tls_cert: String,
    pub epp_tls_key: String,
    pub epp_client_ca: String,
    pub contact_authinfo_key_hex: Option<String>,
    pub epp_read_timeout: Duration,
    pub epp_tls_handshake_timeout: Duration,
    pub epp_write_timeout: Duration,
    pub epp_idle_timeout: Option<Duration>,
    pub epp_shutdown_grace_period: Duration,
    pub epp_max_frame_size: u32,
    pub epp_object_uris: Vec<String>,
    pub epp_extension_uris: Vec<String>,
    pub tcp_keepalive_idle: Duration,
    pub tcp_keepalive_interval: Duration,
    pub tcp_keepalive_retries: u32,
}

impl Settings {
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            app_env: value("APP_ENV", "development"),
            admin_bind: value("ADMIN_BIND", "0.0.0.0:8080")
                .parse()
                .context("ADMIN_BIND must be a valid socket address")?,
            admin_tls_cert: env::var("ADMIN_TLS_CERT").context("ADMIN_TLS_CERT must be set")?,
            admin_tls_key: env::var("ADMIN_TLS_KEY").context("ADMIN_TLS_KEY must be set")?,
            frontend_dist: value("FRONTEND_DIST", "frontend/dist"),
            epp_bind: value("EPP_BIND", "0.0.0.0:700")
                .parse()
                .context("EPP_BIND must be a valid socket address")?,
            database_url: env::var("DATABASE_URL").context("DATABASE_URL must be set")?,
            epp_tls_cert: env::var("EPP_TLS_CERT").context("EPP_TLS_CERT must be set")?,
            epp_tls_key: env::var("EPP_TLS_KEY").context("EPP_TLS_KEY must be set")?,
            epp_client_ca: env::var("EPP_CLIENT_CA").context("EPP_CLIENT_CA must be set")?,
            contact_authinfo_key_hex: env::var("CONTACT_AUTHINFO_KEY_HEX").ok(),
            epp_read_timeout: seconds("EPP_READ_TIMEOUT", 300),
            epp_tls_handshake_timeout: seconds("EPP_TLS_HANDSHAKE_TIMEOUT", 10),
            epp_write_timeout: seconds("EPP_WRITE_TIMEOUT", 30),
            epp_idle_timeout: optional_seconds("EPP_IDLE_TIMEOUT"),
            epp_shutdown_grace_period: seconds("EPP_SHUTDOWN_GRACE_PERIOD", 10),
            epp_max_frame_size: value("EPP_MAX_FRAME_SIZE", "1048576")
                .parse()
                .context("EPP_MAX_FRAME_SIZE must be an integer")?,
            epp_object_uris: object_uris(),
            epp_extension_uris: list("EPP_EXTENSION_URIS", ""),
            tcp_keepalive_idle: seconds("TCP_KEEPALIVE_IDLE", 60),
            tcp_keepalive_interval: seconds("TCP_KEEPALIVE_INTERVAL", 30),
            tcp_keepalive_retries: value("TCP_KEEPALIVE_RETRIES", "5")
                .parse()
                .context("TCP_KEEPALIVE_RETRIES must be an integer")?,
        })
    }
}

fn object_uris() -> Vec<String> {
    const CONTACT_URI: &str = "urn:ietf:params:xml:ns:contact-1.0";
    let mut uris = list("EPP_OBJECT_URIS", "urn:ietf:params:xml:ns:domain-1.0");
    if !uris.iter().any(|uri| uri == CONTACT_URI) {
        uris.push(CONTACT_URI.to_owned());
    }
    uris
}

fn value(name: &str, default: &str) -> String {
    env::var(name).unwrap_or_else(|_| default.to_owned())
}

fn seconds(name: &str, default: u64) -> Duration {
    value(name, &default.to_string())
        .parse::<u64>()
        .map(Duration::from_secs)
        .unwrap_or_else(|_| Duration::from_secs(default))
}

fn optional_seconds(name: &str) -> Option<Duration> {
    env::var(name)
        .ok()?
        .parse::<u64>()
        .ok()
        .map(Duration::from_secs)
}

fn list(name: &str, default: &str) -> Vec<String> {
    value(name, default)
        .split(',')
        .map(str::trim)
        .filter(|item| !item.is_empty())
        .map(str::to_owned)
        .collect()
}
