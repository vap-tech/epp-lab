use std::{
    io,
    net::SocketAddr,
    time::{Duration, Instant},
};

use argon2::{Argon2, PasswordHash, PasswordVerifier};
use sha2::{Digest, Sha256};
use socket2::{SockRef, TcpKeepalive};
use sqlx::PgPool;
use tokio::{
    io::AsyncWriteExt,
    net::{TcpListener, TcpStream},
    sync::watch,
};
use tokio_rustls::{TlsAcceptor, server::TlsStream};

use super::framing::{FrameLimits, read_frame};

#[derive(Clone, Debug)]
pub struct TcpSettings {
    pub bind: SocketAddr,
    pub frame_limits: FrameLimits,
    pub tls_handshake_timeout: Duration,
    pub idle_timeout: Option<Duration>,
    pub object_uris: Vec<String>,
    pub extension_uris: Vec<String>,
    pub keepalive_idle: Duration,
    pub keepalive_interval: Duration,
    pub keepalive_retries: u32,
}

pub async fn run(
    settings: TcpSettings,
    acceptor: TlsAcceptor,
    db: PgPool,
    mut shutdown: watch::Receiver<bool>,
) -> io::Result<()> {
    let listener = TcpListener::bind(settings.bind).await?;
    tracing::info!(address = %settings.bind, "EPP TCP listener listening");

    loop {
        tokio::select! {
            result = listener.accept() => {
                let (stream, remote_addr) = result?;
                configure_keepalive(&stream, &settings)?;
                let limits = settings.frame_limits;
                let tls_handshake_timeout = settings.tls_handshake_timeout;
                let idle_timeout = settings.idle_timeout;
                let object_uris = settings.object_uris.clone();
                let extension_uris = settings.extension_uris.clone();
                let acceptor = acceptor.clone();
                let db = db.clone();
                let connection_shutdown = shutdown.clone();
                tokio::spawn(async move {
                    if let Err(error) = handle_connection(stream, remote_addr, limits, tls_handshake_timeout, idle_timeout, object_uris, extension_uris, acceptor, db, connection_shutdown).await {
                        tracing::debug!(%remote_addr, %error, "EPP connection closed");
                    }
                });
            }
            result = shutdown.changed() => {
                if result.is_err() || *shutdown.borrow() { break; }
            }
        }
    }
    tracing::info!("EPP TCP listener stopped");
    Ok(())
}

fn configure_keepalive(stream: &TcpStream, settings: &TcpSettings) -> io::Result<()> {
    let keepalive = TcpKeepalive::new()
        .with_time(settings.keepalive_idle)
        .with_interval(settings.keepalive_interval)
        .with_retries(settings.keepalive_retries);
    SockRef::from(stream).set_tcp_keepalive(&keepalive)
}

#[allow(clippy::too_many_arguments)]
async fn handle_connection(
    stream: TcpStream,
    remote_addr: SocketAddr,
    limits: FrameLimits,
    tls_handshake_timeout: Duration,
    idle_timeout: Option<Duration>,
    object_uris: Vec<String>,
    extension_uris: Vec<String>,
    acceptor: TlsAcceptor,
    db: PgPool,
    mut shutdown: watch::Receiver<bool>,
) -> Result<(), super::framing::FrameError> {
    tracing::debug!(%remote_addr, "EPP TCP connection accepted");
    let mut stream: TlsStream<TcpStream> =
        tokio::time::timeout(tls_handshake_timeout, acceptor.accept(stream))
            .await
            .map_err(|_| super::framing::FrameError::Timeout)?
            .map_err(|error| super::framing::FrameError::Write(io::Error::other(error)))?;
    let peer_certificate = stream
        .get_ref()
        .1
        .peer_certificates()
        .and_then(|certificates| certificates.first())
        .ok_or_else(|| {
            super::framing::FrameError::Write(io::Error::other("client certificate missing"))
        })?;
    let fingerprint = hex::encode(Sha256::digest(peer_certificate.as_ref()));
    let identity = crate::storage::certificate::find_active_identity(&db, &fingerprint)
        .await
        .map_err(|error| super::framing::FrameError::Write(io::Error::other(error)))?;
    let Some(identity) = identity else {
        return Err(super::framing::FrameError::Write(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "client certificate is not registered, active, and valid",
        )));
    };
    let session_id = crate::storage::session::create(
        &db,
        identity.registrar_id,
        identity.certificate_id,
        &remote_addr.to_string(),
    )
    .await
    .map_err(|error| super::framing::FrameError::Write(io::Error::other(error)))?;
    tracing::info!(%remote_addr, registrar_id = %identity.registrar_id, certificate_id = %identity.certificate_id, fingerprint = %identity.fingerprint_sha256, "mTLS identity resolved");
    let mut shutdown_requested = false;
    let mut logout_requested = false;
    let result = async {
        super::protocol::send_greeting(&mut stream, &limits, &object_uris, &extension_uris).await?;
        let mut session_state = crate::registry::session::SessionState::Unauthenticated;
        let mut should_close = false;
        loop {
            let read = async {
                match idle_timeout {
                    Some(timeout) => {
                        tokio::time::timeout(timeout, read_frame(&mut stream, &limits))
                            .await
                            .map_err(|_| super::framing::FrameError::Timeout)?
                    }
                    None => read_frame(&mut stream, &limits).await,
                }
            };
            let payload = tokio::select! {
                result = read => result?,
                result = shutdown.changed() => {
                    if result.is_err() || *shutdown.borrow() {
                        shutdown_requested = true;
                        break;
                    }
                    continue;
                }
            };
            let started = Instant::now();
            let command_name = if payload.windows(5).any(|part| part == b"hello") {
                "hello"
            } else if payload.windows(5).any(|part| part == b"login") {
                "login"
            } else if payload.windows(6).any(|part| part == b"logout") {
                "logout"
            } else {
                "unknown"
            };
            let sv_trid = format!("SIM-{}", uuid::Uuid::new_v4());
            let request_xml = redact_request_xml(&String::from_utf8_lossy(&payload));
            let cl_trid = extract_cl_trid(&payload);
            let transaction_id = crate::storage::session::create_transaction(
                &db,
                session_id,
                Some(identity.registrar_id),
                command_name,
                cl_trid.as_deref(),
                &sv_trid,
                &request_xml,
            )
            .await
            .map_err(|error| super::framing::FrameError::Write(io::Error::other(error)))?;
            let mut response: Option<super::protocol::Response> = None;
            match crate::epp::parser::parse_command(&payload) {
                Ok(crate::epp::parser::EppCommand::Hello) => {
                    super::protocol::send_greeting(
                        &mut stream,
                        &limits,
                        &object_uris,
                        &extension_uris,
                    )
                    .await?;
                }
                Ok(crate::epp::parser::EppCommand::Login(login)) => {
                    if session_state.allows_login() {
                        let services_supported = login
                            .object_uris
                            .iter()
                            .chain(login.extension_uris.iter())
                            .all(|uri| {
                                object_uris
                                    .iter()
                                    .chain(extension_uris.iter())
                                    .any(|supported| supported == uri)
                            });
                        if !services_supported {
                            let response = super::protocol::send_response(
                                &mut stream,
                                &limits,
                                super::protocol::COMMAND_USE_ERROR,
                                "Requested service is not supported",
                                login.cl_trid.as_deref(),
                                &sv_trid,
                            )
                            .await?;
                            let _ = crate::storage::session::finish_transaction(
                                &db,
                                transaction_id,
                                Some(&response.xml),
                                Some(i32::from(response.code)),
                                started.elapsed().as_millis() as i64,
                            )
                            .await;
                            continue;
                        }
                        let authentication = crate::storage::registrar::find_active_by_client_id(
                            &db,
                            &login.client_id,
                        )
                        .await
                        .map_err(|error| {
                            super::framing::FrameError::Write(io::Error::other(error))
                        })?;
                        let valid = authentication.as_ref().is_some_and(|registrar| {
                            registrar.id == identity.registrar_id
                                && PasswordHash::new(&registrar.password_hash)
                                    .ok()
                                    .and_then(|hash| {
                                        Argon2::default()
                                            .verify_password(login.password.as_bytes(), &hash)
                                            .ok()
                                    })
                                    .is_some()
                        });
                        if valid {
                            crate::storage::session::mark_authenticated(&db, session_id)
                                .await
                                .map_err(|error| {
                                    super::framing::FrameError::Write(io::Error::other(error))
                                })?;
                            session_state = crate::registry::session::SessionState::Authenticated {
                                registrar_id: identity.registrar_id,
                            };
                            response = Some(
                                super::protocol::send_response(
                                    &mut stream,
                                    &limits,
                                    super::protocol::SUCCESS,
                                    "Command completed successfully",
                                    login.cl_trid.as_deref(),
                                    &sv_trid,
                                )
                                .await?,
                            );
                        } else {
                            response = Some(
                                super::protocol::send_response(
                                    &mut stream,
                                    &limits,
                                    super::protocol::AUTH_ERROR,
                                    "Authentication error",
                                    login.cl_trid.as_deref(),
                                    &sv_trid,
                                )
                                .await?,
                            );
                        }
                    } else {
                        response = Some(
                            super::protocol::send_response(
                                &mut stream,
                                &limits,
                                super::protocol::COMMAND_ERROR,
                                "already authenticated",
                                login.cl_trid.as_deref(),
                                &sv_trid,
                            )
                            .await?,
                        );
                    }
                }
                Ok(crate::epp::parser::EppCommand::Logout) => {
                    logout_requested = true;
                    if session_state.allows_logout() {
                        response = Some(
                            super::protocol::send_response(
                                &mut stream,
                                &limits,
                                super::protocol::SUCCESS,
                                "Command completed successfully",
                                None,
                                &sv_trid,
                            )
                            .await?,
                        );
                    } else {
                        super::protocol::send_response(
                            &mut stream,
                            &limits,
                            super::protocol::COMMAND_ERROR,
                            "not authenticated",
                            None,
                            &sv_trid,
                        )
                        .await?;
                    }
                    should_close = true;
                }
                Err(crate::epp::parser::ParseError::Unsupported) => {
                    response = Some(
                        super::protocol::send_response(
                            &mut stream,
                            &limits,
                            super::protocol::COMMAND_NOT_SUPPORTED,
                            "Command not supported",
                            cl_trid.as_deref(),
                            &sv_trid,
                        )
                        .await?,
                    );
                }
                Err(_) => {
                    response = Some(
                        super::protocol::send_response(
                            &mut stream,
                            &limits,
                            2001,
                            "Command syntax error",
                            None,
                            &sv_trid,
                        )
                        .await?,
                    );
                }
            }
            if let Some(response) = response {
                let _ = crate::storage::session::finish_transaction(
                    &db,
                    transaction_id,
                    Some(&response.xml),
                    Some(i32::from(response.code)),
                    started.elapsed().as_millis() as i64,
                )
                .await;
            }
            if should_close {
                break;
            }
            let _ = &mut session_state;
        }
        Ok::<(), super::framing::FrameError>(())
    }
    .await;
    let reason = if shutdown_requested {
        "server_shutdown"
    } else if logout_requested {
        "client_logout"
    } else if result.is_ok() {
        "client_closed"
    } else {
        "protocol_error"
    };
    let _ = crate::storage::session::disconnect(&db, session_id, reason).await;
    stream
        .shutdown()
        .await
        .map_err(super::framing::FrameError::Write)?;
    result
}

fn extract_cl_trid(payload: &[u8]) -> Option<String> {
    let start = payload
        .windows(8)
        .position(|window| window == b"<clTRID>")?
        + 8;
    let end = payload[start..]
        .windows(9)
        .position(|window| window == b"</clTRID>")?;
    String::from_utf8(payload[start..start + end].to_vec()).ok()
}

fn redact_request_xml(xml: &str) -> String {
    let mut result = String::with_capacity(xml.len());
    let mut rest = xml;
    while let Some(start) = rest.find("<pw>") {
        result.push_str(&rest[..start + 4]);
        let value_start = start + 4;
        let Some(end) = rest[value_start..].find("</pw>") else {
            result.push_str("[REDACTED]");
            return result;
        };
        result.push_str("[REDACTED]");
        result.push_str("</pw>");
        rest = &rest[value_start + end + 5..];
    }
    result.push_str(rest);
    result
}

#[cfg(test)]
mod logging_tests {
    use super::{extract_cl_trid, redact_request_xml};

    #[test]
    fn extracts_client_transaction_id() {
        assert_eq!(
            extract_cl_trid(b"<clTRID>abc-1</clTRID>"),
            Some("abc-1".into())
        );
    }

    #[test]
    fn redacts_password_before_persistence() {
        assert_eq!(redact_request_xml("<pw>secret</pw>"), "<pw>[REDACTED]</pw>");
    }
}
