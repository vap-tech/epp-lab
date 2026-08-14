use chrono::Utc;
use tokio::io::{AsyncRead, AsyncWrite};

use super::framing::{FrameError, FrameLimits, write_frame};

pub(crate) const SUCCESS: u16 = 1000;
pub(crate) const AUTH_ERROR: u16 = 2200;
pub(crate) const COMMAND_ERROR: u16 = 2001;
pub(crate) const COMMAND_NOT_SUPPORTED: u16 = 2000;
pub(crate) const COMMAND_USE_ERROR: u16 = 2102;

pub(crate) struct Response {
    #[allow(dead_code)]
    pub xml: String,
    /// XML safe to persist in transaction history. It may differ from `xml`
    /// when a response contains a recoverable secret.
    pub persisted_xml: String,
    #[allow(dead_code)]
    pub code: Option<u16>,
}

pub(crate) async fn send_response<S>(
    stream: &mut S,
    limits: &FrameLimits,
    code: u16,
    message: &str,
    cl_trid: Option<&str>,
    sv_trid: &str,
) -> Result<Response, FrameError>
where
    S: AsyncWrite + Unpin,
{
    let trid = cl_trid
        .map(|value| format!("<clTRID>{}</clTRID>", escape_xml(value)))
        .unwrap_or_default();
    let response = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?><epp xmlns="urn:ietf:params:xml:ns:epp-1.0"><response><result code="{code}"><msg>{message}</msg></result>{trid}<svTRID>{sv_trid}</svTRID></response></epp>"#,
    );
    write_frame(stream, response.as_bytes(), limits).await?;
    Ok(Response {
        persisted_xml: response.clone(),
        xml: response,
        code: Some(code),
    })
}

fn escape_xml(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

pub(crate) async fn send_greeting<S>(
    stream: &mut S,
    limits: &FrameLimits,
    object_uris: &[String],
    extension_uris: &[String],
) -> Result<String, FrameError>
where
    S: AsyncWrite + Unpin,
{
    let objects = object_uris
        .iter()
        .map(|uri| format!("<objURI>{uri}</objURI>"))
        .collect::<String>();
    let extensions = extension_uris
        .iter()
        .map(|uri| format!("<extURI>{uri}</extURI>"))
        .collect::<String>();
    let greeting = format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="no"?>
<epp xmlns="urn:ietf:params:xml:ns:epp-1.0">
  <greeting>
    <svID>epp-registry-simulator</svID>
    <svDate>{}</svDate>
    <svcMenu>
      <version>1.0</version>
      <lang>en</lang>
      {objects}
      {extensions}
    </svcMenu>
  </greeting>
</epp>"#,
        Utc::now().to_rfc3339()
    );
    write_frame(stream, greeting.as_bytes(), limits).await?;
    Ok(greeting)
}

#[allow(dead_code)]
async fn _read_marker<S: AsyncRead + Unpin>(_stream: &mut S) {}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::{escape_xml, send_greeting};
    use crate::epp::framing::{FrameLimits, read_frame};
    use tokio::io::duplex;

    fn limits() -> FrameLimits {
        FrameLimits {
            max_frame_size: 4096,
            read_timeout: Duration::from_millis(100),
            write_timeout: Duration::from_millis(100),
        }
    }

    #[test]
    fn escapes_xml_text() {
        assert_eq!(escape_xml("a<&>\"'"), "a&lt;&amp;&gt;&quot;&apos;");
    }

    #[tokio::test]
    async fn greeting_serializes_advertised_extensions_into_epp_frame() {
        let (mut client, mut server) = duplex(4096);
        let extension = "urn:epp:params:xml:ns:test-1.0";

        let greeting = send_greeting(
            &mut client,
            &limits(),
            &["urn:ietf:params:xml:ns:domain-1.0".to_owned()],
            &[extension.to_owned()],
        )
        .await
        .unwrap();
        let frame = String::from_utf8(read_frame(&mut server, &limits()).await.unwrap()).unwrap();

        assert!(greeting.contains(&format!("<extURI>{extension}</extURI>")));
        assert!(frame.contains(&format!("<extURI>{extension}</extURI>")));
    }
}
