use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    admin::auth::{AdminSession, CsrfProtected},
    app::AppState,
    registry::certificate,
    storage::certificate as storage,
};

#[derive(Deserialize)]
pub(crate) struct CreateCertificateRequest {
    pub pem: String,
}

#[derive(Serialize)]
pub(crate) struct CertificateResponse {
    pub id: Uuid,
    pub fingerprint_sha256: String,
    pub subject: String,
    pub serial_number: Option<String>,
    pub not_before: chrono::DateTime<chrono::Utc>,
    pub not_after: chrono::DateTime<chrono::Utc>,
    pub status: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl From<storage::CertificateRow> for CertificateResponse {
    fn from(row: storage::CertificateRow) -> Self {
        Self {
            id: row.id,
            fingerprint_sha256: row.fingerprint_sha256,
            subject: row.subject,
            serial_number: row.serial_number,
            not_before: row.not_before,
            not_after: row.not_after,
            status: row.status,
            created_at: row.created_at,
        }
    }
}

pub(crate) async fn list(
    _session: AdminSession,
    Path(registrar_id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<CertificateResponse>>, StatusCode> {
    let exists = crate::storage::registrar::find(&state.db, registrar_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .is_some();
    if !exists {
        return Err(StatusCode::NOT_FOUND);
    }
    storage::list(&state.db, registrar_id)
        .await
        .map(|rows| Json(rows.into_iter().map(Into::into).collect()))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

pub(crate) async fn create(
    _session: AdminSession,
    _csrf: CsrfProtected,
    Path(registrar_id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    Json(request): Json<CreateCertificateRequest>,
) -> Result<(StatusCode, Json<CertificateResponse>), StatusCode> {
    let exists = crate::storage::registrar::find(&state.db, registrar_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .is_some();
    if !exists {
        return Err(StatusCode::NOT_FOUND);
    }
    let metadata = certificate::parse_pem(&request.pem).map_err(|_| StatusCode::BAD_REQUEST)?;
    storage::create(
        &state.db,
        registrar_id,
        &metadata.fingerprint_sha256,
        &metadata.subject,
        &metadata.serial_number,
        metadata.not_before,
        metadata.not_after,
    )
    .await
    .map(|row| (StatusCode::CREATED, Json(row.into())))
    .map_err(|error| {
        if matches!(error, sqlx::Error::Database(_)) {
            StatusCode::CONFLICT
        } else {
            StatusCode::INTERNAL_SERVER_ERROR
        }
    })
}
