use argon2::password_hash::{SaltString, rand_core::OsRng};
use argon2::{Argon2, PasswordHasher};
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::{
    admin::auth::{AdminSession, CsrfProtected},
    app::AppState,
    storage::registrar,
};

#[derive(Deserialize)]
pub(crate) struct CreateRegistrarRequest {
    pub handle: String,
    pub name: String,
    pub client_id: String,
    pub password: String,
}

#[derive(Serialize)]
pub(crate) struct RegistrarResponse {
    pub id: uuid::Uuid,
    pub handle: String,
    pub name: String,
    pub client_id: String,
    pub status: String,
}

impl From<registrar::RegistrarRow> for RegistrarResponse {
    fn from(row: registrar::RegistrarRow) -> Self {
        Self {
            id: row.id,
            handle: row.handle,
            name: row.name,
            client_id: row.client_id,
            status: row.status,
        }
    }
}

pub(crate) async fn list(
    _session: AdminSession,
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<RegistrarResponse>>, StatusCode> {
    registrar::list(&state.db)
        .await
        .map(|rows| Json(rows.into_iter().map(Into::into).collect()))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

pub(crate) async fn get(
    _session: AdminSession,
    Path(id): Path<uuid::Uuid>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<RegistrarResponse>, StatusCode> {
    registrar::find(&state.db, id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .map(|row| Json(row.into()))
        .ok_or(StatusCode::NOT_FOUND)
}

pub(crate) async fn create(
    _session: AdminSession,
    _csrf: CsrfProtected,
    State(state): State<Arc<AppState>>,
    Json(request): Json<CreateRegistrarRequest>,
) -> Result<(StatusCode, Json<RegistrarResponse>), StatusCode> {
    if request.handle.trim().is_empty()
        || request.client_id.trim().is_empty()
        || request.password.is_empty()
    {
        return Err(StatusCode::BAD_REQUEST);
    }
    let salt = SaltString::generate(&mut OsRng);
    let hash = Argon2::default()
        .hash_password(request.password.as_bytes(), &salt)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .to_string();
    registrar::create(
        &state.db,
        &request.handle,
        &request.name,
        &request.client_id,
        &hash,
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
