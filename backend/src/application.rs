use std::sync::Arc;

use sqlx::PgPool;
use thiserror::Error;

use crate::{
    domain::extension::{ExtensionKey, ExtensionRegistry, ZoneExtensionAssignment},
    storage::zone,
};

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
