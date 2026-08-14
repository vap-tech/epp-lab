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
    let statuses = crate::application::effective_contact_statuses(&contact.statuses, false);
    match super::protocol::send_contact_info(
        stream, limits, &contact, &statuses, &auth, cl_trid, sv_trid,
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
pub(crate) async fn execute_contact_delete(
    stream: &mut TlsStream<TcpStream>,
    limits: &super::framing::FrameLimits,
    db: &PgPool,
    transaction_id: uuid::Uuid,
    command: &super::parser::ContactDeleteCommand,
    registrar_id: uuid::Uuid,
    cl_trid: Option<&str>,
    sv_trid: &str,
) -> Result<super::protocol::Response, super::framing::FrameError> {
    let Some(identity) = crate::storage::contact::find_identity_by_roid(db, &command.id)
        .await
        .map_err(|e| super::framing::FrameError::Write(std::io::Error::other(e)))?
    else {
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
    if crate::storage::contact::has_client_status(db, identity.id, "clientDeleteProhibited")
        .await
        .map_err(|error| super::framing::FrameError::Write(std::io::Error::other(error)))?
    {
        return super::protocol::send_response(
            stream,
            limits,
            2304,
            "object status prohibits operation",
            cl_trid,
            sv_trid,
        )
        .await;
    }
    match crate::storage::contact::delete(db, identity.id).await {
        Ok(true) => super::protocol::send_contact_delete(stream, limits, cl_trid, sv_trid).await,
        Ok(false) => {
            super::protocol::send_response(
                stream,
                limits,
                2303,
                "object does not exist",
                cl_trid,
                sv_trid,
            )
            .await
        }
        Err(error) => {
            let _ = crate::storage::session::mark_delivery_failed(
                db,
                transaction_id,
                &error.to_string(),
            )
            .await;
            Err(super::framing::FrameError::Write(std::io::Error::other(
                error,
            )))
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) async fn execute_contact_update(
    stream: &mut TlsStream<TcpStream>,
    limits: &super::framing::FrameLimits,
    db: &PgPool,
    _transaction_id: uuid::Uuid,
    cipher: Option<&dyn crate::security::SecretCipher>,
    command: &super::parser::ContactUpdateCommand,
    registrar_id: uuid::Uuid,
    cl_trid: Option<&str>,
    sv_trid: &str,
) -> Result<super::protocol::Response, super::framing::FrameError> {
    let allowed = |status: &str| {
        matches!(
            status,
            "clientDeleteProhibited" | "clientTransferProhibited" | "clientUpdateProhibited"
        )
    };
    if command
        .add_statuses
        .iter()
        .chain(command.rem_statuses.iter())
        .any(|s| !allowed(s))
    {
        return super::protocol::send_response(
            stream,
            limits,
            super::protocol::COMMAND_USE_ERROR,
            "status is not client-managed",
            cl_trid,
            sv_trid,
        )
        .await;
    }
    if command
        .add_statuses
        .iter()
        .any(|status| command.rem_statuses.contains(status))
    {
        return super::protocol::send_response(
            stream,
            limits,
            super::protocol::COMMAND_USE_ERROR,
            "status cannot be added and removed in the same command",
            cl_trid,
            sv_trid,
        )
        .await;
    }
    if let crate::domain::contact::Patch::Set(email) = &command.chg_email
        && crate::domain::contact::EmailAddress::parse(email).is_err()
    {
        return super::protocol::send_response(
            stream,
            limits,
            super::protocol::COMMAND_USE_ERROR,
            "invalid email address",
            cl_trid,
            sv_trid,
        )
        .await;
    }
    if let crate::domain::contact::Patch::Set(auth_info) = &command.chg_auth_info
        && auth_info.is_empty()
    {
        return super::protocol::send_response(
            stream,
            limits,
            super::protocol::COMMAND_USE_ERROR,
            "authInfo cannot be empty",
            cl_trid,
            sv_trid,
        )
        .await;
    }
    let Some(identity) = crate::storage::contact::find_identity_by_roid(db, &command.id)
        .await
        .map_err(|e| super::framing::FrameError::Write(std::io::Error::other(e)))?
    else {
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
    if crate::storage::contact::has_client_status(db, identity.id, "clientUpdateProhibited")
        .await
        .map_err(|error| super::framing::FrameError::Write(std::io::Error::other(error)))?
    {
        return super::protocol::send_response(
            stream,
            limits,
            2304,
            "object status prohibits operation",
            cl_trid,
            sv_trid,
        )
        .await;
    }
    let auth = match &command.chg_auth_info {
        crate::domain::contact::Patch::Set(value) => {
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
            Some(
                cipher
                    .encrypt(value.as_bytes())
                    .map_err(|e| super::framing::FrameError::Write(std::io::Error::other(e)))?,
            )
        }
        _ => None,
    };
    let email = match &command.chg_email {
        crate::domain::contact::Patch::Set(value) => Some(value.as_str()),
        _ => None,
    };
    let voice = match &command.chg_voice {
        crate::domain::contact::Patch::Set(value) => Some(value.as_str()),
        _ => None,
    };
    let fax = match &command.chg_fax {
        crate::domain::contact::Patch::Set(value) => Some(Some(value.as_str())),
        crate::domain::contact::Patch::Clear => Some(None),
        _ => None,
    };
    let organization = match &command.chg_organization {
        crate::domain::contact::Patch::Set(value) => Some(Some(value.as_str())),
        crate::domain::contact::Patch::Clear => Some(None),
        _ => None,
    };
    let city = match &command.chg_city {
        crate::domain::contact::Patch::Set(value) => Some(value.as_str()),
        _ => None,
    };
    let state_province = match &command.chg_state_province {
        crate::domain::contact::Patch::Set(value) => Some(Some(value.as_str())),
        crate::domain::contact::Patch::Clear => Some(None),
        _ => None,
    };
    let postal_code = match &command.chg_postal_code {
        crate::domain::contact::Patch::Set(value) => Some(Some(value.as_str())),
        crate::domain::contact::Patch::Clear => Some(None),
        _ => None,
    };
    let country_code = match &command.chg_country_code {
        crate::domain::contact::Patch::Set(value) => Some(value.as_str()),
        _ => None,
    };
    let disclose_flag = match &command.chg_disclose {
        crate::domain::contact::Patch::Set(flag) => Some(match flag.as_str() {
            "0" => "private",
            "1" => "public",
            _ => {
                return super::protocol::send_response(
                    stream,
                    limits,
                    super::protocol::COMMAND_USE_ERROR,
                    "invalid disclose flag",
                    cl_trid,
                    sv_trid,
                )
                .await;
            }
        }),
        _ => None,
    };
    let disclosure_fields = if command.chg_disclose_fields.is_empty() {
        None
    } else {
        let allowed = ["name", "organization", "address", "voice", "fax", "email"];
        if command
            .chg_disclose_fields
            .iter()
            .any(|field| !allowed.contains(&field.as_str()))
        {
            return super::protocol::send_response(
                stream,
                limits,
                super::protocol::COMMAND_USE_ERROR,
                "invalid disclose field",
                cl_trid,
                sv_trid,
            )
            .await;
        }
        Some(
            command
                .chg_disclose_fields
                .iter()
                .map(String::as_str)
                .collect::<Vec<_>>(),
        )
    };
    let streets = command
        .chg_streets
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>();
    let add_statuses = command
        .add_statuses
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>();
    let remove_statuses = command
        .rem_statuses
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>();
    crate::storage::contact::apply_update(
        db,
        crate::storage::contact::ContactUpdate {
            id: identity.id,
            auth_info_ciphertext: auth.as_deref(),
            email,
            voice,
            fax,
            organization,
            city,
            state_province,
            postal_code,
            country_code,
            streets: &streets,
            add_statuses: &add_statuses,
            remove_statuses: &remove_statuses,
            disclose_flag,
            disclosure_fields: disclosure_fields.as_deref(),
        },
    )
    .await
    .map_err(|e| super::framing::FrameError::Write(std::io::Error::other(e)))?;
    super::protocol::send_response(
        stream,
        limits,
        super::protocol::SUCCESS,
        "Command completed successfully",
        cl_trid,
        sv_trid,
    )
    .await
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
