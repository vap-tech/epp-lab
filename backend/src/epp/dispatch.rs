use super::parser::{ParseError, ParsedCommand};
use argon2::{Argon2, PasswordHash, PasswordVerifier};
use sqlx::PgPool;
use tokio::net::TcpStream;
use tokio_rustls::server::TlsStream;

pub(crate) struct LogoutResult {
    pub response: super::protocol::Response,
    pub authenticated: bool,
}

pub(crate) struct LoginResult {
    pub response: super::protocol::Response,
    pub authenticated: bool,
}

#[allow(clippy::too_many_arguments)]
pub(crate) async fn execute_hello(
    stream: &mut TlsStream<TcpStream>,
    limits: &super::framing::FrameLimits,
    object_uris: &[String],
    extension_uris: &[String],
    db: &PgPool,
    transaction_id: uuid::Uuid,
) -> Result<super::protocol::Response, super::framing::FrameError> {
    let greeting =
        match super::protocol::send_greeting(stream, limits, object_uris, extension_uris).await {
            Ok(greeting) => greeting,
            Err(error) => {
                let _ = crate::storage::session::mark_delivery_failed(
                    db,
                    transaction_id,
                    &error.to_string(),
                )
                .await;
                return Err(error);
            }
        };
    Ok(super::protocol::Response {
        xml: greeting,
        code: None,
    })
}

#[allow(clippy::too_many_arguments)]
pub(crate) async fn execute_logout(
    stream: &mut TlsStream<TcpStream>,
    limits: &super::framing::FrameLimits,
    db: &PgPool,
    transaction_id: uuid::Uuid,
    state: &crate::registry::session::SessionState,
    sv_trid: &str,
    cl_trid: Option<&str>,
) -> Result<LogoutResult, super::framing::FrameError> {
    let (code, message, authenticated) = if state.allows_logout() {
        (
            super::protocol::SUCCESS,
            "Command completed successfully",
            true,
        )
    } else {
        (super::protocol::COMMAND_ERROR, "not authenticated", false)
    };
    let response =
        match super::protocol::send_response(stream, limits, code, message, cl_trid, sv_trid).await
        {
            Ok(response) => response,
            Err(error) => {
                let _ = crate::storage::session::mark_delivery_failed(
                    db,
                    transaction_id,
                    &error.to_string(),
                )
                .await;
                return Err(error);
            }
        };
    Ok(LogoutResult {
        response,
        authenticated,
    })
}

#[allow(clippy::too_many_arguments)]
pub(crate) async fn execute_login(
    stream: &mut TlsStream<TcpStream>,
    limits: &super::framing::FrameLimits,
    db: &PgPool,
    transaction_id: uuid::Uuid,
    session_id: uuid::Uuid,
    state: &crate::registry::session::SessionState,
    login: &super::parser::LoginCommand,
    registrar_id: uuid::Uuid,
    object_uris: &[String],
    extension_uris: &[String],
    sv_trid: &str,
) -> Result<LoginResult, super::framing::FrameError> {
    let (code, message, authenticated) = if !state.allows_login() {
        (
            super::protocol::COMMAND_ERROR,
            "already authenticated",
            false,
        )
    } else {
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
            (
                super::protocol::COMMAND_USE_ERROR,
                "Requested service is not supported",
                false,
            )
        } else {
            let authentication =
                crate::storage::registrar::find_active_by_client_id(db, &login.client_id)
                    .await
                    .map_err(|error| {
                        super::framing::FrameError::Write(std::io::Error::other(error))
                    })?;
            let valid = authentication.as_ref().is_some_and(|registrar| {
                registrar.id == registrar_id
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
                crate::storage::session::mark_authenticated(db, session_id)
                    .await
                    .map_err(|error| {
                        super::framing::FrameError::Write(std::io::Error::other(error))
                    })?;
                (
                    super::protocol::SUCCESS,
                    "Command completed successfully",
                    true,
                )
            } else {
                (super::protocol::AUTH_ERROR, "Authentication error", false)
            }
        }
    };
    let response = match super::protocol::send_response(
        stream,
        limits,
        code,
        message,
        login.cl_trid.as_deref(),
        sv_trid,
    )
    .await
    {
        Ok(response) => response,
        Err(error) => {
            let _ = crate::storage::session::mark_delivery_failed(
                db,
                transaction_id,
                &error.to_string(),
            )
            .await;
            return Err(error);
        }
    };
    Ok(LoginResult {
        response,
        authenticated,
    })
}

#[allow(clippy::too_many_arguments)]
pub(crate) async fn execute_parse_error(
    stream: &mut TlsStream<TcpStream>,
    limits: &super::framing::FrameLimits,
    db: &PgPool,
    transaction_id: uuid::Uuid,
    error: &ParseError,
    cl_trid: Option<&str>,
    sv_trid: &str,
) -> Result<super::protocol::Response, super::framing::FrameError> {
    let (code, message) = match error {
        ParseError::Unsupported => (
            super::protocol::COMMAND_NOT_SUPPORTED,
            "Command not supported",
        ),
        _ => (2001, "Command syntax error"),
    };
    match super::protocol::send_response(stream, limits, code, message, cl_trid, sv_trid).await {
        Ok(response) => Ok(response),
        Err(error) => {
            let _ = crate::storage::session::mark_delivery_failed(
                db,
                transaction_id,
                &error.to_string(),
            )
            .await;
            Err(error)
        }
    }
}

pub(crate) fn command_name(parsed: &Result<ParsedCommand, ParseError>) -> &'static str {
    match parsed {
        Ok(parsed) => parsed.name(),
        Err(ParseError::Unsupported) => "unsupported",
        Err(_) => "invalid",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_parse_results_for_logging() {
        let parsed = Err(ParseError::Unsupported);
        assert_eq!(command_name(&parsed), "unsupported");
    }
}
