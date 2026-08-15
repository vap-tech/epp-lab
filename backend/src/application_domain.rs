use sqlx::PgPool;
use thiserror::Error;

use crate::domain::{domain::DomainName, zone::Zone};

/// Async application boundary for loading configured Zones. The matching
/// algorithm remains synchronous and lives in the domain Zone module.
pub(crate) trait DomainZoneLookup {
    async fn configured_zones(&self) -> Result<Vec<Zone>, String>;
}

pub(crate) struct PostgresDomainZoneLookup<'a> {
    pub(crate) db: &'a PgPool,
}

impl DomainZoneLookup for PostgresDomainZoneLookup<'_> {
    async fn configured_zones(&self) -> Result<Vec<Zone>, String> {
        crate::storage::zone::list(self.db)
            .await
            .map_err(|error| error.to_string())?
            .into_iter()
            .map(crate::storage::zone::to_domain)
            .collect()
    }
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub(crate) enum DomainZoneResolutionError {
    #[error("failed to load configured zones: {0}")]
    Lookup(String),
    #[error("domain does not belong to a configured zone")]
    UnknownZone,
    #[error("domain belongs to an inactive zone")]
    InactiveZone,
}

pub(crate) async fn resolve_domain_zone<L: DomainZoneLookup>(
    lookup: &L,
    domain_name: &DomainName,
) -> Result<Zone, DomainZoneResolutionError> {
    let zones = lookup
        .configured_zones()
        .await
        .map_err(DomainZoneResolutionError::Lookup)?;
    let zone = crate::domain::zone::resolve_configured_zone(domain_name.as_str(), &zones)
        .ok_or(DomainZoneResolutionError::UnknownZone)?;
    if zone.status != crate::domain::zone::ZoneStatus::Active {
        return Err(DomainZoneResolutionError::InactiveZone);
    }
    Ok(zone.clone())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::zone::{ContactUsagePolicy, ZoneId, ZoneName, ZoneStatus};
    use uuid::Uuid;

    struct FixtureLookup {
        zones: Vec<Zone>,
    }

    impl DomainZoneLookup for FixtureLookup {
        async fn configured_zones(&self) -> Result<Vec<Zone>, String> {
            Ok(self.zones.clone())
        }
    }

    fn zone(name: &str, status: ZoneStatus) -> Zone {
        Zone {
            id: ZoneId::new(Uuid::new_v4()),
            name: ZoneName::parse(name).unwrap(),
            status,
            contact_policy: ContactUsagePolicy::default(),
        }
    }

    #[tokio::test]
    async fn resolves_longest_active_zone() {
        let lookup = FixtureLookup {
            zones: vec![
                zone("ru", ZoneStatus::Active),
                zone("net.ru", ZoneStatus::Active),
            ],
        };
        let resolved = resolve_domain_zone(&lookup, &DomainName::parse("example.net.ru").unwrap())
            .await
            .unwrap();
        assert_eq!(resolved.name.ascii(), "net.ru");
    }

    #[tokio::test]
    async fn reports_inactive_longest_configured_zone() {
        let lookup = FixtureLookup {
            zones: vec![
                zone("ru", ZoneStatus::Active),
                zone("net.ru", ZoneStatus::Disabled),
            ],
        };
        assert_eq!(
            resolve_domain_zone(&lookup, &DomainName::parse("example.net.ru").unwrap()).await,
            Err(DomainZoneResolutionError::InactiveZone)
        );
    }

    #[tokio::test]
    async fn reports_unknown_zone() {
        let lookup = FixtureLookup {
            zones: vec![zone("com", ZoneStatus::Active)],
        };
        assert_eq!(
            resolve_domain_zone(&lookup, &DomainName::parse("example.net").unwrap()).await,
            Err(DomainZoneResolutionError::UnknownZone)
        );
    }

    #[tokio::test]
    async fn preserves_lookup_errors() {
        struct FailingLookup;
        impl DomainZoneLookup for FailingLookup {
            async fn configured_zones(&self) -> Result<Vec<Zone>, String> {
                Err("database unavailable".into())
            }
        }
        assert_eq!(
            resolve_domain_zone(&FailingLookup, &DomainName::parse("example.com").unwrap()).await,
            Err(DomainZoneResolutionError::Lookup(
                "database unavailable".into()
            ))
        );
    }
}
