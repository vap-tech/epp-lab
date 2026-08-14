use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    admin::auth::{AdminSession, CsrfProtected},
    app::AppState,
    domain::extension::ExtensionKey,
    storage::zone,
};

#[derive(Debug, Serialize)]
pub(crate) struct ExtensionCatalogItem {
    pub key: String,
    pub display_name: String,
    pub namespace_uri: String,
}

pub(crate) async fn catalog(
    _session: AdminSession,
    State(state): State<Arc<AppState>>,
) -> Json<Vec<ExtensionCatalogItem>> {
    Json(
        state
            .extension_registry
            .list()
            .map(|extension| ExtensionCatalogItem {
                key: extension.key().as_str().to_owned(),
                display_name: extension.display_name().to_owned(),
                namespace_uri: extension.namespace_uri().to_owned(),
            })
            .collect(),
    )
}

#[derive(Debug, Serialize)]
pub(crate) struct ZoneExtensionResponse {
    pub zone_id: Uuid,
    pub extension_key: String,
    pub enabled: bool,
}

pub(crate) async fn list_zone_extensions(
    _session: AdminSession,
    Path(zone_id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<ZoneExtensionResponse>>, StatusCode> {
    let rows = zone::list_extensions(&state.db, zone_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(
        rows.into_iter()
            .map(|row| ZoneExtensionResponse {
                zone_id: row.zone_id,
                extension_key: row.extension_key,
                enabled: row.enabled,
            })
            .collect(),
    ))
}

#[derive(Debug, Deserialize)]
pub(crate) struct SetZoneExtensionRequest {
    pub enabled: bool,
}

pub(crate) async fn set_zone_extension(
    _session: AdminSession,
    _csrf: CsrfProtected,
    Path((zone_id, extension_key)): Path<(Uuid, String)>,
    State(state): State<Arc<AppState>>,
    Json(request): Json<SetZoneExtensionRequest>,
) -> Result<Json<ZoneExtensionResponse>, StatusCode> {
    let key = ExtensionKey::parse(&extension_key).map_err(|_| StatusCode::BAD_REQUEST)?;
    if state.extension_registry.get(&key).is_none() {
        return Err(StatusCode::UNPROCESSABLE_ENTITY);
    }
    if zone::find(&state.db, zone_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .is_none()
    {
        return Err(StatusCode::NOT_FOUND);
    }
    zone::set_extension(
        &state.db,
        zone_id,
        key.as_str(),
        request.enabled,
        chrono::Utc::now(),
    )
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(ZoneExtensionResponse {
        zone_id,
        extension_key: key.as_str().to_owned(),
        enabled: request.enabled,
    }))
}
