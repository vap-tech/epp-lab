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

pub(crate) async fn send_contact_check<S>(
    stream: &mut S,
    limits: &FrameLimits,
    results: &[(String, bool)],
    cl_trid: Option<&str>,
    sv_trid: &str,
) -> Result<Response, FrameError>
where
    S: AsyncWrite + Unpin,
{
    let items = results
        .iter()
        .map(|(id, available)| {
            format!(
                "<contact:cd><contact:id avail=\"{}\">{}</contact:id></contact:cd>",
                if *available { "1" } else { "0" },
                escape_xml(id)
            )
        })
        .collect::<String>();
    let trid = cl_trid
        .map(|value| format!("<clTRID>{}</clTRID>", escape_xml(value)))
        .unwrap_or_default();
    let response = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?><epp xmlns="urn:ietf:params:xml:ns:epp-1.0"><response><result code="1000"><msg>Command completed successfully</msg></result><resData><contact:chkData xmlns:contact="urn:ietf:params:xml:ns:contact-1.0">{items}</contact:chkData></resData>{trid}<svTRID>{sv_trid}</svTRID></response></epp>"#
    );
    write_frame(stream, response.as_bytes(), limits).await?;
    Ok(Response {
        persisted_xml: response.clone(),
        xml: response,
        code: Some(SUCCESS),
    })
}

pub(crate) async fn send_contact_create<S>(
    stream: &mut S,
    limits: &FrameLimits,
    id: &str,
    created_at: &str,
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
        r#"<?xml version="1.0" encoding="UTF-8"?><epp xmlns="urn:ietf:params:xml:ns:epp-1.0"><response><result code="1000"><msg>Command completed successfully</msg></result><resData><contact:creData xmlns:contact="urn:ietf:params:xml:ns:contact-1.0"><contact:id>{}</contact:id><contact:crDate>{created_at}</contact:crDate></contact:creData></resData>{trid}<svTRID>{sv_trid}</svTRID></response></epp>"#,
        escape_xml(id)
    );
    write_frame(stream, response.as_bytes(), limits).await?;
    Ok(Response {
        persisted_xml: response.clone(),
        xml: response,
        code: Some(SUCCESS),
    })
}

pub(crate) async fn send_contact_info<S>(
    stream: &mut S,
    limits: &FrameLimits,
    contact: &crate::storage::contact::ContactDetailRow,
    auth_info: &str,
    cl_trid: Option<&str>,
    sv_trid: &str,
) -> Result<Response, FrameError>
where
    S: AsyncWrite + Unpin,
{
    let streets = contact
        .streets
        .iter()
        .map(|s| format!("<contact:street>{}</contact:street>", escape_xml(s)))
        .collect::<String>();
    let wire = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?><epp xmlns="urn:ietf:params:xml:ns:epp-1.0"><response><result code="1000"><msg>Command completed successfully</msg></result><resData><contact:infData xmlns:contact="urn:ietf:params:xml:ns:contact-1.0"><contact:id>{}</contact:id><contact:roid>{}</contact:roid><contact:status s="ok"/><contact:postalInfo type="int"><contact:name>{}</contact:name>{}<contact:addr>{}<contact:city>{}</contact:city><contact:cc>{}</contact:cc></contact:addr></contact:postalInfo><contact:voice>{}</contact:voice><contact:email>{}</contact:email><contact:authInfo><contact:pw>{}</contact:pw></contact:authInfo><contact:crDate>{}</contact:crDate><contact:upDate>{}</contact:upDate></contact:infData></resData>{}<svTRID>{}</svTRID></response></epp>"#,
        escape_xml(&contact.roid),
        escape_xml(&contact.roid),
        escape_xml(&contact.name),
        contact
            .organization
            .as_deref()
            .map(|o| format!("<contact:org>{}</contact:org>", escape_xml(o)))
            .unwrap_or_default(),
        streets,
        escape_xml(&contact.city),
        escape_xml(&contact.country_code),
        escape_xml(&contact.voice),
        escape_xml(&contact.email),
        escape_xml(auth_info),
        contact.created_at.to_rfc3339(),
        contact.updated_at.to_rfc3339(),
        cl_trid
            .map(|v| format!("<clTRID>{}</clTRID>", escape_xml(v)))
            .unwrap_or_default(),
        escape_xml(sv_trid)
    );
    let persisted = wire.replace(
        &format!("<contact:pw>{}</contact:pw>", escape_xml(auth_info)),
        "<contact:pw>REDACTED</contact:pw>",
    );
    write_frame(stream, wire.as_bytes(), limits).await?;
    Ok(Response {
        xml: wire,
        persisted_xml: persisted,
        code: Some(SUCCESS),
    })
}

pub(crate) async fn send_contact_delete<S>(
    stream: &mut S,
    limits: &FrameLimits,
    cl_trid: Option<&str>,
    sv_trid: &str,
) -> Result<Response, FrameError>
where
    S: AsyncWrite + Unpin,
{
    send_response(
        stream,
        limits,
        SUCCESS,
        "Command completed successfully",
        cl_trid,
        sv_trid,
    )
    .await
}

#[allow(dead_code)]
async fn _read_marker<S: AsyncRead + Unpin>(_stream: &mut S) {}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::{escape_xml, send_contact_info, send_greeting};
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

    #[tokio::test]
    async fn contact_info_keeps_auth_info_out_of_persisted_response() {
        let (mut client, mut server) = duplex(4096);
        let now = chrono::Utc::now();
        let contact = crate::storage::contact::ContactDetailRow {
            id: uuid::Uuid::new_v4(),
            roid: "C123".into(),
            sponsoring_registrar_id: uuid::Uuid::new_v4(),
            registrar_handle: Some("demo".into()),
            email: "contact@example.test".into(),
            voice: "+70000000000".into(),
            voice_extension: None,
            fax: None,
            fax_extension: None,
            name: "Test Contact".into(),
            organization: None,
            streets: vec!["Main 1".into()],
            city: "Moscow".into(),
            state_province: None,
            postal_code: None,
            country_code: "RU".into(),
            disclose_flag: "private".into(),
            disclosure_fields: vec![],
            statuses: vec!["ok".into()],
            created_at: now,
            updated_at: now,
        };
        let response = send_contact_info(
            &mut client,
            &limits(),
            &contact,
            "secret-auth",
            Some("T1"),
            "S1",
        )
        .await
        .unwrap();
        let frame = String::from_utf8(read_frame(&mut server, &limits()).await.unwrap()).unwrap();
        assert!(frame.contains("secret-auth"));
        assert!(response.xml.contains("secret-auth"));
        assert!(!response.persisted_xml.contains("secret-auth"));
        assert!(response.persisted_xml.contains("REDACTED"));
    }
}
