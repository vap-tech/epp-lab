use quick_xml::{
    Reader, Writer,
    events::{BytesText, Event},
};
use std::io::Cursor;
use thiserror::Error;

const EPP_NS: &str = "urn:ietf:params:xml:ns:epp-1.0";
const DOMAIN_NS: &str = "urn:ietf:params:xml:ns:domain-1.0";

#[allow(clippy::large_enum_variant)]
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum EppCommand {
    Hello,
    Login(LoginCommand),
    Logout,
    Contact(ContactCommand),
    Domain(DomainCommand),
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum ContactCommand {
    Check(ContactCheckCommand),
    Create(ContactCreateCommand),
    Info(ContactInfoCommand),
    Update(ContactUpdateCommand),
    Delete(ContactDeleteCommand),
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum DomainCommand {
    Check(DomainCheckCommand),
    Create(DomainCreateCommand),
    Info(DomainInfoCommand),
    Update(DomainUpdateCommand),
    Delete(DomainDeleteCommand),
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct DomainCheckCommand {
    pub names: Vec<String>,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct DomainContactCommand {
    pub id: String,
    pub role: String,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct DomainCreateCommand {
    pub name: String,
    pub period: Option<DomainPeriodCommand>,
    pub nameservers: Vec<String>,
    pub registrant: Option<String>,
    pub contacts: Vec<DomainContactCommand>,
    pub auth_info: String,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct DomainPeriodCommand {
    pub value: u32,
    pub unit: String,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct DomainInfoCommand {
    pub name: String,
    pub auth_info: Option<String>,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct DomainUpdateCommand {
    pub name: String,
    pub add_nameservers: Vec<String>,
    pub rem_nameservers: Vec<String>,
    pub add_contacts: Vec<DomainContactCommand>,
    pub rem_contacts: Vec<DomainContactCommand>,
    pub add_statuses: Vec<String>,
    pub rem_statuses: Vec<String>,
    pub chg_registrant: crate::domain::contact::Patch<String>,
    pub chg_auth_info: crate::domain::contact::Patch<String>,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct DomainDeleteCommand {
    pub name: String,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct ContactCheckCommand {
    pub ids: Vec<String>,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct ContactInfoCommand {
    pub id: String,
    pub auth_info: Option<String>,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct ContactUpdateCommand {
    pub id: String,
    pub add_statuses: Vec<String>,
    pub rem_statuses: Vec<String>,
    pub chg_email: crate::domain::contact::Patch<String>,
    pub chg_auth_info: crate::domain::contact::Patch<String>,
    pub chg_voice: crate::domain::contact::Patch<String>,
    pub chg_fax: crate::domain::contact::Patch<String>,
    pub chg_organization: crate::domain::contact::Patch<String>,
    pub chg_city: crate::domain::contact::Patch<String>,
    pub chg_state_province: crate::domain::contact::Patch<String>,
    pub chg_postal_code: crate::domain::contact::Patch<String>,
    pub chg_country_code: crate::domain::contact::Patch<String>,
    pub chg_streets: Vec<String>,
    pub chg_disclose: crate::domain::contact::Patch<String>,
    pub chg_disclose_fields: Vec<String>,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct ContactDeleteCommand {
    pub id: String,
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
    pub disclose_flag: Option<String>,
    pub disclose_fields: Vec<String>,
    pub localized: Option<ContactPostalInfoCommand>,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct ContactPostalInfoCommand {
    pub name: String,
    pub organization: Option<String>,
    pub streets: Vec<String>,
    pub city: String,
    pub state_province: Option<String>,
    pub postal_code: Option<String>,
    pub country_code: String,
}

impl EppCommand {
    pub(crate) fn name(&self) -> &'static str {
        match self {
            Self::Hello => "hello",
            Self::Login(_) => "login",
            Self::Logout => "logout",
            Self::Contact(ContactCommand::Check(_)) => "contact:check",
            Self::Contact(ContactCommand::Create(_)) => "contact:create",
            Self::Contact(ContactCommand::Info(_)) => "contact:info",
            Self::Contact(ContactCommand::Update(_)) => "contact:update",
            Self::Contact(ContactCommand::Delete(_)) => "contact:delete",
            Self::Domain(DomainCommand::Check(_)) => "domain:check",
            Self::Domain(DomainCommand::Create(_)) => "domain:create",
            Self::Domain(DomainCommand::Info(_)) => "domain:info",
            Self::Domain(DomainCommand::Update(_)) => "domain:update",
            Self::Domain(DomainCommand::Delete(_)) => "domain:delete",
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
    // Do not route based on the namespace URI text alone: login carries
    // negotiated object URIs as text and may legitimately mention Domain.
    // Domain commands have a Domain-qualified element/namespace declaration.
    if xml
        .windows(b"<domain:".len())
        .any(|window| window == b"<domain:")
        || xml
            .windows(b"xmlns:domain=\"urn:ietf:params:xml:ns:domain-1.0\"".len())
            .any(|window| window == b"xmlns:domain=\"urn:ietf:params:xml:ns:domain-1.0\"")
    {
        return parse_domain_command(xml);
    }
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
    let mut contact_localized_values = std::collections::BTreeMap::new();
    let mut contact_localized_streets = Vec::new();
    let mut postal_info_type: Option<String> = None;
    let mut contact_add_statuses = Vec::new();
    let mut contact_rem_statuses = Vec::new();
    let mut contact_disclose_fields = Vec::new();
    let mut contact_clear_fields = Vec::new();
    let mut root_seen = false;
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(event)) => {
                let name = event.name().as_ref().to_vec();
                if name.ends_with(b"status")
                    && let Some(status) = event
                        .attributes()
                        .flatten()
                        .find(|attribute| attribute.key.as_ref() == b"s")
                        .and_then(|attribute| attribute.unescape_value().ok())
                {
                    if path.iter().any(|part| part.ends_with(b"add")) {
                        contact_add_statuses.push(status.into_owned());
                    } else if path.iter().any(|part| part.ends_with(b"rem")) {
                        contact_rem_statuses.push(status.into_owned());
                    }
                }
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
                            disclose_flag: None,
                            disclose_fields: Vec::new(),
                            localized: None,
                        },
                    )));
                } else if name.ends_with(b"info") {
                    command = Some(EppCommand::Contact(ContactCommand::Info(
                        ContactInfoCommand {
                            id: String::new(),
                            auth_info: None,
                        },
                    )));
                } else if name.ends_with(b"update") {
                    command = Some(EppCommand::Contact(ContactCommand::Update(
                        ContactUpdateCommand {
                            id: String::new(),
                            add_statuses: Vec::new(),
                            rem_statuses: Vec::new(),
                            chg_email: Default::default(),
                            chg_auth_info: Default::default(),
                            chg_voice: Default::default(),
                            chg_fax: Default::default(),
                            chg_organization: Default::default(),
                            chg_city: Default::default(),
                            chg_state_province: Default::default(),
                            chg_postal_code: Default::default(),
                            chg_country_code: Default::default(),
                            chg_streets: Vec::new(),
                            chg_disclose: Default::default(),
                            chg_disclose_fields: Vec::new(),
                        },
                    )));
                } else if name.ends_with(b"delete") {
                    command = Some(EppCommand::Contact(ContactCommand::Delete(
                        ContactDeleteCommand { id: String::new() },
                    )));
                }
                if name.ends_with(b"postalInfo") {
                    postal_info_type = event
                        .attributes()
                        .flatten()
                        .find(|attribute| attribute.key.as_ref() == b"type")
                        .and_then(|attribute| attribute.unescape_value().ok())
                        .map(|value| value.into_owned());
                }
                if name.ends_with(b"disclose")
                    && let Some(flag) = event
                        .attributes()
                        .flatten()
                        .find(|attribute| attribute.key.as_ref() == b"flag")
                        .and_then(|attribute| attribute.unescape_value().ok())
                {
                    contact_create_values.insert("disclose", flag.into_owned());
                }
                path.push(name);
            }
            Ok(Event::Empty(event)) => {
                let name = event.name().as_ref().to_vec();
                if path.iter().any(|part| part.ends_with(b"chg")) {
                    let field = if name.ends_with(b"org") {
                        Some("org")
                    } else if name.ends_with(b"fax") {
                        Some("fax")
                    } else if name.ends_with(b"sp") {
                        Some("sp")
                    } else if name.ends_with(b"pc") {
                        Some("pc")
                    } else {
                        None
                    };
                    if let Some(field) = field {
                        contact_clear_fields.push(field);
                    }
                }
                if path.iter().any(|part| part.ends_with(b"disclose")) {
                    let field = if name.ends_with(b"name") {
                        Some("name")
                    } else if name.ends_with(b"org") {
                        Some("organization")
                    } else if name.ends_with(b"addr") {
                        Some("address")
                    } else if name.ends_with(b"voice") {
                        Some("voice")
                    } else if name.ends_with(b"fax") {
                        Some("fax")
                    } else if name.ends_with(b"email") {
                        Some("email")
                    } else {
                        None
                    };
                    if let Some(field) = field {
                        contact_disclose_fields.push(field.to_owned());
                    }
                }
                if name.ends_with(b"status")
                    && let Some(status) = event
                        .attributes()
                        .flatten()
                        .find(|attribute| attribute.key.as_ref() == b"s")
                        .and_then(|attribute| attribute.unescape_value().ok())
                {
                    if path.iter().any(|part| part.ends_with(b"add")) {
                        contact_add_statuses.push(status.into_owned());
                    } else if path.iter().any(|part| part.ends_with(b"rem")) {
                        contact_rem_statuses.push(status.into_owned());
                    }
                }
                if name.ends_with(b"disclose")
                    && let Some(flag) = event
                        .attributes()
                        .flatten()
                        .find(|attribute| attribute.key.as_ref() == b"flag")
                        .and_then(|attribute| attribute.unescape_value().ok())
                {
                    contact_create_values.insert("disclose", flag.into_owned());
                }
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
                    Some(name) if name.ends_with(b"name") => postal_values(
                        &mut contact_create_values,
                        &mut contact_localized_values,
                        postal_info_type.as_deref(),
                        "name",
                        value,
                    ),
                    Some(name) if name.ends_with(b"org") => postal_values(
                        &mut contact_create_values,
                        &mut contact_localized_values,
                        postal_info_type.as_deref(),
                        "org",
                        value,
                    ),
                    Some(name) if name.ends_with(b"city") => postal_values(
                        &mut contact_create_values,
                        &mut contact_localized_values,
                        postal_info_type.as_deref(),
                        "city",
                        value,
                    ),
                    Some(name) if name.ends_with(b"sp") => postal_values(
                        &mut contact_create_values,
                        &mut contact_localized_values,
                        postal_info_type.as_deref(),
                        "sp",
                        value,
                    ),
                    Some(name) if name.ends_with(b"pc") => postal_values(
                        &mut contact_create_values,
                        &mut contact_localized_values,
                        postal_info_type.as_deref(),
                        "pc",
                        value,
                    ),
                    Some(name) if name.ends_with(b"cc") => postal_values(
                        &mut contact_create_values,
                        &mut contact_localized_values,
                        postal_info_type.as_deref(),
                        "cc",
                        value,
                    ),
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
                    Some(name) if name.ends_with(b"street") => {
                        if postal_info_type.as_deref() == Some("loc") {
                            contact_localized_streets.push(value);
                        } else {
                            contact_streets.push(value);
                        }
                    }
                    _ => {}
                }
            }
            Ok(Event::End(event)) => {
                let name = event.name().as_ref().to_vec();
                if path.iter().any(|part| part.ends_with(b"chg")) {
                    let field = if name.ends_with(b"org") {
                        Some("org")
                    } else if name.ends_with(b"fax") {
                        Some("fax")
                    } else if name.ends_with(b"sp") {
                        Some("sp")
                    } else if name.ends_with(b"pc") {
                        Some("pc")
                    } else {
                        None
                    };
                    if let Some(field) = field
                        && !contact_create_values.contains_key(field)
                        && !contact_clear_fields.contains(&field)
                    {
                        contact_clear_fields.push(field);
                    }
                }
                if name.ends_with(b"postalInfo") {
                    postal_info_type = None;
                }
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
            create.disclose_flag = contact_create_values.remove("disclose");
            create.disclose_fields = contact_disclose_fields;
            if !contact_localized_values.is_empty() || !contact_localized_streets.is_empty() {
                create.localized = Some(ContactPostalInfoCommand {
                    name: contact_localized_values
                        .remove("name")
                        .ok_or(ParseError::Command)?,
                    organization: contact_localized_values.remove("org"),
                    streets: contact_localized_streets,
                    city: contact_localized_values
                        .remove("city")
                        .ok_or(ParseError::Command)?,
                    state_province: contact_localized_values.remove("sp"),
                    postal_code: contact_localized_values.remove("pc"),
                    country_code: contact_localized_values
                        .remove("cc")
                        .ok_or(ParseError::Command)?,
                });
            }
            Ok(ParsedCommand {
                command: EppCommand::Contact(ContactCommand::Create(create)),
                cl_trid,
            })
        }
        Some(EppCommand::Contact(ContactCommand::Info(mut info))) => {
            info.id = contact_ids.first().cloned().ok_or(ParseError::Command)?;
            info.auth_info = contact_create_values.remove("pw");
            Ok(ParsedCommand {
                command: EppCommand::Contact(ContactCommand::Info(info)),
                cl_trid,
            })
        }
        Some(EppCommand::Contact(ContactCommand::Update(mut update))) => {
            update.id = contact_ids.first().cloned().ok_or(ParseError::Command)?;
            update.add_statuses = contact_add_statuses;
            update.rem_statuses = contact_rem_statuses;
            let mut patch = |key: &str| match contact_create_values.remove(key) {
                Some(value) => crate::domain::contact::Patch::Set(value),
                None if contact_clear_fields.contains(&key) => crate::domain::contact::Patch::Clear,
                None => crate::domain::contact::Patch::Unchanged,
            };
            update.chg_email = patch("email");
            update.chg_auth_info = patch("pw");
            update.chg_voice = patch("voice");
            update.chg_fax = patch("fax");
            update.chg_organization = patch("org");
            update.chg_city = patch("city");
            update.chg_state_province = patch("sp");
            update.chg_postal_code = patch("pc");
            update.chg_country_code = patch("cc");
            update.chg_streets = contact_streets;
            update.chg_disclose = contact_create_values
                .remove("disclose")
                .map(crate::domain::contact::Patch::Set)
                .unwrap_or_default();
            update.chg_disclose_fields = contact_disclose_fields;
            if update.add_statuses.is_empty()
                && update.rem_statuses.is_empty()
                && update.chg_email.is_unchanged()
                && update.chg_auth_info.is_unchanged()
                && update.chg_voice.is_unchanged()
                && update.chg_fax.is_unchanged()
                && update.chg_organization.is_unchanged()
                && update.chg_city.is_unchanged()
                && update.chg_state_province.is_unchanged()
                && update.chg_postal_code.is_unchanged()
                && update.chg_country_code.is_unchanged()
                && update.chg_streets.is_empty()
                && update.chg_disclose.is_unchanged()
                && update.chg_disclose_fields.is_empty()
            {
                return Err(ParseError::Command);
            }
            Ok(ParsedCommand {
                command: EppCommand::Contact(ContactCommand::Update(update)),
                cl_trid,
            })
        }
        Some(EppCommand::Contact(ContactCommand::Delete(mut delete))) => {
            delete.id = contact_ids.first().cloned().ok_or(ParseError::Command)?;
            Ok(ParsedCommand {
                command: EppCommand::Contact(ContactCommand::Delete(delete)),
                cl_trid,
            })
        }
        Some(EppCommand::Domain(_)) => Err(ParseError::Unsupported),
        None if xml.windows(9).any(|window| window == b"<command>") => Err(ParseError::Unsupported),
        None => Err(ParseError::Command),
    }
}

fn parse_domain_command(xml: &[u8]) -> Result<ParsedCommand, ParseError> {
    let mut reader = Reader::from_reader(xml);
    reader.config_mut().trim_text(true);
    let mut buf = Vec::new();
    let mut path: Vec<Vec<u8>> = Vec::new();
    let mut root_seen = false;
    let mut cl_trid = None;
    let mut command: Option<DomainCommand> = None;
    let mut names = Vec::new();
    let mut domain_name = None;
    let mut nameservers = Vec::new();
    let mut registrant = None;
    let mut contacts = Vec::new();
    let mut auth_info = None;
    let mut period = None;
    let mut period_value = None;
    let mut period_unit = None;
    let mut current_contact_role = None;
    let mut add_nameservers = Vec::new();
    let mut rem_nameservers = Vec::new();
    let mut add_contacts = Vec::new();
    let mut rem_contacts = Vec::new();
    let mut add_statuses = Vec::new();
    let mut rem_statuses = Vec::new();
    let mut chg_registrant = crate::domain::contact::Patch::Unchanged;
    let mut chg_auth_info = crate::domain::contact::Patch::Unchanged;
    let mut in_add = false;
    let mut in_rem = false;
    let mut in_chg = false;
    let mut domain_namespace_seen = false;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(event)) => {
                let name = event.name().as_ref().to_vec();
                if event
                    .attributes()
                    .flatten()
                    .filter_map(|attribute| attribute.unescape_value().ok())
                    .any(|value| value == DOMAIN_NS)
                {
                    domain_namespace_seen = true;
                }
                if !root_seen {
                    root_seen = true;
                    if name != b"epp"
                        || event
                            .attributes()
                            .flatten()
                            .find(|attribute| attribute.key.as_ref() == b"xmlns")
                            .and_then(|attribute| attribute.unescape_value().ok())
                            .as_deref()
                            != Some(EPP_NS)
                    {
                        return Err(ParseError::Namespace);
                    }
                }
                let local = name.rsplit(|byte| *byte == b':').next().unwrap_or(&name);
                if local == b"hostObj" || local == b"hostAddr" {
                    return Err(ParseError::Unsupported);
                }
                if matches!(
                    local,
                    b"check" | b"create" | b"info" | b"update" | b"delete"
                ) && !name.ends_with(b"command")
                {
                    command = Some(match local {
                        b"check" => DomainCommand::Check(DomainCheckCommand { names: Vec::new() }),
                        b"create" => DomainCommand::Create(DomainCreateCommand {
                            name: String::new(),
                            period: None,
                            nameservers: Vec::new(),
                            registrant: None,
                            contacts: Vec::new(),
                            auth_info: String::new(),
                        }),
                        b"info" => DomainCommand::Info(DomainInfoCommand {
                            name: String::new(),
                            auth_info: None,
                        }),
                        b"update" => DomainCommand::Update(DomainUpdateCommand {
                            name: String::new(),
                            add_nameservers: Vec::new(),
                            rem_nameservers: Vec::new(),
                            add_contacts: Vec::new(),
                            rem_contacts: Vec::new(),
                            add_statuses: Vec::new(),
                            rem_statuses: Vec::new(),
                            chg_registrant: Default::default(),
                            chg_auth_info: Default::default(),
                        }),
                        b"delete" => DomainCommand::Delete(DomainDeleteCommand {
                            name: String::new(),
                        }),
                        _ => unreachable!(),
                    });
                }
                if local == b"period" {
                    period_unit = event
                        .attributes()
                        .flatten()
                        .find(|attribute| attribute.key.as_ref() == b"unit")
                        .and_then(|attribute| attribute.unescape_value().ok())
                        .map(|value| value.into_owned());
                }
                if local == b"contact" {
                    current_contact_role = event
                        .attributes()
                        .flatten()
                        .find(|attribute| attribute.key.as_ref() == b"type")
                        .and_then(|attribute| attribute.unescape_value().ok())
                        .map(|value| value.into_owned());
                }
                if local == b"add" {
                    in_add = true;
                } else if local == b"rem" {
                    in_rem = true;
                } else if local == b"chg" {
                    in_chg = true;
                }
                path.push(name);
            }
            Ok(Event::Text(text)) => {
                let value = text
                    .decode()
                    .map_err(|error| ParseError::Xml(error.to_string()))?
                    .into_owned();
                let local = path
                    .last()
                    .and_then(|name| name.rsplit(|byte| *byte == b':').next())
                    .unwrap_or_default();
                if local == b"clTRID" {
                    cl_trid = Some(value);
                } else if local == b"name" {
                    match command {
                        Some(DomainCommand::Check(_)) => names.push(value),
                        Some(DomainCommand::Create(_))
                        | Some(DomainCommand::Info(_))
                        | Some(DomainCommand::Delete(_))
                        | Some(DomainCommand::Update(_))
                            if path.iter().any(|part| part.ends_with(b"domain:name")) =>
                        {
                            domain_name = Some(value);
                        }
                        _ => {}
                    }
                } else if local == b"hostName" {
                    if in_rem {
                        rem_nameservers.push(value);
                    } else if in_add {
                        add_nameservers.push(value);
                    } else {
                        nameservers.push(value);
                    }
                } else if local == b"registrant" {
                    if in_chg {
                        chg_registrant = crate::domain::contact::Patch::Set(value);
                    } else {
                        registrant = Some(value);
                    }
                } else if local == b"contact" && current_contact_role.is_some() {
                    let role = current_contact_role.clone().unwrap_or_default();
                    let contact = DomainContactCommand { id: value, role };
                    if matches!(command.as_ref(), Some(DomainCommand::Create(_))) {
                        contacts.push(contact);
                    } else if in_rem {
                        rem_contacts.push(contact);
                    } else {
                        add_contacts.push(contact);
                    }
                } else if local == b"pw" {
                    if in_chg {
                        chg_auth_info = crate::domain::contact::Patch::Set(value);
                    } else {
                        auth_info = Some(value);
                    }
                } else if local == b"period" {
                    period_value = value.parse::<u32>().ok();
                } else if local == b"status" {
                    let target = if in_rem {
                        &mut rem_statuses
                    } else {
                        &mut add_statuses
                    };
                    target.push(value);
                }
            }
            Ok(Event::End(event)) => {
                let name = event.name().as_ref().to_vec();
                let local = name.rsplit(|byte| *byte == b':').next().unwrap_or(&name);
                if local == b"contact" {
                    current_contact_role = None;
                } else if local == b"add" {
                    in_add = false;
                } else if local == b"rem" {
                    in_rem = false;
                } else if local == b"chg" {
                    in_chg = false;
                }
                if local == b"period" {
                    period = Some(DomainPeriodCommand {
                        value: period_value.take().ok_or(ParseError::Command)?,
                        unit: period_unit.take().ok_or(ParseError::Command)?,
                    });
                }
                path.pop();
            }
            Ok(Event::Empty(event)) => {
                let name = event.name().as_ref().to_vec();
                let local = name.rsplit(|byte| *byte == b':').next().unwrap_or(&name);
                if local == b"hostObj" || local == b"hostAddr" {
                    return Err(ParseError::Unsupported);
                }
                if local == b"status"
                    && let Some(status) = event
                        .attributes()
                        .flatten()
                        .find(|attribute| attribute.key.as_ref() == b"s")
                        .and_then(|attribute| attribute.unescape_value().ok())
                {
                    if in_rem {
                        rem_statuses.push(status.into_owned());
                    } else {
                        add_statuses.push(status.into_owned());
                    }
                }
            }
            Ok(Event::Eof) => break,
            Err(error) => return Err(ParseError::Xml(error.to_string())),
            _ => {}
        }
        buf.clear();
    }

    if !domain_namespace_seen {
        return Err(ParseError::Namespace);
    }

    let command = match command.ok_or(ParseError::Command)? {
        DomainCommand::Check(_) => DomainCommand::Check(DomainCheckCommand { names }),
        DomainCommand::Create(_) => DomainCommand::Create(DomainCreateCommand {
            name: domain_name.ok_or(ParseError::Command)?,
            period,
            nameservers,
            registrant,
            contacts,
            auth_info: auth_info.ok_or(ParseError::Command)?,
        }),
        DomainCommand::Info(_) => DomainCommand::Info(DomainInfoCommand {
            name: domain_name.ok_or(ParseError::Command)?,
            auth_info,
        }),
        DomainCommand::Update(_) => {
            if add_nameservers.is_empty()
                && rem_nameservers.is_empty()
                && add_contacts.is_empty()
                && rem_contacts.is_empty()
                && add_statuses.is_empty()
                && rem_statuses.is_empty()
                && chg_registrant.is_unchanged()
                && chg_auth_info.is_unchanged()
            {
                return Err(ParseError::Command);
            }
            DomainCommand::Update(DomainUpdateCommand {
                name: domain_name.ok_or(ParseError::Command)?,
                add_nameservers,
                rem_nameservers,
                add_contacts,
                rem_contacts,
                add_statuses,
                rem_statuses,
                chg_registrant,
                chg_auth_info,
            })
        }
        DomainCommand::Delete(_) => DomainCommand::Delete(DomainDeleteCommand {
            name: domain_name.ok_or(ParseError::Command)?,
        }),
    };
    Ok(ParsedCommand {
        command: EppCommand::Domain(command),
        cl_trid,
    })
}

fn postal_values(
    international: &mut std::collections::BTreeMap<&'static str, String>,
    localized: &mut std::collections::BTreeMap<&'static str, String>,
    postal_info_type: Option<&str>,
    key: &'static str,
    value: String,
) {
    if postal_info_type == Some("loc") {
        localized.insert(key, value);
    } else {
        international.insert(key, value);
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
    fn login_can_negotiate_domain_without_being_parsed_as_domain_command() {
        let xml = br#"<epp xmlns="urn:ietf:params:xml:ns:epp-1.0"><command><login><clID>REG-1</clID><pw>secret</pw><options><version>1.0</version><lang>en</lang></options><svcs><objURI>urn:ietf:params:xml:ns:domain-1.0</objURI></svcs></login><clTRID>abc</clTRID></command></epp>"#;
        assert!(matches!(
            parse_command(xml).unwrap().command,
            EppCommand::Login(_)
        ));
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
            br#"<epp xmlns="urn:ietf:params:xml:ns:epp-1.0"><command><info xmlns:contact="urn:ietf:params:xml:ns:contact-1.0"><contact:info><contact:id>C123</contact:id></contact:info></info></command></epp>"#,
        )
        .unwrap();
        assert_eq!(parsed.name(), "contact:info");
        assert_eq!(
            parsed.command,
            EppCommand::Contact(ContactCommand::Info(ContactInfoCommand {
                id: "C123".into(),
                auth_info: None,
            }))
        );
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
    fn parses_contact_update_email_patch() {
        let parsed = parse_command(
            br#"<epp xmlns="urn:ietf:params:xml:ns:epp-1.0"><command><update xmlns:contact="urn:ietf:params:xml:ns:contact-1.0"><contact:update><contact:id>C123</contact:id><contact:chg><contact:email>new@example.test</contact:email></contact:chg></contact:update></update></command></epp>"#,
        )
        .unwrap();
        assert_eq!(
            parsed.command,
            EppCommand::Contact(ContactCommand::Update(ContactUpdateCommand {
                id: "C123".into(),
                add_statuses: vec![],
                rem_statuses: vec![],
                chg_email: crate::domain::contact::Patch::Set("new@example.test".into()),
                chg_auth_info: crate::domain::contact::Patch::Unchanged,
                chg_voice: crate::domain::contact::Patch::Unchanged,
                chg_fax: crate::domain::contact::Patch::Unchanged,
                chg_organization: crate::domain::contact::Patch::Unchanged,
                chg_city: crate::domain::contact::Patch::Unchanged,
                chg_state_province: crate::domain::contact::Patch::Unchanged,
                chg_postal_code: crate::domain::contact::Patch::Unchanged,
                chg_country_code: crate::domain::contact::Patch::Unchanged,
                chg_streets: vec![],
                chg_disclose: crate::domain::contact::Patch::Unchanged,
                chg_disclose_fields: vec![],
            }))
        );
    }

    #[test]
    fn parses_localized_postal_info_in_contact_create() {
        let parsed = parse_command(
            r#"<epp xmlns="urn:ietf:params:xml:ns:epp-1.0"><command><create><contact:create xmlns:contact="urn:ietf:params:xml:ns:contact-1.0"><contact:id>C123</contact:id><contact:postalInfo type="int"><contact:name>International Name</contact:name><contact:addr><contact:street>Main 1</contact:street><contact:city>Moscow</contact:city><contact:cc>RU</contact:cc></contact:addr></contact:postalInfo><contact:postalInfo type="loc"><contact:name>Локальное имя</contact:name><contact:org>Компания</contact:org><contact:addr><contact:street>Улица 1</contact:street><contact:city>Москва</contact:city><contact:cc>RU</contact:cc></contact:addr></contact:postalInfo><contact:voice>+70000000000</contact:voice><contact:email>contact@example.test</contact:email><contact:authInfo><contact:pw>secret</contact:pw></contact:authInfo></contact:create></create></command></epp>"#.as_bytes(),
        )
        .unwrap();
        let EppCommand::Contact(ContactCommand::Create(create)) = parsed.command else {
            panic!("expected contact:create");
        };
        let localized = create.localized.expect("localized postal info");
        assert_eq!(localized.name, "Локальное имя");
        assert_eq!(localized.organization.as_deref(), Some("Компания"));
        assert_eq!(localized.streets, ["Улица 1"]);
    }

    #[test]
    fn parses_contact_update_auth_info_patch() {
        let parsed = parse_command(
            br#"<epp xmlns="urn:ietf:params:xml:ns:epp-1.0"><command><update xmlns:contact="urn:ietf:params:xml:ns:contact-1.0"><contact:update><contact:id>C123</contact:id><contact:chg><contact:authInfo><contact:pw>new-secret</contact:pw></contact:authInfo></contact:chg></contact:update></update></command></epp>"#,
        )
        .unwrap();
        assert_eq!(
            parsed.command,
            EppCommand::Contact(ContactCommand::Update(ContactUpdateCommand {
                id: "C123".into(),
                add_statuses: vec![],
                rem_statuses: vec![],
                chg_email: crate::domain::contact::Patch::Unchanged,
                chg_auth_info: crate::domain::contact::Patch::Set("new-secret".into()),
                chg_voice: crate::domain::contact::Patch::Unchanged,
                chg_fax: crate::domain::contact::Patch::Unchanged,
                chg_organization: crate::domain::contact::Patch::Unchanged,
                chg_city: crate::domain::contact::Patch::Unchanged,
                chg_state_province: crate::domain::contact::Patch::Unchanged,
                chg_postal_code: crate::domain::contact::Patch::Unchanged,
                chg_country_code: crate::domain::contact::Patch::Unchanged,
                chg_streets: vec![],
                chg_disclose: crate::domain::contact::Patch::Unchanged,
                chg_disclose_fields: vec![],
            }))
        );
        let redacted = redact_password(
            br#"<epp><command><update><contact:pw>new-secret</contact:pw></update></command></epp>"#,
        )
        .unwrap();
        assert!(!redacted.contains("new-secret"));
    }

    #[test]
    fn parses_contact_update_clear_patches() {
        let parsed = parse_command(
            br#"<epp xmlns="urn:ietf:params:xml:ns:epp-1.0"><command><update xmlns:contact="urn:ietf:params:xml:ns:contact-1.0"><contact:update><contact:id>C123</contact:id><contact:chg><contact:postalInfo type="int"><contact:org></contact:org><contact:addr><contact:sp/><contact:pc/></contact:addr></contact:postalInfo><contact:fax/></contact:chg></contact:update></update></command></epp>"#,
        )
        .unwrap();
        let EppCommand::Contact(ContactCommand::Update(update)) = parsed.command else {
            panic!("expected contact:update");
        };
        assert_eq!(
            update.chg_organization,
            crate::domain::contact::Patch::Clear
        );
        assert_eq!(
            update.chg_state_province,
            crate::domain::contact::Patch::Clear
        );
        assert_eq!(update.chg_postal_code, crate::domain::contact::Patch::Clear);
        assert_eq!(update.chg_fax, crate::domain::contact::Patch::Clear);
    }

    #[test]
    fn parses_contact_update_client_status_add_and_rem() {
        let parsed = parse_command(
            br#"<epp xmlns="urn:ietf:params:xml:ns:epp-1.0"><command><update xmlns:contact="urn:ietf:params:xml:ns:contact-1.0"><contact:update><contact:id>C123</contact:id><contact:add><contact:status s="clientUpdateProhibited"/></contact:add><contact:rem><contact:status s="clientDeleteProhibited"/></contact:rem></contact:update></update></command></epp>"#,
        )
        .unwrap();
        let EppCommand::Contact(ContactCommand::Update(update)) = parsed.command else {
            panic!("expected contact update");
        };
        assert_eq!(update.add_statuses, vec!["clientUpdateProhibited"]);
        assert_eq!(update.rem_statuses, vec!["clientDeleteProhibited"]);
    }

    #[test]
    fn parses_contact_create_required_and_optional_fields() {
        let parsed = parse_command(
            br#"<epp xmlns="urn:ietf:params:xml:ns:epp-1.0"><command><create><contact:create xmlns:contact="urn:ietf:params:xml:ns:contact-1.0"><contact:id>C123</contact:id><contact:postalInfo type="int"><contact:name>Name</contact:name><contact:org>Org</contact:org><contact:addr><contact:street>Main 1</contact:street><contact:city>Moscow</contact:city><contact:cc>RU</contact:cc></contact:addr></contact:postalInfo><contact:voice x="123">+70000000000</contact:voice><contact:email>a@example.test</contact:email><contact:authInfo><contact:pw>secret</contact:pw></contact:authInfo><contact:disclose flag="1"><contact:email/></contact:disclose></contact:create></create></command></epp>"#,
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
        assert_eq!(create.disclose_flag.as_deref(), Some("1"));
        assert_eq!(create.disclose_fields, ["email"]);
    }

    #[test]
    fn parses_domain_batch_check() {
        let parsed = parse_command(
            br#"<epp xmlns="urn:ietf:params:xml:ns:epp-1.0"><command><check><domain:check xmlns:domain="urn:ietf:params:xml:ns:domain-1.0"><domain:name>one.com</domain:name><domain:name>two.net</domain:name></domain:check></check><clTRID>check-1</clTRID></command></epp>"#,
        )
        .unwrap();
        assert_eq!(
            parsed.command,
            EppCommand::Domain(DomainCommand::Check(DomainCheckCommand {
                names: vec!["one.com".into(), "two.net".into()]
            }))
        );
        assert_eq!(parsed.cl_trid.as_deref(), Some("check-1"));
    }

    #[test]
    fn parses_domain_create_with_period_contacts_and_host_attrs() {
        let parsed = parse_command(
            br#"<epp xmlns="urn:ietf:params:xml:ns:epp-1.0"><command><create><domain:create xmlns:domain="urn:ietf:params:xml:ns:domain-1.0"><domain:name>example.com</domain:name><domain:period unit="y">2</domain:period><domain:ns><domain:hostAttr><domain:hostName>ns1.example.net</domain:hostName></domain:hostAttr><domain:hostAttr><domain:hostName>ns2.example.net</domain:hostName></domain:hostAttr></domain:ns><domain:registrant>C123</domain:registrant><domain:contact type="admin">C124</domain:contact><domain:authInfo><domain:pw>secret</domain:pw></domain:authInfo></domain:create></create></command></epp>"#,
        )
        .unwrap();
        let EppCommand::Domain(DomainCommand::Create(create)) = parsed.command else {
            panic!("expected domain:create");
        };
        assert_eq!(create.name, "example.com");
        assert_eq!(
            create.period,
            Some(DomainPeriodCommand {
                value: 2,
                unit: "y".into()
            })
        );
        assert_eq!(create.nameservers, ["ns1.example.net", "ns2.example.net"]);
        assert_eq!(create.registrant.as_deref(), Some("C123"));
        assert_eq!(
            create.contacts,
            [DomainContactCommand {
                id: "C124".into(),
                role: "admin".into()
            }]
        );
        assert_eq!(create.auth_info, "secret");
    }

    #[test]
    fn parses_domain_info_auth_info_and_update_operations() {
        let info = parse_command(
            br#"<epp xmlns="urn:ietf:params:xml:ns:epp-1.0"><command><info><domain:info xmlns:domain="urn:ietf:params:xml:ns:domain-1.0"><domain:name>example.com</domain:name><domain:authInfo><domain:pw>secret</domain:pw></domain:authInfo></domain:info></info></command></epp>"#,
        )
        .unwrap();
        assert_eq!(
            info.command,
            EppCommand::Domain(DomainCommand::Info(DomainInfoCommand {
                name: "example.com".into(),
                auth_info: Some("secret".into())
            }))
        );

        let update = parse_command(
            br#"<epp xmlns="urn:ietf:params:xml:ns:epp-1.0"><command><update><domain:update xmlns:domain="urn:ietf:params:xml:ns:domain-1.0"><domain:name>example.com</domain:name><domain:add><domain:ns><domain:hostAttr><domain:hostName>ns3.example.net</domain:hostName></domain:hostAttr></domain:ns><domain:contact type="tech">C125</domain:contact><domain:status s="clientHold"/></domain:add><domain:rem><domain:contact type="admin">C124</domain:contact><domain:status s="clientTransferProhibited"/></domain:rem><domain:chg><domain:registrant>C126</domain:registrant><domain:authInfo><domain:pw>new-secret</domain:pw></domain:authInfo></domain:chg></domain:update></update></command></epp>"#,
        )
        .unwrap();
        let EppCommand::Domain(DomainCommand::Update(update)) = update.command else {
            panic!("expected domain:update");
        };
        assert_eq!(update.name, "example.com");
        assert_eq!(update.add_nameservers, ["ns3.example.net"]);
        assert_eq!(update.add_contacts[0].role, "tech");
        assert_eq!(update.rem_contacts[0].id, "C124");
        assert_eq!(update.add_statuses, ["clientHold"]);
        assert_eq!(update.rem_statuses, ["clientTransferProhibited"]);
        assert_eq!(
            update.chg_registrant,
            crate::domain::contact::Patch::Set("C126".into())
        );
        assert_eq!(
            update.chg_auth_info,
            crate::domain::contact::Patch::Set("new-secret".into())
        );
    }

    #[test]
    fn parses_domain_delete_and_rejects_host_objects_or_empty_update() {
        let delete = parse_command(
            br#"<epp xmlns="urn:ietf:params:xml:ns:epp-1.0"><command><delete><domain:delete xmlns:domain="urn:ietf:params:xml:ns:domain-1.0"><domain:name>example.com</domain:name></domain:delete></delete></command></epp>"#,
        )
        .unwrap();
        assert_eq!(
            delete.command,
            EppCommand::Domain(DomainCommand::Delete(DomainDeleteCommand {
                name: "example.com".into()
            }))
        );

        assert!(matches!(
            parse_command(br#"<epp xmlns="urn:ietf:params:xml:ns:epp-1.0"><command><create><domain:create xmlns:domain="urn:ietf:params:xml:ns:domain-1.0"><domain:name>example.com</domain:name><domain:hostObj>ns1.example.net</domain:hostObj><domain:authInfo><domain:pw>secret</domain:pw></domain:authInfo></domain:create></create></command></epp>"#),
            Err(ParseError::Unsupported)
        ));
        assert!(matches!(
            parse_command(br#"<epp xmlns="urn:ietf:params:xml:ns:epp-1.0"><command><update><domain:update xmlns:domain="urn:ietf:params:xml:ns:domain-1.0"><domain:name>example.com</domain:name></domain:update></update></command></epp>"#),
            Err(ParseError::Command)
        ));
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
