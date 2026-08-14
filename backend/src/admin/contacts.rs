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
    pub email: String,
    pub statuses: Vec<String>,
    pub linked: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
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
) -> Result<Json<ContactSummary>, StatusCode> {
    crate::storage::contact::find_summary(&state.db, id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .map(ContactSummary::from)
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
            email: row.email,
            statuses: row.statuses,
            linked: false,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}
