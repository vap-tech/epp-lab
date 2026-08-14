use std::sync::Arc;

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
