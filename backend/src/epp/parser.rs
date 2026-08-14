use quick_xml::{
    Reader, Writer,
    events::{BytesText, Event},
};
use std::io::Cursor;
use thiserror::Error;

#[allow(clippy::large_enum_variant)]
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum EppCommand {
    Hello,
    Login(LoginCommand),
    Logout,
    Contact(ContactCommand),
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum ContactCommand {
    Check(ContactCheckCommand),
    Create(ContactCreateCommand),
    Info,
    Update,
    Delete,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct ContactCheckCommand {
    pub ids: Vec<String>,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct ContactCreateCommand {
    pub id: String,
    pub name: String,
    pub organization: Option<String>,
    pub streets: Vec<String>,
    pub city: String,
    pub state_province: Option<String>,
    pub postal_code: Option<String>,
    pub country_code: String,
    pub voice: String,
    pub voice_extension: Option<String>,
    pub fax: Option<String>,
    pub fax_extension: Option<String>,
    pub email: String,
    pub auth_info: String,
}

impl EppCommand {
    pub(crate) fn name(&self) -> &'static str {
        match self {
            Self::Hello => "hello",
            Self::Login(_) => "login",
            Self::Logout => "logout",
            Self::Contact(ContactCommand::Check(_)) => "contact:check",
            Self::Contact(ContactCommand::Create(_)) => "contact:create",
            Self::Contact(ContactCommand::Info) => "contact:info",
            Self::Contact(ContactCommand::Update) => "contact:update",
            Self::Contact(ContactCommand::Delete) => "contact:delete",
        }
    }
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

impl ParsedCommand {
    pub(crate) fn name(&self) -> &'static str {
        self.command.name()
    }
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
    let mut contact_ids = Vec::new();
    let mut contact_create_values = std::collections::BTreeMap::new();
    let mut contact_streets = Vec::new();
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
                if name.ends_with(b"check") {
                    command = Some(EppCommand::Contact(ContactCommand::Check(
                        ContactCheckCommand { ids: Vec::new() },
                    )));
                } else if name.ends_with(b"create") {
                    command = Some(EppCommand::Contact(ContactCommand::Create(
                        ContactCreateCommand {
                            id: String::new(),
                            name: String::new(),
                            organization: None,
                            streets: Vec::new(),
                            city: String::new(),
                            state_province: None,
                            postal_code: None,
                            country_code: String::new(),
                            voice: String::new(),
                            voice_extension: None,
                            fax: None,
                            fax_extension: None,
                            email: String::new(),
                            auth_info: String::new(),
                        },
                    )));
                } else if name.ends_with(b"info") {
                    command = Some(EppCommand::Contact(ContactCommand::Info));
                } else if name.ends_with(b"update") {
                    command = Some(EppCommand::Contact(ContactCommand::Update));
                } else if name.ends_with(b"delete") {
                    command = Some(EppCommand::Contact(ContactCommand::Delete));
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
                    Some(name) if name.ends_with(b"pw") => {
                        password = Some(value.clone());
                        contact_create_values.insert("pw", value);
                    }
                    Some(name) if name.ends_with(b"clTRID") => cl_trid = Some(value),
                    Some(name) if name.ends_with(b"objURI") => object_uris.push(value),
                    Some(name) if name.ends_with(b"extURI") => extension_uris.push(value),
                    Some(name) if name.ends_with(b"id") => contact_ids.push(value),
                    Some(name) if name.ends_with(b"name") => {
                        contact_create_values.insert("name", value);
                    }
                    Some(name) if name.ends_with(b"org") => {
                        contact_create_values.insert("org", value);
                    }
                    Some(name) if name.ends_with(b"city") => {
                        contact_create_values.insert("city", value);
                    }
                    Some(name) if name.ends_with(b"sp") => {
                        contact_create_values.insert("sp", value);
                    }
                    Some(name) if name.ends_with(b"pc") => {
                        contact_create_values.insert("pc", value);
                    }
                    Some(name) if name.ends_with(b"cc") => {
                        contact_create_values.insert("cc", value);
                    }
                    Some(name) if name.ends_with(b"voice") => {
                        contact_create_values.insert("voice", value);
                    }
                    Some(name) if name.ends_with(b"ext") => {
                        contact_create_values.insert("ext", value);
                    }
                    Some(name) if name.ends_with(b"fax") => {
                        contact_create_values.insert("fax", value);
                    }
                    Some(name) if name.ends_with(b"email") => {
                        contact_create_values.insert("email", value);
                    }
                    Some(name) if name.ends_with(b"street") => contact_streets.push(value),
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
        Some(EppCommand::Contact(ContactCommand::Check(_))) => {
            if contact_ids.is_empty() {
                return Err(ParseError::Command);
            }
            Ok(ParsedCommand {
                command: EppCommand::Contact(ContactCommand::Check(ContactCheckCommand {
                    ids: contact_ids,
                })),
                cl_trid,
            })
        }
        Some(EppCommand::Contact(ContactCommand::Create(mut create))) => {
            create.id = contact_ids.first().cloned().ok_or(ParseError::Command)?;
            create.name = contact_create_values
                .remove("name")
                .ok_or(ParseError::Command)?;
            create.organization = contact_create_values.remove("org");
            create.streets = contact_streets;
            create.city = contact_create_values
                .remove("city")
                .ok_or(ParseError::Command)?;
            create.state_province = contact_create_values.remove("sp");
            create.postal_code = contact_create_values.remove("pc");
            create.country_code = contact_create_values
                .remove("cc")
                .ok_or(ParseError::Command)?;
            create.voice = contact_create_values
                .remove("voice")
                .ok_or(ParseError::Command)?;
            create.voice_extension = contact_create_values.remove("ext");
            create.fax = contact_create_values.remove("fax");
            create.email = contact_create_values
                .remove("email")
                .ok_or(ParseError::Command)?;
            create.auth_info = contact_create_values
                .remove("pw")
                .ok_or(ParseError::Command)?;
            Ok(ParsedCommand {
                command: EppCommand::Contact(ContactCommand::Create(create)),
                cl_trid,
            })
        }
        Some(EppCommand::Contact(contact)) => Ok(ParsedCommand {
            command: EppCommand::Contact(contact),
            cl_trid,
        }),
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
    fn recognizes_contact_commands_without_implementing_business_logic() {
        let parsed = parse_command(
            br#"<epp xmlns="urn:ietf:params:xml:ns:epp-1.0"><command><info xmlns:contact="urn:ietf:params:xml:ns:contact-1.0"><contact:info/></info></command></epp>"#,
        )
        .unwrap();
        assert_eq!(parsed.name(), "contact:info");
    }

    #[test]
    fn parses_contact_check_ids() {
        let parsed = parse_command(
            br#"<epp xmlns="urn:ietf:params:xml:ns:epp-1.0"><command><check><contact:check xmlns:contact="urn:ietf:params:xml:ns:contact-1.0"><contact:id>C123</contact:id><contact:id>C456</contact:id></contact:check></check></command></epp>"#,
        )
        .unwrap();
        assert_eq!(
            parsed.command,
            EppCommand::Contact(ContactCommand::Check(ContactCheckCommand {
                ids: vec!["C123".into(), "C456".into()]
            }))
        );
    }

    #[test]
    fn parses_contact_create_required_and_optional_fields() {
        let parsed = parse_command(
            br#"<epp xmlns="urn:ietf:params:xml:ns:epp-1.0"><command><create><contact:create xmlns:contact="urn:ietf:params:xml:ns:contact-1.0"><contact:id>C123</contact:id><contact:postalInfo type="int"><contact:name>Name</contact:name><contact:org>Org</contact:org><contact:addr><contact:street>Main 1</contact:street><contact:city>Moscow</contact:city><contact:cc>RU</contact:cc></contact:addr></contact:postalInfo><contact:voice x="123">+70000000000</contact:voice><contact:email>a@example.test</contact:email><contact:authInfo><contact:pw>secret</contact:pw></contact:authInfo></contact:create></create></command></epp>"#,
        )
        .unwrap();
        let EppCommand::Contact(ContactCommand::Create(create)) = parsed.command else {
            panic!("expected contact create");
        };
        assert_eq!(create.id, "C123");
        assert_eq!(create.name, "Name");
        assert_eq!(create.organization.as_deref(), Some("Org"));
        assert_eq!(create.streets, vec!["Main 1"]);
        assert_eq!(create.country_code, "RU");
        assert_eq!(create.auth_info, "secret");
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

    #[test]
    fn exposes_protocol_command_name() {
        let parsed = parse_command(
            br#"<epp xmlns="urn:ietf:params:xml:ns:epp-1.0"><command><hello/></command></epp>"#,
        )
        .unwrap();
        assert_eq!(parsed.name(), "hello");
    }
}
