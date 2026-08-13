mod admin;
mod app;
mod config;
pub mod epp;
mod observability;
mod registry;
mod storage;
mod tls;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    observability::init();
    let settings = config::Settings::from_env()?;
    let state = app::build_state(settings.clone()).await?;
    let tls_acceptor = tls::load_acceptor(
        std::path::Path::new(&settings.epp_tls_cert),
        std::path::Path::new(&settings.epp_tls_key),
        std::path::Path::new(&settings.epp_client_ca),
    )?;
    let listener = tokio::net::TcpListener::bind(settings.admin_bind).await?;
    let (shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);
    let epp_task = tokio::spawn(epp::run(
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
        shutdown_rx,
    ));
    tracing::info!(address = %settings.admin_bind, "admin API listening");
    tokio::select! {
        result = axum::serve(listener, admin::router(state)).with_graceful_shutdown(shutdown_signal()) => {
            result?;
        }
        result = epp_task => {
            result??;
        }
    }
    let _ = shutdown_tx.send(true);
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
