use quick_xml::{Reader, events::Event};
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

#[derive(Debug, Error)]
pub(crate) enum ParseError {
    #[error("invalid XML: {0}")]
    Xml(String),
    #[error("unsupported or malformed EPP command")]
    Command,
    #[error("unsupported EPP command")]
    Unsupported,
}

pub(crate) fn parse_command(xml: &[u8]) -> Result<EppCommand, ParseError> {
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
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(event)) => {
                let name = event.name().as_ref().to_vec();
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
                    Some(name) if name == b"clID" => client_id = Some(value),
                    Some(name) if name == b"pw" => password = Some(value),
                    Some(name) if name == b"clTRID" => cl_trid = Some(value),
                    Some(name) if name == b"objURI" => object_uris.push(value),
                    Some(name) if name == b"extURI" => extension_uris.push(value),
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
        Some(EppCommand::Hello) => Ok(EppCommand::Hello),
        Some(EppCommand::Logout) => Ok(EppCommand::Logout),
        Some(EppCommand::Login(mut login)) => {
            login.client_id = client_id.ok_or(ParseError::Command)?;
            login.password = password.ok_or(ParseError::Command)?;
            login.cl_trid = cl_trid;
            login.object_uris = object_uris;
            login.extension_uris = extension_uris;
            Ok(EppCommand::Login(login))
        }
        None if xml.windows(9).any(|window| window == b"<command>") => Err(ParseError::Unsupported),
        None => Err(ParseError::Command),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_hello() {
        assert_eq!(
            parse_command(br#"<epp><command><hello/></command></epp>"#).unwrap(),
            EppCommand::Hello
        );
    }

    #[test]
    fn parses_login() {
        let command = parse_command(br#"<epp><command><login><clID>REG-1</clID><pw>secret</pw></login><clTRID>abc</clTRID></command></epp>"#).unwrap();
        assert_eq!(
            command,
            EppCommand::Login(LoginCommand {
                client_id: "REG-1".into(),
                password: "secret".into(),
                cl_trid: Some("abc".into()),
                object_uris: Vec::new(),
                extension_uris: Vec::new(),
            })
        );
    }

    #[test]
    fn parses_logout() {
        assert_eq!(
            parse_command(br#"<epp><command><logout/></command></epp>"#).unwrap(),
            EppCommand::Logout
        );
    }

    #[test]
    fn rejects_login_without_password() {
        assert!(matches!(
            parse_command(br#"<epp><command><login><clID>REG-1</clID></login></command></epp>"#),
            Err(ParseError::Command)
        ));
    }

    #[test]
    fn rejects_malformed_xml() {
        assert!(matches!(
            parse_command(br#"<epp><command><hello></epp>"#),
            Err(ParseError::Xml(_))
        ));
    }

    #[test]
    fn identifies_unsupported_command() {
        assert!(matches!(
            parse_command(br#"<epp><command><info/></command></epp>"#),
            Err(ParseError::Unsupported)
        ));
    }
}
