use idna::domain_to_ascii;
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ZoneId(Uuid);

impl ZoneId {
    pub fn new(value: Uuid) -> Self {
        Self(value)
    }

    pub fn into_uuid(self) -> Uuid {
        self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ZoneName {
    ascii: String,
    unicode: String,
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum ZoneNameError {
    #[error("zone name must not be empty")]
    Empty,
    #[error("zone name must not have leading or trailing dots")]
    BoundaryDot,
    #[error("zone name must not contain empty labels")]
    EmptyLabel,
    #[error("zone name must be a canonical lowercase ASCII name")]
    NotCanonical,
    #[error("zone name is not a valid IDNA name")]
    InvalidIdna,
}

impl ZoneName {
    pub fn parse(value: &str) -> Result<Self, ZoneNameError> {
        if value.is_empty() {
            return Err(ZoneNameError::Empty);
        }
        if value.starts_with('.') || value.ends_with('.') {
            return Err(ZoneNameError::BoundaryDot);
        }
        if value.split('.').any(str::is_empty) {
            return Err(ZoneNameError::EmptyLabel);
        }
        if value.is_ascii() && value != value.to_ascii_lowercase() {
            return Err(ZoneNameError::NotCanonical);
        }
        let ascii = domain_to_ascii(value).map_err(|_| ZoneNameError::InvalidIdna)?;
        if ascii.is_empty() || ascii != ascii.to_ascii_lowercase() {
            return Err(ZoneNameError::NotCanonical);
        }
        let unicode = idna::domain_to_unicode(&ascii).0;
        Ok(Self { ascii, unicode })
    }

    pub fn ascii(&self) -> &str {
        &self.ascii
    }
    pub fn unicode(&self) -> &str {
        &self.unicode
    }
    pub fn labels(&self) -> impl DoubleEndedIterator<Item = &str> {
        self.ascii.split('.')
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ZoneStatus {
    Active,
    Disabled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContactRequirement {
    Forbidden,
    Optional,
    Required,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ContactUsagePolicy {
    pub registrant: ContactRequirement,
    pub admin: ContactRequirement,
    pub tech: ContactRequirement,
    pub billing: ContactRequirement,
}

impl Default for ContactUsagePolicy {
    fn default() -> Self {
        Self {
            registrant: ContactRequirement::Required,
            admin: ContactRequirement::Optional,
            tech: ContactRequirement::Optional,
            billing: ContactRequirement::Optional,
        }
    }
}

impl ContactUsagePolicy {
    pub fn is_contactless(self) -> bool {
        self.registrant == ContactRequirement::Forbidden
            && self.admin == ContactRequirement::Forbidden
            && self.tech == ContactRequirement::Forbidden
            && self.billing == ContactRequirement::Forbidden
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Zone {
    pub id: ZoneId,
    pub name: ZoneName,
    pub status: ZoneStatus,
    pub contact_policy: ContactUsagePolicy,
}

pub fn resolve_zone<'a>(
    domain_name: &str,
    zones: impl IntoIterator<Item = &'a Zone>,
) -> Option<&'a Zone> {
    resolve_configured_zone(domain_name, zones).filter(|zone| zone.status == ZoneStatus::Active)
}

/// Resolves the most specific configured zone without applying its lifecycle
/// status. Application code uses this to distinguish an unknown zone from a
/// configured but inactive zone.
pub fn resolve_configured_zone<'a>(
    domain_name: &str,
    zones: impl IntoIterator<Item = &'a Zone>,
) -> Option<&'a Zone> {
    let domain = domain_to_ascii(domain_name).ok()?.to_ascii_lowercase();
    zones
        .into_iter()
        .filter(|zone| {
            domain == zone.name.ascii() || domain.ends_with(&format!(".{}", zone.name.ascii()))
        })
        .max_by_key(|zone| zone.name.labels().count())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn zone(name: &str, status: ZoneStatus) -> Zone {
        Zone {
            id: ZoneId::new(Uuid::new_v4()),
            name: ZoneName::parse(name).unwrap(),
            status,
            contact_policy: ContactUsagePolicy::default(),
        }
    }

    #[test]
    fn canonicalizes_unicode_zone_name() {
        let name = ZoneName::parse("рф").unwrap();
        assert_eq!(name.ascii(), "xn--p1ai");
        assert_eq!(name.unicode(), "рф");
    }

    #[test]
    fn rejects_noncanonical_ascii_and_boundary_dots() {
        assert_eq!(ZoneName::parse("RU"), Err(ZoneNameError::NotCanonical));
        assert_eq!(ZoneName::parse(".ru"), Err(ZoneNameError::BoundaryDot));
        assert_eq!(ZoneName::parse("ru."), Err(ZoneNameError::BoundaryDot));
    }

    #[test]
    fn contactless_is_derived_from_all_forbidden_roles() {
        let policy = ContactUsagePolicy {
            registrant: ContactRequirement::Forbidden,
            admin: ContactRequirement::Forbidden,
            tech: ContactRequirement::Forbidden,
            billing: ContactRequirement::Forbidden,
        };
        assert!(policy.is_contactless());
        assert!(!ContactUsagePolicy::default().is_contactless());
    }

    #[test]
    fn resolves_longest_active_suffix() {
        let zones = [
            zone("ru", ZoneStatus::Active),
            zone("net.ru", ZoneStatus::Active),
        ];
        assert_eq!(
            resolve_zone("example.net.ru", &zones).unwrap().name.ascii(),
            "net.ru"
        );
    }

    #[test]
    fn does_not_resolve_disabled_zone_or_partial_label_match() {
        let zones = [
            zone("ru", ZoneStatus::Disabled),
            zone("com", ZoneStatus::Active),
        ];
        assert!(resolve_zone("example.ru", &zones).is_none());
        assert!(resolve_zone("notcom", &zones).is_none());
    }

    #[test]
    fn configured_resolution_keeps_inactive_longest_match() {
        let zones = [
            zone("ru", ZoneStatus::Active),
            zone("net.ru", ZoneStatus::Disabled),
        ];
        assert_eq!(
            resolve_configured_zone("example.net.ru", &zones)
                .unwrap()
                .name
                .ascii(),
            "net.ru"
        );
        assert!(resolve_zone("example.net.ru", &zones).is_none());
    }
}
