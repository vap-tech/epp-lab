use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum SessionState {
    Unauthenticated,
    #[allow(dead_code)]
    Authenticated {
        registrar_id: Uuid,
    },
}

impl SessionState {
    pub(crate) fn allows_login(&self) -> bool {
        matches!(self, Self::Unauthenticated)
    }
    pub(crate) fn allows_logout(&self) -> bool {
        matches!(self, Self::Authenticated { .. })
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
        };
        assert!(!state.allows_login());
        assert!(state.allows_logout());
    }
}
