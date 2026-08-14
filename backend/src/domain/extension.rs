use std::collections::BTreeMap;
use thiserror::Error;

use super::zone::{Zone, ZoneId, ZoneStatus};

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ExtensionKey(String);

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum ExtensionKeyError {
    #[error("extension key must not be empty")]
    Empty,
    #[error("extension key must contain only lowercase ASCII letters, digits, dots or hyphens")]
    Invalid,
}

impl ExtensionKey {
    pub fn parse(value: &str) -> Result<Self, ExtensionKeyError> {
        if value.is_empty() {
            return Err(ExtensionKeyError::Empty);
        }
        if value.bytes().any(|byte| {
            !(byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'.' || byte == b'-')
        }) {
            return Err(ExtensionKeyError::Invalid);
        }
        Ok(Self(value.to_owned()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

pub trait ExtensionDefinition: Send + Sync {
    fn key(&self) -> ExtensionKey;
    fn display_name(&self) -> &'static str;
    fn namespace_uri(&self) -> &'static str;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ZoneExtensionAssignment {
    pub zone_id: ZoneId,
    pub extension_key: ExtensionKey,
    pub enabled: bool,
}

pub struct ExtensionRegistry {
    definitions: BTreeMap<ExtensionKey, Box<dyn ExtensionDefinition>>,
}

impl ExtensionRegistry {
    pub fn empty() -> Self {
        Self {
            definitions: BTreeMap::new(),
        }
    }

    pub fn from_definitions(
        definitions: impl IntoIterator<Item = Box<dyn ExtensionDefinition>>,
    ) -> Result<Self, ExtensionRegistryError> {
        let mut registry = Self::empty();
        for definition in definitions {
            let key = definition.key();
            if registry.definitions.insert(key, definition).is_some() {
                return Err(ExtensionRegistryError::DuplicateKey);
            }
        }
        Ok(registry)
    }

    pub fn get(&self, key: &ExtensionKey) -> Option<&dyn ExtensionDefinition> {
        self.definitions.get(key).map(Box::as_ref)
    }

    pub fn find_by_namespace(&self, namespace_uri: &str) -> Option<&dyn ExtensionDefinition> {
        self.definitions
            .values()
            .find(|definition| definition.namespace_uri() == namespace_uri)
            .map(Box::as_ref)
    }

    pub fn list(&self) -> impl Iterator<Item = &dyn ExtensionDefinition> {
        self.definitions.values().map(Box::as_ref)
    }

    pub fn advertised_namespaces<'a>(
        &'a self,
        zones: impl IntoIterator<Item = &'a Zone>,
        assignments: impl IntoIterator<Item = &'a ZoneExtensionAssignment>,
    ) -> Vec<&'static str> {
        let active_zone_ids: Vec<_> = zones
            .into_iter()
            .filter(|zone| zone.status == ZoneStatus::Active)
            .map(|zone| zone.id)
            .collect();
        let mut namespaces = assignments
            .into_iter()
            .filter(|assignment| assignment.enabled)
            .filter(|assignment| active_zone_ids.contains(&assignment.zone_id))
            .filter_map(|assignment| self.get(&assignment.extension_key))
            .map(ExtensionDefinition::namespace_uri)
            .collect::<Vec<_>>();
        namespaces.sort_unstable();
        namespaces.dedup();
        namespaces
    }
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum ExtensionRegistryError {
    #[error("extension key is registered more than once")]
    DuplicateKey,
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestExtension;

    impl ExtensionDefinition for TestExtension {
        fn key(&self) -> ExtensionKey {
            ExtensionKey::parse("test-extension-1").unwrap()
        }

        fn display_name(&self) -> &'static str {
            "Test Extension"
        }

        fn namespace_uri(&self) -> &'static str {
            "urn:epp:params:xml:ns:test-1.0"
        }
    }

    #[test]
    fn empty_production_registry_is_honest() {
        let registry = ExtensionRegistry::empty();
        assert_eq!(registry.list().count(), 0);
    }

    #[test]
    fn looks_up_registered_definition_by_key_and_namespace() {
        let registry = ExtensionRegistry::from_definitions(vec![
            Box::new(TestExtension) as Box<dyn ExtensionDefinition>
        ])
        .unwrap();
        let key = ExtensionKey::parse("test-extension-1").unwrap();
        assert_eq!(registry.get(&key).unwrap().display_name(), "Test Extension");
        assert!(
            registry
                .find_by_namespace("urn:epp:params:xml:ns:test-1.0")
                .is_some()
        );
    }

    #[test]
    fn rejects_duplicate_keys() {
        let result = ExtensionRegistry::from_definitions(vec![
            Box::new(TestExtension) as Box<dyn ExtensionDefinition>,
            Box::new(TestExtension) as Box<dyn ExtensionDefinition>,
        ]);
        assert!(matches!(result, Err(ExtensionRegistryError::DuplicateKey)));
    }

    #[test]
    fn validates_extension_keys() {
        assert!(ExtensionKey::parse("fee-0.6").is_ok());
        assert_eq!(ExtensionKey::parse("Fee"), Err(ExtensionKeyError::Invalid));
        assert_eq!(ExtensionKey::parse(""), Err(ExtensionKeyError::Empty));
    }

    #[test]
    fn advertises_only_registered_enabled_extensions_for_active_zones() {
        let registry = ExtensionRegistry::from_definitions(vec![
            Box::new(TestExtension) as Box<dyn ExtensionDefinition>
        ])
        .unwrap();
        let active_zone = Zone {
            id: ZoneId::new(uuid::Uuid::new_v4()),
            name: crate::domain::zone::ZoneName::parse("com").unwrap(),
            status: ZoneStatus::Active,
            contact_policy: Default::default(),
        };
        let disabled_zone = Zone {
            status: ZoneStatus::Disabled,
            ..active_zone.clone()
        };
        let assignment = ZoneExtensionAssignment {
            zone_id: active_zone.id,
            extension_key: ExtensionKey::parse("test-extension-1").unwrap(),
            enabled: true,
        };
        assert_eq!(
            registry
                .advertised_namespaces([&active_zone], [&assignment])
                .len(),
            1
        );
        assert!(
            registry
                .advertised_namespaces([&disabled_zone], [&assignment])
                .is_empty()
        );
    }
}
