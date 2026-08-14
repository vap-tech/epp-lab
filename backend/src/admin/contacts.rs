use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use serde::Serialize;
use uuid::Uuid;

use crate::{admin::auth::AdminSession, app::AppState};

#[derive(Debug, Serialize)]
pub(crate) struct ContactSummary {
    pub id: Uuid,
    pub contact_id: String,
    pub roid: String,
    pub registrar_id: Uuid,
    pub registrar_handle: Option<String>,
    pub email: String,
    pub statuses: Vec<String>,
    pub linked: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize)]
pub(crate) struct ContactDetail {
    #[serde(flatten)]
    pub summary: ContactSummary,
    pub name: String,
    pub organization: Option<String>,
    pub streets: Vec<String>,
    pub city: String,
    pub state_province: Option<String>,
    pub postal_code: Option<String>,
    pub country_code: String,
    pub voice: String,
    pub voice_extension: Option<String>,
    pub fax: Option<String>,
    pub fax_extension: Option<String>,
    pub disclose_flag: String,
    pub disclosure_fields: Vec<String>,
}

pub(crate) async fn list(
    _session: AdminSession,
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<ContactSummary>>, StatusCode> {
    let rows = crate::storage::contact::list_summaries(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(rows.into_iter().map(ContactSummary::from).collect()))
}

pub(crate) async fn get(
    _session: AdminSession,
    Path(id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<ContactDetail>, StatusCode> {
    crate::storage::contact::find_detail(&state.db, id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .map(ContactDetail::from)
        .map(Json)
        .ok_or(StatusCode::NOT_FOUND)
}

impl From<crate::storage::contact::ContactSummaryRow> for ContactSummary {
    fn from(row: crate::storage::contact::ContactSummaryRow) -> Self {
        Self {
            id: row.id,
            contact_id: row.roid.clone(),
            roid: row.roid,
            registrar_id: row.sponsoring_registrar_id,
            registrar_handle: row.registrar_handle,
            email: row.email,
            statuses: row.statuses,
            linked: false,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

impl From<crate::storage::contact::ContactDetailRow> for ContactDetail {
    fn from(row: crate::storage::contact::ContactDetailRow) -> Self {
        Self {
            summary: ContactSummary {
                id: row.id,
                contact_id: row.roid.clone(),
                roid: row.roid,
                registrar_id: row.sponsoring_registrar_id,
                registrar_handle: row.registrar_handle,
                email: row.email,
                statuses: row.statuses,
                linked: false,
                created_at: row.created_at,
                updated_at: row.updated_at,
            },
            name: row.name,
            organization: row.organization,
            streets: row.streets,
            city: row.city,
            state_province: row.state_province,
            postal_code: row.postal_code,
            country_code: row.country_code,
            voice: row.voice,
            voice_extension: row.voice_extension,
            fax: row.fax,
            fax_extension: row.fax_extension,
            disclose_flag: row.disclose_flag,
            disclosure_fields: row.disclosure_fields,
        }
    }
}
