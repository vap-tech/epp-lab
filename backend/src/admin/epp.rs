use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    admin::auth::AdminSession,
    app::AppState,
    storage::epp_queries::{self, SessionRow, TransactionRow},
};

#[derive(Deserialize)]
pub(crate) struct SessionQuery {
    pub page: Option<i64>,
    pub page_size: Option<i64>,
    pub registrar_id: Option<Uuid>,
    pub state: Option<String>,
    pub remote_addr: Option<String>,
    pub date_from: Option<DateTime<Utc>>,
    pub date_to: Option<DateTime<Utc>>,
}
#[derive(Deserialize)]
pub(crate) struct TransactionQuery {
    pub page: Option<i64>,
    pub page_size: Option<i64>,
    pub session_id: Option<Uuid>,
    pub registrar_id: Option<Uuid>,
    pub command: Option<String>,
    pub response_code: Option<i32>,
    pub delivery_status: Option<String>,
    pub trid: Option<String>,
}
#[derive(Serialize)]
pub(crate) struct Page<T> {
    pub items: Vec<T>,
    pub page: i64,
    pub page_size: i64,
    pub total: i64,
    pub total_pages: i64,
}
#[derive(Serialize)]
pub(crate) struct Registrar {
    pub id: Uuid,
    pub handle: String,
    pub name: String,
}
#[derive(Serialize)]
pub(crate) struct Certificate {
    pub id: Uuid,
    pub fingerprint_sha256: String,
}
#[derive(Serialize)]
pub(crate) struct SessionDto {
    pub id: Uuid,
    pub registrar: Option<Registrar>,
    pub certificate: Option<Certificate>,
    pub remote_addr: String,
    pub connected_at: DateTime<Utc>,
    pub authenticated_at: Option<DateTime<Utc>>,
    pub disconnected_at: Option<DateTime<Utc>>,
    pub disconnect_reason: Option<String>,
    pub transaction_count: i64,
}
#[derive(Serialize)]
pub(crate) struct TransactionDto {
    pub id: Uuid,
    pub session_id: Uuid,
    pub registrar: Option<Registrar>,
    pub command: String,
    pub object_name: Option<String>,
    pub cl_trid: Option<String>,
    pub sv_trid: String,
    pub request_xml: Option<String>,
    pub response_xml: Option<String>,
    pub response_code: Option<i32>,
    pub delivery_status: String,
    pub delivery_error: Option<String>,
    pub started_at: DateTime<Utc>,
    pub finished_at: Option<DateTime<Utc>>,
    pub duration_ms: Option<i64>,
}

fn paging(page: Option<i64>, size: Option<i64>) -> Result<(i64, i64), StatusCode> {
    let p = page.unwrap_or(1);
    let s = size.unwrap_or(50);
    if p < 1 || !(1..=100).contains(&s) {
        Err(StatusCode::BAD_REQUEST)
    } else {
        Ok((p, s))
    }
}
fn total_pages(total: i64, size: i64) -> i64 {
    if total == 0 {
        0
    } else {
        (total + size - 1) / size
    }
}
fn session_dto(r: SessionRow) -> SessionDto {
    SessionDto {
        id: r.id,
        registrar: r
            .registrar_id
            .zip(r.registrar_handle)
            .zip(r.registrar_name)
            .map(|((id, handle), name)| Registrar { id, handle, name }),
        certificate: r
            .certificate_id
            .zip(r.fingerprint_sha256)
            .map(|(id, fingerprint_sha256)| Certificate {
                id,
                fingerprint_sha256,
            }),
        remote_addr: r.remote_addr,
        connected_at: r.connected_at,
        authenticated_at: r.authenticated_at,
        disconnected_at: r.disconnected_at,
        disconnect_reason: r.disconnect_reason,
        transaction_count: r.transaction_count,
    }
}
fn transaction_dto(r: TransactionRow) -> TransactionDto {
    TransactionDto {
        id: r.id,
        session_id: r.session_id,
        registrar: r
            .registrar_id
            .zip(r.registrar_handle)
            .zip(r.registrar_name)
            .map(|((id, handle), name)| Registrar { id, handle, name }),
        command: r.command,
        object_name: None,
        cl_trid: r.cl_trid,
        sv_trid: r.sv_trid,
        request_xml: r.request_xml,
        response_xml: r.response_xml,
        response_code: r.response_code,
        delivery_status: r.delivery_status,
        delivery_error: r.delivery_error,
        started_at: r.started_at,
        finished_at: r.finished_at,
        duration_ms: r.duration_ms,
    }
}

pub(crate) async fn sessions(
    _admin: AdminSession,
    State(state): State<Arc<AppState>>,
    Query(q): Query<SessionQuery>,
) -> Result<Json<Page<SessionDto>>, StatusCode> {
    let (page, size) = paging(q.page, q.page_size)?;
    if let Some(s) = q.state.as_deref()
        && !matches!(s, "connected" | "authenticated" | "closed")
    {
        return Err(StatusCode::BAD_REQUEST);
    }
    let (items, total) = epp_queries::list_sessions(
        &state.db,
        page,
        size,
        q.registrar_id,
        q.state.as_deref(),
        q.remote_addr.as_deref(),
        q.date_from,
        q.date_to,
    )
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(Page {
        items: items.into_iter().map(session_dto).collect(),
        page,
        page_size: size,
        total,
        total_pages: total_pages(total, size),
    }))
}
pub(crate) async fn session(
    _admin: AdminSession,
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<SessionDto>, StatusCode> {
    epp_queries::get_session(&state.db, id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .map(session_dto)
        .map(Json)
        .ok_or(StatusCode::NOT_FOUND)
}
pub(crate) async fn transactions(
    _admin: AdminSession,
    State(state): State<Arc<AppState>>,
    Query(q): Query<TransactionQuery>,
) -> Result<Json<Page<TransactionDto>>, StatusCode> {
    let (page, size) = paging(q.page, q.page_size)?;
    if let Some(s) = q.delivery_status.as_deref()
        && !matches!(s, "delivered" | "failed" | "unknown")
    {
        return Err(StatusCode::BAD_REQUEST);
    }
    let (items, total) = epp_queries::list_transactions(
        &state.db,
        page,
        size,
        q.session_id,
        q.registrar_id,
        q.command.as_deref(),
        q.response_code,
        q.delivery_status.as_deref(),
        q.trid.as_deref(),
    )
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(Page {
        items: items.into_iter().map(transaction_dto).collect(),
        page,
        page_size: size,
        total,
        total_pages: total_pages(total, size),
    }))
}
pub(crate) async fn transaction(
    _admin: AdminSession,
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<TransactionDto>, StatusCode> {
    epp_queries::get_transaction(&state.db, id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .map(transaction_dto)
        .map(Json)
        .ok_or(StatusCode::NOT_FOUND)
}
