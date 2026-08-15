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

pub(crate) async fn send_domain_check<S>(
    stream: &mut S,
    limits: &FrameLimits,
    results: &[crate::application::DomainCheckResult],
    cl_trid: Option<&str>,
    sv_trid: &str,
) -> Result<Response, FrameError>
where
    S: AsyncWrite + Unpin,
{
    let items = results
        .iter()
        .map(|result| {
            let reason = result
                .reason
                .as_deref()
                .map(|value| format!("<domain:reason>{}</domain:reason>", escape_xml(value)))
                .unwrap_or_default();
            format!(
                "<domain:cd><domain:name avail=\"{}\">{}</domain:name>{}</domain:cd>",
                if result.available { "1" } else { "0" },
                escape_xml(&result.name),
                reason
            )
        })
        .collect::<String>();
    let trid = cl_trid
        .map(|value| format!("<clTRID>{}</clTRID>", escape_xml(value)))
        .unwrap_or_default();
    let response = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?><epp xmlns="urn:ietf:params:xml:ns:epp-1.0"><response><result code="1000"><msg>Command completed successfully</msg></result><resData><domain:chkData xmlns:domain="urn:ietf:params:xml:ns:domain-1.0">{items}</domain:chkData></resData>{trid}<svTRID>{sv_trid}</svTRID></response></epp>"#
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

pub(crate) async fn send_domain_create<S>(
    stream: &mut S,
    limits: &FrameLimits,
    name: &str,
    created_at: &str,
    expires_at: &str,
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
        r#"<?xml version="1.0" encoding="UTF-8"?><epp xmlns="urn:ietf:params:xml:ns:epp-1.0"><response><result code="1000"><msg>Command completed successfully</msg></result><resData><domain:creData xmlns:domain="urn:ietf:params:xml:ns:domain-1.0"><domain:name>{}</domain:name><domain:crDate>{}</domain:crDate><domain:exDate>{}</domain:exDate></domain:creData></resData>{}<svTRID>{}</svTRID></response></epp>"#,
        escape_xml(name),
        escape_xml(created_at),
        escape_xml(expires_at),
        trid,
        escape_xml(sv_trid)
    );
    write_frame(stream, response.as_bytes(), limits).await?;
    Ok(Response {
        persisted_xml: response.clone(),
        xml: response,
        code: Some(SUCCESS),
    })
}

#[allow(clippy::too_many_arguments)]
pub(crate) async fn send_contact_info<S>(
    stream: &mut S,
    limits: &FrameLimits,
    contact: &crate::storage::contact::ContactDetailRow,
    statuses: &[String],
    auth_info: Option<&str>,
    full_access: bool,
    cl_trid: Option<&str>,
    sv_trid: &str,
) -> Result<Response, FrameError>
where
    S: AsyncWrite + Unpin,
{
    let phone = |element: &str, number: &str, extension: Option<&str>| match extension {
        Some(extension) => format!(
            r#"<contact:{element} x="{}">{}</contact:{element}>"#,
            escape_xml(extension),
            escape_xml(number),
        ),
        None => format!(
            "<contact:{element}>{}</contact:{element}>",
            escape_xml(number)
        ),
    };
    let allows = |field: &str| {
        full_access
            || (contact.disclose_flag == "public"
                && contact.disclosure_fields.iter().any(|value| value == field))
    };
    let show_name = allows("name");
    let show_organization = allows("organization");
    let show_address = allows("address");
    // RFC postalInfo requires both name and address. A non-sponsor response
    // omits the whole element unless it can remain schema-valid.
    let show_postal_info = show_name && show_address;
    let streets = contact
        .streets
        .iter()
        .map(|s| format!("<contact:street>{}</contact:street>", escape_xml(s)))
        .collect::<String>();
    let statuses = statuses
        .iter()
        .map(|status| format!(r#"<contact:status s="{}"/>"#, escape_xml(status)))
        .collect::<String>();
    let postal_info = if show_postal_info {
        let name = format!("<contact:name>{}</contact:name>", escape_xml(&contact.name));
        format!(
            "<contact:postalInfo type=\"int\">{}{}<contact:addr>{}<contact:city>{}</contact:city>{}<contact:cc>{}</contact:cc></contact:addr></contact:postalInfo>",
            name,
            if show_organization {
                contact
                    .organization
                    .as_deref()
                    .map(|organization| {
                        format!("<contact:org>{}</contact:org>", escape_xml(organization))
                    })
                    .unwrap_or_default()
            } else {
                String::new()
            },
            if show_address { streets } else { String::new() },
            if show_address {
                escape_xml(&contact.city)
            } else {
                String::new()
            },
            if show_address {
                format!(
                    "{}{}",
                    contact
                        .state_province
                        .as_deref()
                        .map(|state| format!("<contact:sp>{}</contact:sp>", escape_xml(state)))
                        .unwrap_or_default(),
                    contact
                        .postal_code
                        .as_deref()
                        .map(|postal_code| format!(
                            "<contact:pc>{}</contact:pc>",
                            escape_xml(postal_code)
                        ))
                        .unwrap_or_default()
                )
            } else {
                String::new()
            },
            if show_address {
                escape_xml(&contact.country_code)
            } else {
                String::new()
            },
        )
    } else {
        String::new()
    };
    let localized_postal_info = if show_postal_info {
        contact.localized_name.as_ref().map(|name| {
            let streets = contact
                .localized_streets
                .iter()
                .map(|street| format!("<contact:street>{}</contact:street>", escape_xml(street)))
                .collect::<String>();
            format!(
                "<contact:postalInfo type=\"loc\"><contact:name>{}</contact:name>{}<contact:addr>{}{}{}<contact:city>{}</contact:city><contact:cc>{}</contact:cc></contact:addr></contact:postalInfo>",
                escape_xml(name),
                if show_organization { contact.localized_organization.as_deref().map(|organization| format!("<contact:org>{}</contact:org>", escape_xml(organization))).unwrap_or_default() } else { String::new() },
                if show_address { streets } else { String::new() },
                contact.localized_state_province.as_deref().map(|state_province| format!("<contact:sp>{}</contact:sp>", escape_xml(state_province))).unwrap_or_default(),
                contact.localized_postal_code.as_deref().map(|postal_code| format!("<contact:pc>{}</contact:pc>", escape_xml(postal_code))).unwrap_or_default(),
                if show_address { escape_xml(contact.localized_city.as_deref().unwrap_or_default()) } else { String::new() },
                if show_address { escape_xml(contact.localized_country_code.as_deref().unwrap_or_default()) } else { String::new() },
            )
        }).unwrap_or_default()
    } else {
        String::new()
    };
    let disclose_fields = contact
        .disclosure_fields
        .iter()
        .filter_map(|field| match field.as_str() {
            "name" => Some("name"),
            "organization" => Some("org"),
            "address" => Some("addr"),
            "voice" => Some("voice"),
            "fax" => Some("fax"),
            "email" => Some("email"),
            _ => None,
        })
        .map(|field| format!("<contact:{field}/>"))
        .collect::<String>();
    let disclose = format!(
        r#"<contact:disclose flag="{}">{disclose_fields}</contact:disclose>"#,
        if contact.disclose_flag == "public" {
            "1"
        } else {
            "0"
        }
    );
    let fax = if allows("fax") {
        contact
            .fax
            .as_deref()
            .map(|fax| phone("fax", fax, contact.fax_extension.as_deref()))
            .unwrap_or_default()
    } else {
        String::new()
    };
    let update_metadata = if contact.updated_at > contact.created_at {
        format!(
            "<contact:upID>{}</contact:upID><contact:upDate>{}</contact:upDate>",
            escape_xml(contact.updated_by_handle.as_deref().unwrap_or_default()),
            contact.updated_at.to_rfc3339(),
        )
    } else {
        String::new()
    };
    let transfer_metadata = contact
        .transferred_at
        .map(|transferred_at| {
            format!(
                "<contact:trDate>{}</contact:trDate>",
                transferred_at.to_rfc3339()
            )
        })
        .unwrap_or_default();
    let wire = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?><epp xmlns="urn:ietf:params:xml:ns:epp-1.0"><response><result code="1000"><msg>Command completed successfully</msg></result><resData><contact:infData xmlns:contact="urn:ietf:params:xml:ns:contact-1.0"><contact:id>{}</contact:id><contact:roid>{}</contact:roid>{}{}{}{}{}{}{}{}<contact:clID>{}</contact:clID><contact:crID>{}</contact:crID><contact:crDate>{}</contact:crDate>{}{}</contact:infData></resData>{}<svTRID>{}</svTRID></response></epp>"#,
        escape_xml(&contact.roid),
        escape_xml(&contact.roid),
        statuses,
        postal_info,
        localized_postal_info,
        if allows("voice") {
            phone("voice", &contact.voice, contact.voice_extension.as_deref())
        } else {
            String::new()
        },
        fax,
        if allows("email") {
            format!(
                "<contact:email>{}</contact:email>",
                escape_xml(&contact.email)
            )
        } else {
            String::new()
        },
        disclose,
        auth_info
            .map(|value| format!(
                "<contact:authInfo><contact:pw>{}</contact:pw></contact:authInfo>",
                escape_xml(value)
            ))
            .unwrap_or_default(),
        escape_xml(contact.registrar_handle.as_deref().unwrap_or_default()),
        escape_xml(contact.created_by_handle.as_deref().unwrap_or_default()),
        contact.created_at.to_rfc3339(),
        update_metadata,
        transfer_metadata,
        cl_trid
            .map(|v| format!("<clTRID>{}</clTRID>", escape_xml(v)))
            .unwrap_or_default(),
        escape_xml(sv_trid)
    );
    let persisted = match auth_info {
        Some(value) => wire.replace(
            &format!("<contact:pw>{}</contact:pw>", escape_xml(value)),
            "<contact:pw>REDACTED</contact:pw>",
        ),
        None => wire.clone(),
    };
    write_frame(stream, wire.as_bytes(), limits).await?;
    Ok(Response {
        xml: wire,
        persisted_xml: persisted,
        code: Some(SUCCESS),
    })
}

#[allow(clippy::too_many_arguments)]
pub(crate) async fn send_domain_info<S>(
    stream: &mut S,
    limits: &FrameLimits,
    domain: &crate::storage::domain::DomainRow,
    contacts: &[(String, String)],
    nameservers: &[crate::storage::domain::DomainNameserverRow],
    statuses: &[String],
    auth_info: Option<&str>,
    cl_trid: Option<&str>,
    sv_trid: &str,
) -> Result<Response, FrameError>
where
    S: AsyncWrite + Unpin,
{
    let contact_xml = contacts
        .iter()
        .map(|(role, roid)| {
            format!(
                r#"<domain:contact type="{}">{}</domain:contact>"#,
                escape_xml(role),
                escape_xml(roid)
            )
        })
        .collect::<String>();
    let ns_xml = nameservers.iter().map(|ns| format!(
        "<domain:ns><domain:hostAttr><domain:hostName>{}</domain:hostName></domain:hostAttr></domain:ns>", escape_xml(&ns.hostname)
    )).collect::<String>();
    let status_xml = statuses
        .iter()
        .map(|status| format!("<domain:status s=\"{}\"/>", escape_xml(status)))
        .collect::<String>();
    let auth_xml = auth_info
        .map(|value| {
            format!(
                "<domain:authInfo><domain:pw>{}</domain:pw></domain:authInfo>",
                escape_xml(value)
            )
        })
        .unwrap_or_default();
    let trid = cl_trid
        .map(|value| format!("<clTRID>{}</clTRID>", escape_xml(value)))
        .unwrap_or_default();
    let expires = format!(
        "<domain:exDate>{}</domain:exDate>",
        domain.expires_at.to_rfc3339()
    );
    let response = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?><epp xmlns="urn:ietf:params:xml:ns:epp-1.0"><response><result code="1000"><msg>Command completed successfully</msg></result><resData><domain:infData xmlns:domain="urn:ietf:params:xml:ns:domain-1.0"><domain:name>{name}</domain:name><domain:roid>{roid}</domain:roid><domain:clID>{cl_id}</domain:clID><domain:crID>{created_by}</domain:crID><domain:crDate>{created_at}</domain:crDate>{updated}{expires}<domain:ns>{ns_xml}</domain:ns>{contact_xml}{status_xml}{auth_xml}</domain:infData></resData>{trid}<svTRID>{sv_trid}</svTRID></response></epp>"#,
        name = escape_xml(&domain.name),
        roid = escape_xml(&domain.roid),
        cl_id = domain.sponsoring_registrar_id,
        created_by = domain.created_by,
        created_at = domain.created_at.to_rfc3339(),
        updated = domain
            .updated_at
            .map(|date| format!("<domain:upDate>{}</domain:upDate>", date.to_rfc3339()))
            .unwrap_or_default(),
        expires = expires,
        ns_xml = ns_xml,
        contact_xml = contact_xml,
        status_xml = status_xml,
        auth_xml = auth_xml,
        trid = trid,
        sv_trid = escape_xml(sv_trid)
    );
    let persisted_xml = response.replace(&auth_xml, "");
    write_frame(stream, response.as_bytes(), limits).await?;
    Ok(Response {
        persisted_xml,
        xml: response,
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
            created_by_handle: Some("demo".into()),
            updated_by_handle: Some("demo".into()),
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
            localized_name: Some("Локальное имя".into()),
            localized_organization: Some("Компания".into()),
            localized_streets: vec!["Улица 1".into()],
            localized_city: Some("Москва".into()),
            localized_state_province: None,
            localized_postal_code: None,
            localized_country_code: Some("RU".into()),
            disclose_flag: "public".into(),
            disclosure_fields: vec!["email".into()],
            statuses: vec!["ok".into()],
            created_at: now,
            updated_at: now,
            transferred_at: None,
        };
        let response = send_contact_info(
            &mut client,
            &limits(),
            &contact,
            &["ok".to_owned()],
            Some("secret-auth"),
            true,
            Some("T1"),
            "S1",
        )
        .await
        .unwrap();
        let frame = String::from_utf8(read_frame(&mut server, &limits()).await.unwrap()).unwrap();
        assert!(frame.contains("secret-auth"));
        assert!(response.xml.contains("secret-auth"));
        assert!(response.xml.contains(r#"<contact:postalInfo type="loc">"#));
        assert!(response.xml.contains("Локальное имя"));
        assert!(response.xml.contains(r#"<contact:disclose flag="1">"#));
        assert!(response.xml.contains("<contact:email/>"));
        assert!(!response.persisted_xml.contains("secret-auth"));
        assert!(response.persisted_xml.contains("REDACTED"));
    }

    #[tokio::test]
    async fn contact_info_for_non_sponsor_hides_private_fields_and_auth_info() {
        let (mut client, mut server) = duplex(4096);
        let now = chrono::Utc::now();
        let contact = crate::storage::contact::ContactDetailRow {
            id: uuid::Uuid::new_v4(),
            roid: "C123".into(),
            sponsoring_registrar_id: uuid::Uuid::new_v4(),
            registrar_handle: Some("demo".into()),
            created_by_handle: Some("demo".into()),
            updated_by_handle: Some("demo".into()),
            email: "private@example.test".into(),
            voice: "+70000000000".into(),
            voice_extension: None,
            fax: None,
            fax_extension: None,
            name: "Private Name".into(),
            organization: None,
            streets: vec!["Private Street".into()],
            city: "Moscow".into(),
            state_province: None,
            postal_code: None,
            country_code: "RU".into(),
            localized_name: None,
            localized_organization: None,
            localized_streets: vec![],
            localized_city: None,
            localized_state_province: None,
            localized_postal_code: None,
            localized_country_code: None,
            disclose_flag: "private".into(),
            disclosure_fields: vec![],
            statuses: vec!["ok".into()],
            created_at: now,
            updated_at: now,
            transferred_at: None,
        };
        let response = send_contact_info(
            &mut client,
            &limits(),
            &contact,
            &["ok".to_owned()],
            None,
            false,
            Some("T1"),
            "S1",
        )
        .await
        .unwrap();
        let frame = String::from_utf8(read_frame(&mut server, &limits()).await.unwrap()).unwrap();
        assert!(!frame.contains("Private Name"));
        assert!(!frame.contains("private@example.test"));
        assert!(!frame.contains("authInfo"));
        assert_eq!(response.xml, response.persisted_xml);
    }
}
