use axum::{Json, extract::State, http::StatusCode};
use serde::Serialize;
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    admin::auth::AdminSession,
    app::AppState,
    storage::zone::{self, ZoneRow},
};

#[derive(Debug, Serialize)]
pub(crate) struct ZoneResponse {
    pub id: Uuid,
    pub ascii_name: String,
    pub unicode_name: String,
    pub status: String,
    pub contact_policy: ContactPolicyResponse,
    pub contactless: bool,
    pub enabled_extensions_count: i64,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize)]
pub(crate) struct ContactPolicyResponse {
    pub registrant: String,
    pub admin: String,
    pub tech: String,
    pub billing: String,
}

impl TryFrom<ZoneRow> for ZoneResponse {
    type Error = String;

    fn try_from(row: ZoneRow) -> Result<Self, Self::Error> {
        let policy = crate::domain::zone::ContactUsagePolicy {
            registrant: parse_requirement(&row.registrant_requirement)?,
            admin: parse_requirement(&row.admin_requirement)?,
            tech: parse_requirement(&row.tech_requirement)?,
            billing: parse_requirement(&row.billing_requirement)?,
        };
        Ok(Self {
            id: row.id,
            ascii_name: row.ascii_name,
            unicode_name: row.unicode_name.unwrap_or_default(),
            status: row.status,
            contactless: policy.is_contactless(),
            contact_policy: ContactPolicyResponse {
                registrant: row.registrant_requirement,
                admin: row.admin_requirement,
                tech: row.tech_requirement,
                billing: row.billing_requirement,
            },
            enabled_extensions_count: row.enabled_extensions_count,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}

pub(crate) async fn list(
    _session: AdminSession,
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<ZoneResponse>>, StatusCode> {
    let rows = zone::list(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    rows.into_iter()
        .map(ZoneResponse::try_from)
        .collect::<Result<Vec<_>, _>>()
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

fn parse_requirement(value: &str) -> Result<crate::domain::zone::ContactRequirement, String> {
    match value {
        "forbidden" => Ok(crate::domain::zone::ContactRequirement::Forbidden),
        "optional" => Ok(crate::domain::zone::ContactRequirement::Optional),
        "required" => Ok(crate::domain::zone::ContactRequirement::Required),
        _ => Err(format!("unknown contact requirement: {value}")),
    }
}
