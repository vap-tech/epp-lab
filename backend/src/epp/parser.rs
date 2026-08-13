use quick_xml::{
    Reader, Writer,
    events::{BytesText, Event},
};
use std::io::Cursor;
use thiserror::Error;

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum EppCommand {
    Hello,
    Login(LoginCommand),
    Logout,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct LoginCommand {
    pub client_id: String,
    pub password: String,
    pub cl_trid: Option<String>,
    pub object_uris: Vec<String>,
    pub extension_uris: Vec<String>,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct ParsedCommand {
    pub command: EppCommand,
    pub cl_trid: Option<String>,
}

#[derive(Debug, Error)]
pub(crate) enum ParseError {
    #[error("invalid XML: {0}")]
    Xml(String),
    #[error("unsupported or malformed EPP command")]
    Command,
    #[error("unsupported EPP command")]
    Unsupported,
    #[error("invalid EPP namespace")]
    Namespace,
}

pub(crate) fn parse_command(xml: &[u8]) -> Result<ParsedCommand, ParseError> {
    let mut reader = Reader::from_reader(xml);
    reader.config_mut().trim_text(true);
    let mut buf = Vec::new();
    let mut path: Vec<Vec<u8>> = Vec::new();
    let mut command = None;
    let mut client_id = None;
    let mut password = None;
    let mut cl_trid = None;
    let mut object_uris = Vec::new();
    let mut extension_uris = Vec::new();
    let mut root_seen = false;
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(event)) => {
                let name = event.name().as_ref().to_vec();
                if !root_seen {
                    root_seen = true;
                    if name != b"epp"
                        || event
                            .attributes()
                            .flatten()
                            .find(|attribute| attribute.key.as_ref() == b"xmlns")
                            .and_then(|attribute| attribute.unescape_value().ok())
                            .as_deref()
                            != Some("urn:ietf:params:xml:ns:epp-1.0")
                    {
                        return Err(ParseError::Namespace);
                    }
                }
                if name.ends_with(b"hello") {
                    command = Some(EppCommand::Hello);
                }
                if name.ends_with(b"logout") {
                    command = Some(EppCommand::Logout);
                }
                if name.ends_with(b"login") {
                    command = Some(EppCommand::Login(LoginCommand {
                        client_id: String::new(),
                        password: String::new(),
                        cl_trid: None,
                        object_uris: Vec::new(),
                        extension_uris: Vec::new(),
                    }));
                }
                path.push(name);
            }
            Ok(Event::Empty(event)) => {
                let name = event.name().as_ref().to_vec();
                if name.ends_with(b"hello") {
                    command = Some(EppCommand::Hello);
                } else if name.ends_with(b"logout") {
                    command = Some(EppCommand::Logout);
                }
            }
            Ok(Event::Text(text)) => {
                let value = text
                    .decode()
                    .map_err(|e| ParseError::Xml(e.to_string()))?
                    .into_owned();
                match path.last().map(Vec::as_slice) {
                    Some(name) if name.ends_with(b"clID") => client_id = Some(value),
                    Some(name) if name.ends_with(b"pw") => password = Some(value),
                    Some(name) if name.ends_with(b"clTRID") => cl_trid = Some(value),
                    Some(name) if name.ends_with(b"objURI") => object_uris.push(value),
                    Some(name) if name.ends_with(b"extURI") => extension_uris.push(value),
                    _ => {}
                }
            }
            Ok(Event::End(_)) => {
                path.pop();
            }
            Ok(Event::Eof) => break,
            Err(error) => return Err(ParseError::Xml(error.to_string())),
            _ => {}
        }
        buf.clear();
    }
    match command {
        Some(EppCommand::Hello) => Ok(ParsedCommand {
            command: EppCommand::Hello,
            cl_trid,
        }),
        Some(EppCommand::Logout) => Ok(ParsedCommand {
            command: EppCommand::Logout,
            cl_trid,
        }),
        Some(EppCommand::Login(mut login)) => {
            login.client_id = client_id.ok_or(ParseError::Command)?;
            login.password = password.ok_or(ParseError::Command)?;
            login.cl_trid = cl_trid.clone();
            login.object_uris = object_uris;
            login.extension_uris = extension_uris;
            Ok(ParsedCommand {
                command: EppCommand::Login(login),
                cl_trid,
            })
        }
        None if xml.windows(9).any(|window| window == b"<command>") => Err(ParseError::Unsupported),
        None => Err(ParseError::Command),
    }
}

pub(crate) fn redact_password(xml: &[u8]) -> Result<String, ParseError> {
    let mut reader = Reader::from_reader(xml);
    let mut writer = Writer::new(Cursor::new(Vec::new()));
    let mut buf = Vec::new();
    let mut path: Vec<Vec<u8>> = Vec::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(event)) => {
                let name = event.name().as_ref().to_vec();
                let is_password = name.ends_with(b"pw");
                path.push(name);
                writer
                    .write_event(Event::Start(event))
                    .map_err(|e| ParseError::Xml(e.to_string()))?;
                if is_password {
                    writer
                        .write_event(Event::Text(BytesText::new("[REDACTED]")))
                        .map_err(|e| ParseError::Xml(e.to_string()))?;
                }
            }
            Ok(Event::Text(text)) if path.last().is_some_and(|name| name.ends_with(b"pw")) => {
                let _ = text;
            }
            Ok(Event::End(event)) => {
                writer
                    .write_event(Event::End(event))
                    .map_err(|e| ParseError::Xml(e.to_string()))?;
                path.pop();
            }
            Ok(Event::Eof) => break,
            Ok(event) => writer
                .write_event(event)
                .map_err(|e| ParseError::Xml(e.to_string()))?,
            Err(error) => return Err(ParseError::Xml(error.to_string())),
        }
        buf.clear();
    }
    String::from_utf8(writer.into_inner().into_inner()).map_err(|e| ParseError::Xml(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_hello() {
        assert_eq!(
            parse_command(
                br#"<epp xmlns="urn:ietf:params:xml:ns:epp-1.0"><command><hello/></command></epp>"#
            )
            .unwrap(),
            ParsedCommand {
                command: EppCommand::Hello,
                cl_trid: None
            }
        );
    }

    #[test]
    fn parses_login() {
        let command = parse_command(br#"<epp xmlns="urn:ietf:params:xml:ns:epp-1.0"><command><login><clID>REG-1</clID><pw>secret</pw></login><clTRID>abc</clTRID></command></epp>"#).unwrap();
        assert_eq!(
            command,
            ParsedCommand {
                command: EppCommand::Login(LoginCommand {
                    client_id: "REG-1".into(),
                    password: "secret".into(),
                    cl_trid: Some("abc".into()),
                    object_uris: Vec::new(),
                    extension_uris: Vec::new(),
                }),
                cl_trid: Some("abc".into()),
            }
        );
    }

    #[test]
    fn parses_logout() {
        assert_eq!(
            parse_command(br#"<epp xmlns="urn:ietf:params:xml:ns:epp-1.0"><command><logout/></command></epp>"#).unwrap(),
            ParsedCommand { command: EppCommand::Logout, cl_trid: None }
        );
    }

    #[test]
    fn rejects_login_without_password() {
        assert!(matches!(
            parse_command(br#"<epp xmlns="urn:ietf:params:xml:ns:epp-1.0"><command><login><clID>REG-1</clID></login></command></epp>"#),
            Err(ParseError::Command)
        ));
    }

    #[test]
    fn rejects_malformed_xml() {
        assert!(matches!(
            parse_command(br#"<epp xmlns="urn:ietf:params:xml:ns:epp-1.0"><command><hello></epp>"#),
            Err(ParseError::Xml(_))
        ));
    }

    #[test]
    fn identifies_unsupported_command() {
        assert!(matches!(
            parse_command(
                br#"<epp xmlns="urn:ietf:params:xml:ns:epp-1.0"><command><info/></command></epp>"#
            ),
            Err(ParseError::Unsupported)
        ));
    }

    #[test]
    fn rejects_wrong_namespace() {
        assert!(matches!(
            parse_command(br#"<epp xmlns="urn:example"><command><hello/></command></epp>"#),
            Err(ParseError::Namespace)
        ));
    }

    #[test]
    fn extracts_transaction_id_and_redacts_namespaced_password() {
        let xml = br#"<epp xmlns="urn:ietf:params:xml:ns:epp-1.0"><command><login><clID>REG-1</clID><e:pw xmlns:e="urn:example">secret</e:pw></login><clTRID>abc</clTRID></command></epp>"#;
        let parsed = parse_command(xml).unwrap();
        assert_eq!(parsed.cl_trid.as_deref(), Some("abc"));
        let redacted = redact_password(xml).unwrap();
        assert!(!redacted.contains("secret"));
        assert!(redacted.contains("[REDACTED]"));
    }
}
