use chrono::{DateTime, Months, Utc};
use std::collections::{BTreeSet, HashSet};
use thiserror::Error;
use uuid::Uuid;

use super::contact::ContactId;
use super::zone::{ContactRequirement, ContactUsagePolicy, Zone, ZoneId};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DomainId(Uuid);

impl DomainId {
    pub fn new(value: Uuid) -> Self {
        Self(value)
    }

    pub fn into_uuid(self) -> Uuid {
        self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DomainRoid(String);

impl DomainRoid {
    pub fn parse(value: &str) -> Result<Self, DomainIdentityError> {
        if value.is_empty()
            || value.len() > 80
            || value
                .bytes()
                .any(|byte| !(byte.is_ascii_alphanumeric() || byte == b'-'))
        {
            return Err(DomainIdentityError::InvalidRoid);
        }
        Ok(Self(value.to_owned()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum DomainIdentityError {
    #[error("domain ROID is invalid")]
    InvalidRoid,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DomainName(String);

impl DomainName {
    pub fn parse(value: &str) -> Result<Self, DomainNameError> {
        if value.is_empty() {
            return Err(DomainNameError::Empty);
        }
        if value.starts_with('.') || value.ends_with('.') {
            return Err(DomainNameError::BoundaryDot);
        }
        if !value.is_ascii() {
            return Err(DomainNameError::NonAscii);
        }
        if value.len() > 253 {
            return Err(DomainNameError::TooLong);
        }
        if value != value.to_ascii_lowercase() {
            return Err(DomainNameError::NotCanonical);
        }

        for label in value.split('.') {
            if label.is_empty() {
                return Err(DomainNameError::EmptyLabel);
            }
            if label.len() > 63
                || label.starts_with('-')
                || label.ends_with('-')
                || !label
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
            {
                return Err(DomainNameError::InvalidLabel);
            }
        }

        Ok(Self(value.to_owned()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn labels(&self) -> impl DoubleEndedIterator<Item = &str> {
        self.0.split('.')
    }
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum DomainNameError {
    #[error("domain name is empty")]
    Empty,
    #[error("domain name must not have leading or trailing dots")]
    BoundaryDot,
    #[error("domain name contains an empty label")]
    EmptyLabel,
    #[error("domain name must be canonical lowercase ASCII")]
    NotCanonical,
    #[error("domain name must use ASCII/A-label representation")]
    NonAscii,
    #[error("domain name is too long")]
    TooLong,
    #[error("domain name contains an invalid label")]
    InvalidLabel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DomainContactRole {
    Registrant,
    Admin,
    Tech,
    Billing,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DomainContacts {
    pub registrant: Option<ContactId>,
    pub admin: Vec<ContactId>,
    pub tech: Vec<ContactId>,
    pub billing: Vec<ContactId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DomainNameServer {
    pub hostname: DomainName,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Domain {
    pub id: DomainId,
    pub name: DomainName,
    pub roid: DomainRoid,
    pub zone_id: ZoneId,
    pub sponsoring_registrar_id: Uuid,
    pub registrant: Option<ContactId>,
    pub contacts: DomainContacts,
    pub nameservers: Vec<DomainNameServer>,
    pub auth_info: String,
    pub client_statuses: BTreeSet<DomainClientStatus>,
    pub server_statuses: BTreeSet<DomainServerStatus>,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_by: Option<Uuid>,
    pub updated_at: Option<DateTime<Utc>>,
    pub expires_at: DateTime<Utc>,
    pub transferred_at: Option<DateTime<Utc>>,
}

impl DomainNameServer {
    pub fn new(hostname: DomainName) -> Self {
        Self { hostname }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegistrationPeriodUnit {
    Years,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RegistrationPeriod {
    pub value: u8,
    pub unit: RegistrationPeriodUnit,
}

impl RegistrationPeriod {
    pub const DEFAULT: Self = Self {
        value: 1,
        unit: RegistrationPeriodUnit::Years,
    };

    pub fn years(value: u8) -> Result<Self, RegistrationPeriodError> {
        if !(1..=10).contains(&value) {
            return Err(RegistrationPeriodError::OutOfRange);
        }
        Ok(Self {
            value,
            unit: RegistrationPeriodUnit::Years,
        })
    }

    pub fn expires_at(
        self,
        created_at: DateTime<Utc>,
    ) -> Result<DateTime<Utc>, RegistrationPeriodError> {
        created_at
            .checked_add_months(Months::new(u32::from(self.value) * 12))
            .ok_or(RegistrationPeriodError::Overflow)
    }
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum RegistrationPeriodError {
    #[error("registration period must be between 1 and 10 years")]
    OutOfRange,
    #[error("registration period exceeds the supported date range")]
    Overflow,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub enum DomainClientStatus {
    ClientDeleteProhibited,
    ClientHold,
    ClientRenewProhibited,
    ClientTransferProhibited,
    ClientUpdateProhibited,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub enum DomainServerStatus {
    ServerDeleteProhibited,
    ServerHold,
    ServerRenewProhibited,
    ServerTransferProhibited,
    ServerUpdateProhibited,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub enum DomainStatus {
    Ok,
    Inactive,
    ClientDeleteProhibited,
    ClientHold,
    ClientRenewProhibited,
    ClientTransferProhibited,
    ClientUpdateProhibited,
    ServerDeleteProhibited,
    ServerHold,
    ServerRenewProhibited,
    ServerTransferProhibited,
    ServerUpdateProhibited,
}

pub fn effective_statuses(
    client_statuses: impl IntoIterator<Item = DomainClientStatus>,
    server_statuses: impl IntoIterator<Item = DomainServerStatus>,
    has_nameservers: bool,
) -> BTreeSet<DomainStatus> {
    let mut statuses = BTreeSet::new();
    for status in client_statuses {
        statuses.insert(match status {
            DomainClientStatus::ClientDeleteProhibited => DomainStatus::ClientDeleteProhibited,
            DomainClientStatus::ClientHold => DomainStatus::ClientHold,
            DomainClientStatus::ClientRenewProhibited => DomainStatus::ClientRenewProhibited,
            DomainClientStatus::ClientTransferProhibited => DomainStatus::ClientTransferProhibited,
            DomainClientStatus::ClientUpdateProhibited => DomainStatus::ClientUpdateProhibited,
        });
    }
    for status in server_statuses {
        statuses.insert(match status {
            DomainServerStatus::ServerDeleteProhibited => DomainStatus::ServerDeleteProhibited,
            DomainServerStatus::ServerHold => DomainStatus::ServerHold,
            DomainServerStatus::ServerRenewProhibited => DomainStatus::ServerRenewProhibited,
            DomainServerStatus::ServerTransferProhibited => DomainStatus::ServerTransferProhibited,
            DomainServerStatus::ServerUpdateProhibited => DomainStatus::ServerUpdateProhibited,
        });
    }
    if statuses.is_empty() {
        statuses.insert(if has_nameservers {
            DomainStatus::Ok
        } else {
            DomainStatus::Inactive
        });
    }
    statuses
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EffectiveDomainContacts {
    pub registrant: Option<ContactId>,
    pub admin: Vec<ContactId>,
    pub tech: Vec<ContactId>,
    pub billing: Vec<ContactId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContactUsageViolation {
    pub role: DomainContactRole,
}

pub fn effective_contacts(
    contacts: &DomainContacts,
    policy: ContactUsagePolicy,
) -> Result<EffectiveDomainContacts, ContactUsageViolation> {
    let registrant = project_single(
        contacts.registrant,
        policy.registrant,
        DomainContactRole::Registrant,
    )?;
    let admin = project_many(&contacts.admin, policy.admin, DomainContactRole::Admin)?;
    let tech = project_many(&contacts.tech, policy.tech, DomainContactRole::Tech)?;
    let billing = project_many(
        &contacts.billing,
        policy.billing,
        DomainContactRole::Billing,
    )?;
    Ok(EffectiveDomainContacts {
        registrant,
        admin,
        tech,
        billing,
    })
}

pub fn validate_contact_usage_for_create(
    contacts: &DomainContacts,
    policy: ContactUsagePolicy,
) -> Result<(), ContactUsageViolation> {
    let values = [
        (
            contacts.registrant.is_some(),
            policy.registrant,
            DomainContactRole::Registrant,
        ),
        (
            !contacts.admin.is_empty(),
            policy.admin,
            DomainContactRole::Admin,
        ),
        (
            !contacts.tech.is_empty(),
            policy.tech,
            DomainContactRole::Tech,
        ),
        (
            !contacts.billing.is_empty(),
            policy.billing,
            DomainContactRole::Billing,
        ),
    ];
    for (present, requirement, role) in values {
        if (requirement == ContactRequirement::Forbidden && present)
            || (requirement == ContactRequirement::Required && !present)
        {
            return Err(ContactUsageViolation { role });
        }
    }
    Ok(())
}

fn project_single(
    value: Option<ContactId>,
    requirement: ContactRequirement,
    role: DomainContactRole,
) -> Result<Option<ContactId>, ContactUsageViolation> {
    if requirement == ContactRequirement::Required && value.is_none() {
        return Err(ContactUsageViolation { role });
    }
    Ok(match requirement {
        ContactRequirement::Forbidden => None,
        ContactRequirement::Optional | ContactRequirement::Required => value,
    })
}

fn project_many(
    values: &[ContactId],
    requirement: ContactRequirement,
    role: DomainContactRole,
) -> Result<Vec<ContactId>, ContactUsageViolation> {
    if requirement == ContactRequirement::Required && values.is_empty() {
        return Err(ContactUsageViolation { role });
    }
    Ok(match requirement {
        ContactRequirement::Forbidden => Vec::new(),
        ContactRequirement::Optional | ContactRequirement::Required => values.to_vec(),
    })
}

pub fn has_same_zone_nameserver(
    nameservers: &[DomainNameServer],
    domain_zone_id: ZoneId,
    configured_zones: &[Zone],
) -> bool {
    nameservers.iter().any(|nameserver| {
        super::zone::resolve_zone(nameserver.hostname.as_str(), configured_zones)
            .is_some_and(|zone| zone.id == domain_zone_id)
    })
}

pub fn has_duplicate_contacts(contacts: &DomainContacts) -> bool {
    let mut seen = HashSet::new();
    contacts
        .registrant
        .into_iter()
        .chain(contacts.admin.iter().copied())
        .chain(contacts.tech.iter().copied())
        .chain(contacts.billing.iter().copied())
        .any(|contact_id| !seen.insert(contact_id))
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Datelike, TimeZone};

    fn contact() -> ContactId {
        ContactId::new(Uuid::new_v4())
    }

    fn zone(name: &str, status: super::super::zone::ZoneStatus) -> Zone {
        Zone {
            id: ZoneId::new(Uuid::new_v4()),
            name: super::super::zone::ZoneName::parse(name).unwrap(),
            status,
            contact_policy: ContactUsagePolicy::default(),
        }
    }

    #[test]
    fn domain_name_requires_canonical_ascii_labels() {
        assert_eq!(
            DomainName::parse("Example.com"),
            Err(DomainNameError::NotCanonical)
        );
        assert_eq!(
            DomainName::parse("пример.рф"),
            Err(DomainNameError::NonAscii)
        );
        assert_eq!(
            DomainName::parse("example..com"),
            Err(DomainNameError::EmptyLabel)
        );
        assert_eq!(
            DomainName::parse("-example.com"),
            Err(DomainNameError::InvalidLabel)
        );
        assert_eq!(
            DomainName::parse("example.com"),
            Ok(DomainName("example.com".into()))
        );
        assert_eq!(
            DomainName::parse("xn--e1afmkfd.xn--p1ai").unwrap().as_str(),
            "xn--e1afmkfd.xn--p1ai"
        );
    }

    #[test]
    fn registration_period_is_one_to_ten_years() {
        assert_eq!(
            RegistrationPeriod::years(0),
            Err(RegistrationPeriodError::OutOfRange)
        );
        assert_eq!(
            RegistrationPeriod::years(11),
            Err(RegistrationPeriodError::OutOfRange)
        );
        assert_eq!(
            RegistrationPeriod::DEFAULT,
            RegistrationPeriod::years(1).unwrap()
        );
    }

    #[test]
    fn expiration_is_calendar_aware() {
        let created = Utc.with_ymd_and_hms(2024, 2, 29, 12, 0, 0).unwrap();
        let expires = RegistrationPeriod::years(1)
            .unwrap()
            .expires_at(created)
            .unwrap();
        assert_eq!(
            (expires.year(), expires.month(), expires.day()),
            (2025, 2, 28)
        );
    }

    #[test]
    fn status_is_ok_or_inactive_only_without_explicit_statuses() {
        assert_eq!(
            effective_statuses([], [], false),
            BTreeSet::from([DomainStatus::Inactive])
        );
        assert_eq!(
            effective_statuses([], [], true),
            BTreeSet::from([DomainStatus::Ok])
        );
        assert_eq!(
            effective_statuses([DomainClientStatus::ClientHold], [], true),
            BTreeSet::from([DomainStatus::ClientHold])
        );
    }

    #[test]
    fn contact_projection_hides_forbidden_and_requires_required() {
        let id = contact();
        let contacts = DomainContacts {
            registrant: Some(id),
            admin: vec![contact()],
            ..Default::default()
        };
        let policy = ContactUsagePolicy {
            registrant: ContactRequirement::Forbidden,
            admin: ContactRequirement::Optional,
            tech: ContactRequirement::Forbidden,
            billing: ContactRequirement::Forbidden,
        };
        let effective = effective_contacts(&contacts, policy).unwrap();
        assert_eq!(effective.registrant, None);
        assert_eq!(effective.admin, contacts.admin);

        let required = ContactUsagePolicy {
            registrant: ContactRequirement::Required,
            ..policy
        };
        assert_eq!(
            effective_contacts(&DomainContacts::default(), required),
            Err(ContactUsageViolation {
                role: DomainContactRole::Registrant
            })
        );
    }

    #[test]
    fn same_zone_nameserver_uses_resolved_zone_id() {
        let ru = zone("ru", super::super::zone::ZoneStatus::Active);
        let net_ru = zone("net.ru", super::super::zone::ZoneStatus::Active);
        let zones = [ru.clone(), net_ru.clone()];
        let nameservers = vec![DomainNameServer::new(
            DomainName::parse("ns1.foo.net.ru").unwrap(),
        )];
        assert!(has_same_zone_nameserver(&nameservers, net_ru.id, &zones));
        assert!(!has_same_zone_nameserver(&nameservers, ru.id, &zones));
        let external = vec![DomainNameServer::new(
            DomainName::parse("ns1.example.com").unwrap(),
        )];
        assert!(!has_same_zone_nameserver(&external, net_ru.id, &zones));
    }

    #[test]
    fn duplicate_contact_references_are_detectable() {
        let id = contact();
        assert!(has_duplicate_contacts(&DomainContacts {
            registrant: Some(id),
            admin: vec![id],
            ..Default::default()
        }));
    }
}
