use super::parser::{ParseError, ParsedCommand};
use sqlx::PgPool;
use tokio::net::TcpStream;
use tokio_rustls::server::TlsStream;

pub(crate) struct LogoutResult {
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
