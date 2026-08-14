use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum SessionState {
    Unauthenticated,
    #[allow(dead_code)]
    Authenticated {
        registrar_id: Uuid,
        object_uris: Vec<String>,
    },
}

impl SessionState {
    pub(crate) fn allows_login(&self) -> bool {
        matches!(self, Self::Unauthenticated)
    }
    pub(crate) fn allows_logout(&self) -> bool {
        matches!(self, Self::Authenticated { .. })
    }
    pub(crate) fn has_object_uri(&self, uri: &str) -> bool {
        matches!(self, Self::Authenticated { object_uris, .. } if object_uris.iter().any(|item| item == uri))
    }
}

#[cfg(test)]
mod tests {
    use super::SessionState;

    #[test]
    fn command_permissions_follow_state() {
        let state = SessionState::Unauthenticated;
        assert!(state.allows_login());
        assert!(!state.allows_logout());
    }

    #[test]
    fn authenticated_state_rejects_second_login_and_allows_logout() {
        let state = SessionState::Authenticated {
            registrar_id: uuid::Uuid::new_v4(),
            object_uris: vec!["urn:ietf:params:xml:ns:contact-1.0".to_owned()],
        };
        assert!(!state.allows_login());
        assert!(state.allows_logout());
        assert!(state.has_object_uri("urn:ietf:params:xml:ns:contact-1.0"));
    }
}
