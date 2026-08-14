use chrono::{DateTime, Utc};
use std::collections::BTreeSet;
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ContactId(Uuid);
impl ContactId {
    pub fn new(value: Uuid) -> Self {
        Self(value)
    }
    pub fn into_uuid(self) -> Uuid {
        self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ContactRoid(String);
impl ContactRoid {
    pub fn parse(value: &str) -> Result<Self, ContactIdentityError> {
        if value.is_empty()
            || value.len() > 80
            || value
                .bytes()
                .any(|b| !(b.is_ascii_alphanumeric() || b == b'-'))
        {
            return Err(ContactIdentityError::InvalidRoid);
        }
        Ok(Self(value.to_owned()))
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum ContactIdentityError {
    #[error("contact ROID is invalid")]
    InvalidRoid,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PostalInfoSet {
    pub international: PostalInfo,
    pub localized: Option<PostalInfo>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PostalInfo {
    pub name: String,
    pub organization: Option<String>,
    pub address: PostalAddress,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PostalAddress {
    pub streets: Vec<String>,
    pub city: String,
    pub state_province: Option<String>,
    pub postal_code: Option<String>,
    pub country_code: CountryCode,
}
impl PostalAddress {
    pub fn validate(&self) -> Result<(), ContactFieldError> {
        if self.streets.is_empty()
            || self.streets.len() > 3
            || self.streets.iter().any(String::is_empty)
        {
            return Err(ContactFieldError::InvalidStreets);
        }
        required(&self.city, "city")?;
        optional(self.state_province.as_deref(), "state/province")?;
        optional(self.postal_code.as_deref(), "postal code")
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CountryCode(String);
impl CountryCode {
    pub fn parse(value: &str) -> Result<Self, ContactFieldError> {
        if value.len() != 2 || !value.bytes().all(|b| b.is_ascii_uppercase()) {
            return Err(ContactFieldError::InvalidCountryCode);
        }
        Ok(Self(value.to_owned()))
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PhoneNumber {
    pub number: String,
    pub extension: Option<String>,
}
impl PhoneNumber {
    pub fn validate(&self) -> Result<(), ContactFieldError> {
        required(&self.number, "phone number")?;
        optional(self.extension.as_deref(), "phone extension")
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmailAddress(String);
impl EmailAddress {
    pub fn parse(value: &str) -> Result<Self, ContactFieldError> {
        let (local, domain) = value
            .split_once('@')
            .ok_or(ContactFieldError::InvalidEmail)?;
        if local.is_empty() || domain.is_empty() || !domain.contains('.') || value.len() > 254 {
            return Err(ContactFieldError::InvalidEmail);
        }
        Ok(Self(value.to_owned()))
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub enum ContactStatus {
    ClientDeleteProhibited,
    ClientTransferProhibited,
    ClientUpdateProhibited,
    Linked,
    Ok,
    PendingCreate,
    PendingDelete,
    PendingTransfer,
    PendingUpdate,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisclosureFlag {
    Public,
    Private,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub enum DisclosureField {
    Name,
    Organization,
    Address,
    Voice,
    Fax,
    Email,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DisclosurePreference {
    pub flag: DisclosureFlag,
    pub fields: BTreeSet<DisclosureField>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Contact {
    pub id: ContactId,
    pub roid: ContactRoid,
    pub postal_info: PostalInfoSet,
    pub voice: PhoneNumber,
    pub fax: Option<PhoneNumber>,
    pub email: EmailAddress,
    pub auth_info: String,
    pub disclose: DisclosurePreference,
    pub client_statuses: BTreeSet<ContactStatus>,
    pub server_statuses: BTreeSet<ContactStatus>,
    pub sponsoring_registrar_id: Uuid,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_by: Uuid,
    pub updated_at: DateTime<Utc>,
    pub transferred_at: Option<DateTime<Utc>>,
}
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum ContactFieldError {
    #[error("{0} is required")]
    Required(&'static str),
    #[error("{0} is too long")]
    TooLong(&'static str),
    #[error("postal streets are invalid")]
    InvalidStreets,
    #[error("country code is invalid")]
    InvalidCountryCode,
    #[error("email address is invalid")]
    InvalidEmail,
}
fn required(value: &str, field: &'static str) -> Result<(), ContactFieldError> {
    if value.is_empty() {
        return Err(ContactFieldError::Required(field));
    }
    optional(Some(value), field)
}
fn optional(value: Option<&str>, field: &'static str) -> Result<(), ContactFieldError> {
    if value.is_some_and(|v| v.len() > 255) {
        return Err(ContactFieldError::TooLong(field));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn validates_contact_scalars() {
        assert!(ContactRoid::parse("SH8013").is_ok());
        assert!(ContactRoid::parse("bad value").is_err());
        assert!(CountryCode::parse("RU").is_ok());
        assert!(CountryCode::parse("ru").is_err());
        assert!(EmailAddress::parse("a@example.test").is_ok());
        assert!(EmailAddress::parse("invalid").is_err());
    }
    #[test]
    fn validates_postal_address() {
        let address = PostalAddress {
            streets: vec!["Main 1".into()],
            city: "Moscow".into(),
            state_province: None,
            postal_code: None,
            country_code: CountryCode::parse("RU").unwrap(),
        };
        assert!(address.validate().is_ok());
        assert!(
            PostalAddress {
                city: String::new(),
                ..address
            }
            .validate()
            .is_err()
        );
    }
}
