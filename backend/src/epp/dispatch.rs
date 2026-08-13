use super::parser::{ParseError, ParsedCommand};
use sqlx::PgPool;
use tokio::net::TcpStream;
use tokio_rustls::server::TlsStream;

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
