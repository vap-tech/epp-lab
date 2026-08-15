mod admin;
mod app;
mod application;
#[allow(dead_code)]
mod application_domain;
mod config;
pub mod domain;
pub mod epp;
mod observability;
mod registry;
mod security;
mod storage;
mod tls;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tokio_rustls::rustls::crypto::aws_lc_rs::default_provider()
        .install_default()
        .map_err(|_| anyhow::anyhow!("failed to install rustls crypto provider"))?;
    observability::init();
    let settings = config::Settings::from_env()?;
    let state = app::build_state(settings.clone()).await?;
    let tls_acceptor = tls::load_acceptor(
        std::path::Path::new(&settings.epp_tls_cert),
        std::path::Path::new(&settings.epp_tls_key),
        std::path::Path::new(&settings.epp_client_ca),
    )?;
    let admin_tls = axum_server::tls_rustls::RustlsConfig::from_pem_file(
        &settings.admin_tls_cert,
        &settings.admin_tls_key,
    )
    .await?;
    let admin_handle = axum_server::Handle::new();
    let mut admin_server = Box::pin(
        axum_server::tls_rustls::bind_rustls(settings.admin_bind, admin_tls)
            .handle(admin_handle.clone())
            .serve(admin::router(state.clone()).into_make_service()),
    );
    let (shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);
    let mut epp_task = tokio::spawn(epp::run(
        epp::TcpSettings {
            bind: settings.epp_bind,
            frame_limits: epp::framing::FrameLimits {
                max_frame_size: settings.epp_max_frame_size,
                read_timeout: settings.epp_read_timeout,
                write_timeout: settings.epp_write_timeout,
            },
            tls_handshake_timeout: settings.epp_tls_handshake_timeout,
            idle_timeout: settings.epp_idle_timeout,
            object_uris: settings.epp_object_uris.clone(),
            extension_uris: settings.epp_extension_uris.clone(),
            keepalive_idle: settings.tcp_keepalive_idle,
            keepalive_interval: settings.tcp_keepalive_interval,
            keepalive_retries: settings.tcp_keepalive_retries,
        },
        tls_acceptor,
        state.db.clone(),
        state.extension_registry.clone(),
        state.contact_authinfo_cipher.clone(),
        shutdown_rx,
    ));
    tracing::info!(address = %settings.admin_bind, "admin HTTPS API listening");
    tokio::select! {
        result = &mut admin_server => {
            result?;
        }
        result = &mut epp_task => {
            result??;
        }
        _ = shutdown_signal() => {}
    }
    let _ = shutdown_tx.send(true);
    admin_handle.graceful_shutdown(Some(settings.epp_shutdown_grace_period));
    let _ = admin_server.await;
    match tokio::time::timeout(settings.epp_shutdown_grace_period, epp_task).await {
        Ok(result) => result??,
        Err(_) => tracing::warn!("EPP shutdown grace period elapsed"),
    }
    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = tokio::signal::ctrl_c();
    #[cfg(unix)]
    let terminate = async {
        match tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()) {
            Ok(mut signal) => signal.recv().await,
            Err(error) => {
                tracing::error!(%error, "failed to install SIGTERM handler");
                None
            }
        }
    };
    #[cfg(not(unix))]
    let terminate = std::future::pending::<Option<()>>();

    tokio::select! {
        result = ctrl_c => {
            if let Err(error) = result { tracing::error!(%error, "failed to receive Ctrl+C"); }
        }
        _ = terminate => {}
    }
    tracing::info!("shutdown signal received");
}
