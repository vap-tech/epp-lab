use std::{convert::Infallible, sync::Arc};

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use thiserror::Error;

use crate::{
    domain::extension::{ExtensionKey, ExtensionRegistry, ZoneExtensionAssignment},
    storage::zone,
};

#[derive(Debug, Error)]
pub(crate) enum ZoneCommandError {
    #[error("invalid zone name: {0}")]
    InvalidName(String),
    #[error("zone already exists")]
    AlreadyExists,
    #[error("zone not found")]
    NotFound,
    #[error("database error: {0}")]
    Database(#[source] sqlx::Error),
}

#[allow(dead_code)]
#[derive(Debug, Error)]
pub(crate) enum ContactCreateError {
    #[error("invalid contact data: {0}")]
    InvalidData(String),
    #[error("authInfo encryption failed: {0}")]
    Encryption(#[source] crate::security::SecretCipherError),
}

/// Stage 4's fixed disclosure policy accepts every RFC-valid preference.
/// Keeping this decision at the application boundary makes later registry
/// policy changes independent of the XML parser.
fn contact_disclosure(
    flag: Option<&str>,
    fields: &[String],
) -> Result<crate::domain::contact::DisclosurePreference, ContactCreateError> {
    use crate::domain::contact::{DisclosureField, DisclosureFlag, DisclosurePreference};

    let flag = match flag.unwrap_or("0") {
        "0" => DisclosureFlag::Private,
        "1" => DisclosureFlag::Public,
        _ => {
            return Err(ContactCreateError::InvalidData(
                "invalid disclose flag".to_owned(),
            ));
        }
    };
    let mut parsed = std::collections::BTreeSet::new();
    for field in fields {
        let field = match field.as_str() {
            "name" => DisclosureField::Name,
            "organization" => DisclosureField::Organization,
            "address" => DisclosureField::Address,
            "voice" => DisclosureField::Voice,
            "fax" => DisclosureField::Fax,
            "email" => DisclosureField::Email,
            _ => {
                return Err(ContactCreateError::InvalidData(
                    "invalid disclose field".to_owned(),
                ));
            }
        };
        parsed.insert(field);
    }
    Ok(DisclosurePreference {
        flag,
        fields: parsed,
    })
}

#[allow(dead_code)]
pub(crate) fn prepare_contact_create(
    command: &crate::epp::parser::ContactCreateCommand,
    registrar_id: uuid::Uuid,
    cipher: &dyn crate::security::SecretCipher,
    now: DateTime<Utc>,
) -> Result<crate::domain::contact::Contact, ContactCreateError> {
    use crate::domain::contact::{
        Contact, ContactId, ContactRoid, CountryCode, EmailAddress, PhoneNumber, PostalAddress,
        PostalInfo, PostalInfoSet,
    };
    let auth_info = cipher
        .encrypt(command.auth_info.as_bytes())
        .map_err(ContactCreateError::Encryption)?;
    let address = PostalAddress {
        streets: command.streets.clone(),
        city: command.city.clone(),
        state_province: command.state_province.clone(),
        postal_code: command.postal_code.clone(),
        country_code: CountryCode::parse(&command.country_code)
            .map_err(|error| ContactCreateError::InvalidData(error.to_string()))?,
    };
    let localized = command
        .localized
        .as_ref()
        .map(|postal| {
            Ok(PostalInfo {
                name: postal.name.clone(),
                organization: postal.organization.clone(),
                address: PostalAddress {
                    streets: postal.streets.clone(),
                    city: postal.city.clone(),
                    state_province: postal.state_province.clone(),
                    postal_code: postal.postal_code.clone(),
                    country_code: CountryCode::parse(&postal.country_code)
                        .map_err(|error| ContactCreateError::InvalidData(error.to_string()))?,
                },
            })
        })
        .transpose()?;
    let contact = Contact {
        id: ContactId::new(uuid::Uuid::new_v4()),
        roid: ContactRoid::parse(&command.id)
            .map_err(|error| ContactCreateError::InvalidData(error.to_string()))?,
        postal_info: PostalInfoSet {
            international: PostalInfo {
                name: command.name.clone(),
                organization: command.organization.clone(),
                address,
            },
            localized,
        },
        voice: PhoneNumber {
            number: command.voice.clone(),
            extension: command.voice_extension.clone(),
        },
        fax: command.fax.clone().map(|number| PhoneNumber {
            number,
            extension: command.fax_extension.clone(),
        }),
        email: EmailAddress::parse(&command.email)
            .map_err(|error| ContactCreateError::InvalidData(error.to_string()))?,
        auth_info,
        disclose: contact_disclosure(command.disclose_flag.as_deref(), &command.disclose_fields)?,
        client_statuses: Default::default(),
        // `ok` is derived when no stored status applies. A pending status
        // would claim an asynchronous lifecycle that does not exist.
        server_statuses: Default::default(),
        sponsoring_registrar_id: registrar_id,
        created_by: registrar_id,
        created_at: now,
        updated_by: registrar_id,
        updated_at: now,
        transferred_at: None,
    };
    contact
        .validate()
        .map_err(|error| ContactCreateError::InvalidData(error.to_string()))?;
    Ok(contact)
}

pub(crate) fn effective_contact_statuses(persisted: &[String], linked: bool) -> Vec<String> {
    let parsed = persisted
        .iter()
        .filter_map(|status| crate::domain::contact::ContactStatus::parse(status))
        .collect::<Vec<_>>();
    crate::domain::contact::effective_statuses(parsed, linked)
        .into_iter()
        .map(crate::domain::contact::ContactStatus::as_str)
        .map(str::to_owned)
        .collect()
}

/// Application boundary for checking whether another registry aggregate owns a
/// live reference to a Contact. Stage 4 has no such aggregate yet, so the
/// concrete lookup is deliberately explicit rather than backed by a fictitious
/// table. A future domain/host repository can replace this implementation.
pub(crate) trait ContactAssociationLookup {
    async fn has_active_links(&self, contact_id: uuid::Uuid) -> Result<bool, Infallible>;
}

pub(crate) struct NoContactAssociations;

impl ContactAssociationLookup for NoContactAssociations {
    async fn has_active_links(&self, _contact_id: uuid::Uuid) -> Result<bool, Infallible> {
        Ok(false)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ContactCheckResult {
    pub available: bool,
}

#[allow(dead_code)]
pub(crate) async fn check_contact(
    db: &PgPool,
    roid: &str,
) -> Result<ContactCheckResult, sqlx::Error> {
    Ok(ContactCheckResult {
        available: !crate::storage::contact::exists_by_roid(db, roid).await?,
    })
}

pub(crate) async fn create_zone(
    db: &PgPool,
    name: &str,
    now: DateTime<Utc>,
) -> Result<crate::domain::zone::Zone, ZoneCommandError> {
    let zone = crate::domain::zone::Zone {
        id: crate::domain::zone::ZoneId::new(uuid::Uuid::new_v4()),
        name: crate::domain::zone::ZoneName::parse(name)
            .map_err(|error| ZoneCommandError::InvalidName(error.to_string()))?,
        status: crate::domain::zone::ZoneStatus::Active,
        contact_policy: Default::default(),
    };
    zone::create(db, &zone, now).await.map_err(|error| {
        if let sqlx::Error::Database(database_error) = &error
            && database_error.constraint() == Some("zones_ascii_name_key")
        {
            ZoneCommandError::AlreadyExists
        } else {
            ZoneCommandError::Database(error)
        }
    })?;
    Ok(zone)
}

pub(crate) async fn update_zone_status(
    db: &PgPool,
    id: uuid::Uuid,
    status: crate::domain::zone::ZoneStatus,
    now: DateTime<Utc>,
) -> Result<(), ZoneCommandError> {
    let value = match status {
        crate::domain::zone::ZoneStatus::Active => "active",
        crate::domain::zone::ZoneStatus::Disabled => "disabled",
    };
    if zone::update_status(db, id, value, now)
        .await
        .map_err(ZoneCommandError::Database)?
    {
        Ok(())
    } else {
        Err(ZoneCommandError::NotFound)
    }
}

pub(crate) async fn update_zone_contact_policy(
    db: &PgPool,
    id: uuid::Uuid,
    policy: crate::domain::zone::ContactUsagePolicy,
    now: DateTime<Utc>,
) -> Result<(), ZoneCommandError> {
    if zone::update_contact_policy(db, id, policy, now)
        .await
        .map_err(ZoneCommandError::Database)?
    {
        Ok(())
    } else {
        Err(ZoneCommandError::NotFound)
    }
}

#[derive(Debug, Error)]
pub(crate) enum CapabilityError {
    #[error("failed to load zones: {0}")]
    Zones(#[source] sqlx::Error),
    #[error("failed to load zone extensions: {0}")]
    Extensions(#[source] sqlx::Error),
    #[error("invalid persisted extension key: {0}")]
    InvalidExtensionKey(String),
    #[error("invalid persisted zone: {0}")]
    InvalidZone(String),
}

/// Computes the extensions advertised to newly opened EPP sessions.
///
/// Storage access stays at this application boundary; the registry itself
/// only applies the synchronous capability rules to domain values.
pub(crate) async fn advertised_extension_uris(
    db: &PgPool,
    registry: &Arc<ExtensionRegistry>,
) -> Result<Vec<String>, CapabilityError> {
    let rows = zone::list(db).await.map_err(CapabilityError::Zones)?;
    let mut zones = Vec::with_capacity(rows.len());
    let extension_rows = zone::list_all_extensions(db)
        .await
        .map_err(CapabilityError::Extensions)?;
    let mut assignments = Vec::with_capacity(extension_rows.len());

    for row in rows {
        zones.push(zone::to_domain(row).map_err(CapabilityError::InvalidZone)?);
    }

    for extension in extension_rows {
        let extension_key = ExtensionKey::parse(&extension.extension_key)
            .map_err(|error| CapabilityError::InvalidExtensionKey(error.to_string()))?;
        assignments.push(ZoneExtensionAssignment {
            zone_id: crate::domain::zone::ZoneId::new(extension.zone_id),
            extension_key,
            enabled: extension.enabled,
        });
    }

    Ok(registry
        .advertised_namespaces(zones.iter(), assignments.iter())
        .into_iter()
        .map(str::to_owned)
        .collect())
}

#[cfg(test)]
mod tests {
    use chrono::Utc;

    use super::*;

    #[test]
    fn contact_create_encrypts_auth_info_before_building_aggregate() {
        let command = crate::epp::parser::ContactCreateCommand {
            id: "SH8013".into(),
            name: "Name".into(),
            organization: None,
            streets: vec!["Main 1".into()],
            city: "Moscow".into(),
            state_province: None,
            postal_code: None,
            country_code: "RU".into(),
            voice: "+70000000000".into(),
            voice_extension: None,
            fax: None,
            fax_extension: None,
            email: "a@example.test".into(),
            auth_info: "plain-auth-info".into(),
            disclose_flag: None,
            disclose_fields: vec![],
            localized: None,
        };
        let cipher = crate::security::AesGcmSecretCipher::from_hex(
            "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f",
        )
        .unwrap();
        let contact =
            prepare_contact_create(&command, uuid::Uuid::new_v4(), &cipher, Utc::now()).unwrap();
        assert_ne!(contact.auth_info, command.auth_info);
        assert_eq!(
            crate::security::SecretCipher::decrypt(&cipher, &contact.auth_info).unwrap(),
            command.auth_info.as_bytes()
        );
        assert!(contact.server_statuses.is_empty());
        assert!(
            !contact
                .server_statuses
                .contains(&crate::domain::contact::ContactStatus::PendingCreate)
        );
    }

    #[tokio::test]
    async fn contact_association_boundary_is_honest_without_linkable_objects() {
        let lookup = NoContactAssociations;
        assert!(
            !ContactAssociationLookup::has_active_links(&lookup, uuid::Uuid::new_v4())
                .await
                .unwrap()
        );
    }
}
