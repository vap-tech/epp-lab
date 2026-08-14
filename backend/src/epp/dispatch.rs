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

fn services_supported(
    requested_objects: &[String],
    requested_extensions: &[String],
    supported_objects: &[String],
    supported_extensions: &[String],
) -> bool {
    requested_objects
        .iter()
        .chain(requested_extensions.iter())
        .all(|uri| {
            supported_objects
                .iter()
                .chain(supported_extensions.iter())
                .any(|supported| supported == uri)
        })
}

fn credentials_valid(
    registrar: Option<&crate::storage::registrar::AuthenticationRow>,
    expected_registrar_id: uuid::Uuid,
    password: &str,
) -> bool {
    registrar.is_some_and(|registrar| {
        registrar.id == expected_registrar_id
            && PasswordHash::new(&registrar.password_hash)
                .ok()
                .and_then(|hash| {
                    Argon2::default()
                        .verify_password(password.as_bytes(), &hash)
                        .ok()
                })
                .is_some()
    })
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
        persisted_xml: greeting.clone(),
        xml: greeting,
        code: None,
    })
}

#[allow(clippy::too_many_arguments)]
pub(crate) async fn execute_contact_check(
    stream: &mut TlsStream<TcpStream>,
    limits: &super::framing::FrameLimits,
    db: &PgPool,
    transaction_id: uuid::Uuid,
    state: &crate::registry::session::SessionState,
    command: &super::parser::ContactCheckCommand,
    cl_trid: Option<&str>,
    sv_trid: &str,
) -> Result<super::protocol::Response, super::framing::FrameError> {
    if !matches!(
        state,
        crate::registry::session::SessionState::Authenticated { .. }
    ) {
        return super::protocol::send_response(
            stream,
            limits,
            super::protocol::COMMAND_ERROR,
            "not authenticated",
            cl_trid,
            sv_trid,
        )
        .await;
    }
    let mut results = Vec::with_capacity(command.ids.len());
    for id in &command.ids {
        let available = crate::application::check_contact(db, id)
            .await
            .map_err(|error| super::framing::FrameError::Write(std::io::Error::other(error)))?
            .available;
        results.push((id.clone(), available));
    }
    match super::protocol::send_contact_check(stream, limits, &results, cl_trid, sv_trid).await {
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

#[allow(clippy::too_many_arguments)]
pub(crate) async fn execute_contact_create(
    stream: &mut TlsStream<TcpStream>,
    limits: &super::framing::FrameLimits,
    db: &PgPool,
    transaction_id: uuid::Uuid,
    cipher: Option<&dyn crate::security::SecretCipher>,
    command: &super::parser::ContactCreateCommand,
    registrar_id: uuid::Uuid,
    cl_trid: Option<&str>,
    sv_trid: &str,
) -> Result<super::protocol::Response, super::framing::FrameError> {
    let Some(cipher) = cipher else {
        return super::protocol::send_response(
            stream,
            limits,
            super::protocol::COMMAND_ERROR,
            "authInfo encryption is not configured",
            cl_trid,
            sv_trid,
        )
        .await;
    };
    let contact = crate::application::prepare_contact_create(
        command,
        registrar_id,
        cipher,
        chrono::Utc::now(),
    )
    .map_err(|error| super::framing::FrameError::Write(std::io::Error::other(error)))?;
    crate::storage::contact::create(db, &contact)
        .await
        .map_err(|error| super::framing::FrameError::Write(std::io::Error::other(error)))?;
    let created_at = contact.created_at.to_rfc3339();
    match super::protocol::send_contact_create(
        stream,
        limits,
        contact.roid.as_str(),
        &created_at,
        cl_trid,
        sv_trid,
    )
    .await
    {
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

#[allow(clippy::too_many_arguments)]
pub(crate) async fn execute_contact_info(
    stream: &mut TlsStream<TcpStream>,
    limits: &super::framing::FrameLimits,
    db: &PgPool,
    transaction_id: uuid::Uuid,
    cipher: Option<&dyn crate::security::SecretCipher>,
    command: &super::parser::ContactInfoCommand,
    registrar_id: uuid::Uuid,
    cl_trid: Option<&str>,
    sv_trid: &str,
) -> Result<super::protocol::Response, super::framing::FrameError> {
    let Some(cipher) = cipher else {
        return super::protocol::send_response(
            stream,
            limits,
            super::protocol::COMMAND_ERROR,
            "authInfo encryption is not configured",
            cl_trid,
            sv_trid,
        )
        .await;
    };
    let id = crate::storage::contact::find_identity_by_roid(db, &command.id)
        .await
        .map_err(|e| super::framing::FrameError::Write(std::io::Error::other(e)))?;
    let Some(identity) = id else {
        return super::protocol::send_response(
            stream,
            limits,
            2303,
            "object does not exist",
            cl_trid,
            sv_trid,
        )
        .await;
    };
    if identity.sponsoring_registrar_id != registrar_id {
        return super::protocol::send_response(
            stream,
            limits,
            2201,
            "authorization error",
            cl_trid,
            sv_trid,
        )
        .await;
    }
    let contact = crate::storage::contact::find_detail(db, identity.id)
        .await
        .map_err(|e| super::framing::FrameError::Write(std::io::Error::other(e)))?
        .ok_or_else(|| {
            super::framing::FrameError::Write(std::io::Error::other("contact disappeared"))
        })?;
    let auth = cipher
        .decrypt(&identity.auth_info_ciphertext)
        .map_err(|e| super::framing::FrameError::Write(std::io::Error::other(e)))?;
    let auth = String::from_utf8(auth)
        .map_err(|e| super::framing::FrameError::Write(std::io::Error::other(e)))?;
    match super::protocol::send_contact_info(stream, limits, &contact, &auth, cl_trid, sv_trid)
        .await
    {
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
        let services_supported = services_supported(
            &login.object_uris,
            &login.extension_uris,
            object_uris,
            extension_uris,
        );
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
            let valid = credentials_valid(authentication.as_ref(), registrar_id, &login.password);
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

    #[test]
    fn validates_requested_services() {
        let requested = vec!["urn:ietf:params:xml:ns:domain-1.0".to_owned()];
        let supported = requested.clone();
        assert!(services_supported(&requested, &[], &supported, &[]));
        assert!(!services_supported(
            &["urn:example:unsupported".to_owned()],
            &[],
            &supported,
            &[]
        ));
    }

    #[test]
    fn rejects_missing_or_invalid_credentials() {
        let registrar_id = uuid::Uuid::new_v4();
        assert!(!credentials_valid(None, registrar_id, "secret"));
        let row = crate::storage::registrar::AuthenticationRow {
            id: registrar_id,
            password_hash: "not-a-password-hash".to_owned(),
        };
        assert!(!credentials_valid(Some(&row), registrar_id, "secret"));
        assert!(!credentials_valid(
            Some(&row),
            uuid::Uuid::new_v4(),
            "secret"
        ));
    }

    #[test]
    fn accepts_matching_registrar_and_password() {
        use argon2::password_hash::{PasswordHasher, SaltString, rand_core::OsRng};

        let registrar_id = uuid::Uuid::new_v4();
        let salt = SaltString::generate(&mut OsRng);
        let password_hash = Argon2::default()
            .hash_password(b"secret", &salt)
            .unwrap()
            .to_string();
        let row = crate::storage::registrar::AuthenticationRow {
            id: registrar_id,
            password_hash,
        };
        assert!(credentials_valid(Some(&row), registrar_id, "secret"));
        assert!(!credentials_valid(Some(&row), registrar_id, "wrong"));
    }
}
