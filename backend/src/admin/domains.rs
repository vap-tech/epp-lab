use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use serde::Serialize;
use uuid::Uuid;

use crate::{
    admin::{auth::AdminSession, epp::Page},
    app::AppState,
};

#[derive(Debug, serde::Deserialize)]
pub(crate) struct DomainQuery {
    pub page: Option<i64>,
    pub page_size: Option<i64>,
    pub search: Option<String>,
    pub zone_id: Option<Uuid>,
}

#[derive(Debug, Serialize)]
pub(crate) struct DomainSummary {
    pub id: Uuid,
    pub domain_name: String,
    pub roid: String,
    pub zone: DomainZone,
    pub registrar: DomainRegistrar,
    pub statuses: Vec<String>,
    pub expires_at: chrono::DateTime<chrono::Utc>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Serialize)]
pub(crate) struct DomainZone {
    pub id: Uuid,
    pub name: String,
}

#[derive(Debug, Serialize)]
pub(crate) struct DomainRegistrar {
    pub id: Uuid,
    pub handle: String,
}

#[derive(Debug, Serialize)]
pub(crate) struct DomainDetail {
    #[serde(flatten)]
    pub summary: DomainSummary,
    pub nameservers: Vec<String>,
    pub contacts: Vec<DomainContact>,
}

#[derive(Debug, Serialize)]
pub(crate) struct DomainContact {
    pub role: String,
    pub contact_id: Uuid,
    pub effective: bool,
}

pub(crate) async fn list(
    _session: AdminSession,
    State(state): State<Arc<AppState>>,
    Query(query): Query<DomainQuery>,
) -> Result<Json<Page<DomainSummary>>, StatusCode> {
    let page = query.page.unwrap_or(1);
    let page_size = query.page_size.unwrap_or(50);
    if page < 1 || !(1..=100).contains(&page_size) {
        return Err(StatusCode::BAD_REQUEST);
    }
    let (rows, total) = crate::storage::domain::list_summaries(
        &state.db,
        crate::storage::domain::DomainListQuery {
            page,
            page_size,
            search: query.search,
            zone_id: query.zone_id,
        },
    )
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(Page {
        items: rows.into_iter().map(DomainSummary::from).collect(),
        page,
        page_size,
        total,
        total_pages: if total == 0 {
            0
        } else {
            (total + page_size - 1) / page_size
        },
    }))
}

pub(crate) async fn get(
    _session: AdminSession,
    Path(id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<DomainDetail>, StatusCode> {
    let Some(row) = crate::storage::domain::find_summary(&state.db, id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    else {
        return Err(StatusCode::NOT_FOUND);
    };
    let nameservers = crate::storage::domain::list_nameservers(&state.db, id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .into_iter()
        .map(|row| row.hostname)
        .collect();
    let policy = crate::storage::zone::find(&state.db, row.zone_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .and_then(|zone| crate::storage::zone::to_domain(zone).ok())
        .map(|zone| zone.contact_policy);
    let contacts = crate::storage::domain::list_contacts(&state.db, id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .into_iter()
        .map(|row| {
            let effective = policy
                .map(|policy| match row.role.as_str() {
                    "registrant" => {
                        policy.registrant != crate::domain::zone::ContactRequirement::Forbidden
                    }
                    "admin" => policy.admin != crate::domain::zone::ContactRequirement::Forbidden,
                    "tech" => policy.tech != crate::domain::zone::ContactRequirement::Forbidden,
                    "billing" => {
                        policy.billing != crate::domain::zone::ContactRequirement::Forbidden
                    }
                    _ => false,
                })
                .unwrap_or(false);
            DomainContact {
                role: row.role,
                contact_id: row.contact_id,
                effective,
            }
        })
        .collect();
    Ok(Json(DomainDetail {
        summary: row.into(),
        nameservers,
        contacts,
    }))
}

impl From<crate::storage::domain::DomainSummaryRow> for DomainSummary {
    fn from(row: crate::storage::domain::DomainSummaryRow) -> Self {
        let statuses = if row.statuses.is_empty() {
            vec![
                if row.has_nameservers {
                    "ok"
                } else {
                    "inactive"
                }
                .to_owned(),
            ]
        } else {
            row.statuses
        };
        Self {
            id: row.id,
            domain_name: row.name,
            roid: row.roid,
            zone: DomainZone {
                id: row.zone_id,
                name: row.zone_name,
            },
            registrar: DomainRegistrar {
                id: row.registrar_id,
                handle: row.registrar_handle,
            },
            statuses,
            expires_at: row.expires_at,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}
