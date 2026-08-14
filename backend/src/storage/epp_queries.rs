use chrono::{DateTime, Utc};
use sqlx::{FromRow, PgPool, Postgres, QueryBuilder};
use uuid::Uuid;

#[derive(Debug, FromRow)]
pub(crate) struct SessionRow {
    pub id: Uuid,
    pub registrar_id: Option<Uuid>,
    pub registrar_handle: Option<String>,
    pub registrar_name: Option<String>,
    pub certificate_id: Option<Uuid>,
    pub fingerprint_sha256: Option<String>,
    pub remote_addr: String,
    pub connected_at: DateTime<Utc>,
    pub authenticated_at: Option<DateTime<Utc>>,
    pub disconnected_at: Option<DateTime<Utc>>,
    pub disconnect_reason: Option<String>,
    pub transaction_count: i64,
}

#[derive(Debug, FromRow)]
pub(crate) struct TransactionRow {
    pub id: Uuid,
    pub session_id: Uuid,
    pub registrar_id: Option<Uuid>,
    pub registrar_handle: Option<String>,
    pub registrar_name: Option<String>,
    pub command: String,
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

#[allow(clippy::too_many_arguments)]
pub(crate) async fn list_sessions(
    pool: &PgPool,
    page: i64,
    page_size: i64,
    registrar_id: Option<Uuid>,
    state: Option<&str>,
    remote_addr: Option<&str>,
    date_from: Option<DateTime<Utc>>,
    date_to: Option<DateTime<Utc>>,
) -> Result<(Vec<SessionRow>, i64), sqlx::Error> {
    let mut count = QueryBuilder::<Postgres>::new("SELECT COUNT(*) FROM epp_sessions s");
    let mut data = QueryBuilder::<Postgres>::new(
        "SELECT s.id, s.registrar_id, r.handle AS registrar_handle, r.name AS registrar_name, s.certificate_id, c.fingerprint_sha256, s.remote_addr, s.connected_at, s.authenticated_at, s.disconnected_at, s.disconnect_reason, (SELECT COUNT(*) FROM epp_transactions t WHERE t.session_id = s.id) AS transaction_count FROM epp_sessions s LEFT JOIN registrars r ON r.id = s.registrar_id LEFT JOIN registrar_certificates c ON c.id = s.certificate_id",
    );
    let mut filters = SessionFilters {
        first: true,
        count: 0,
    };
    add_session_filters(
        &mut count,
        &mut filters,
        registrar_id,
        state,
        remote_addr,
        date_from,
        date_to,
    );
    let total: i64 = count.build_query_scalar().fetch_one(pool).await?;
    let mut filters = SessionFilters {
        first: true,
        count: 0,
    };
    add_session_filters(
        &mut data,
        &mut filters,
        registrar_id,
        state,
        remote_addr,
        date_from,
        date_to,
    );
    data.push(" ORDER BY s.connected_at DESC LIMIT ")
        .push_bind(page_size)
        .push(" OFFSET ")
        .push_bind((page - 1) * page_size);
    Ok((data.build_query_as().fetch_all(pool).await?, total))
}

struct SessionFilters {
    first: bool,
    count: usize,
}
fn add_session_filters(
    q: &mut QueryBuilder<'_, Postgres>,
    f: &mut SessionFilters,
    registrar_id: Option<Uuid>,
    state: Option<&str>,
    remote: Option<&str>,
    from: Option<DateTime<Utc>>,
    to: Option<DateTime<Utc>>,
) {
    fn add(q: &mut QueryBuilder<'_, Postgres>, f: &mut SessionFilters, sql: &str) {
        q.push(if f.first { " WHERE " } else { " AND " }).push(sql);
        f.first = false;
        f.count += 1;
    }
    if let Some(v) = registrar_id {
        add(q, f, "s.registrar_id = ");
        q.push_bind(v);
    }
    if let Some(v) = state {
        add(
            q,
            f,
            "CASE WHEN s.disconnected_at IS NOT NULL THEN 'closed' WHEN s.authenticated_at IS NOT NULL THEN 'authenticated' ELSE 'connected' END = ",
        );
        q.push_bind(v.to_owned());
    }
    if let Some(v) = remote {
        add(q, f, "s.remote_addr ILIKE ");
        q.push_bind(format!("%{v}%"));
    }
    if let Some(v) = from {
        add(q, f, "s.connected_at >= ");
        q.push_bind(v);
    }
    if let Some(v) = to {
        add(q, f, "s.connected_at < ");
        q.push_bind(v);
    }
}

pub(crate) async fn get_session(
    pool: &PgPool,
    id: Uuid,
) -> Result<Option<SessionRow>, sqlx::Error> {
    sqlx::query_as("SELECT s.id, s.registrar_id, r.handle AS registrar_handle, r.name AS registrar_name, s.certificate_id, c.fingerprint_sha256, s.remote_addr, s.connected_at, s.authenticated_at, s.disconnected_at, s.disconnect_reason, (SELECT COUNT(*) FROM epp_transactions t WHERE t.session_id = s.id) AS transaction_count FROM epp_sessions s LEFT JOIN registrars r ON r.id = s.registrar_id LEFT JOIN registrar_certificates c ON c.id = s.certificate_id WHERE s.id = $1").bind(id).fetch_optional(pool).await
}

#[allow(clippy::too_many_arguments)]
pub(crate) async fn list_transactions(
    pool: &PgPool,
    page: i64,
    page_size: i64,
    session_id: Option<Uuid>,
    registrar_id: Option<Uuid>,
    command: Option<&str>,
    response_code: Option<i32>,
    delivery: Option<&str>,
    trid: Option<&str>,
) -> Result<(Vec<TransactionRow>, i64), sqlx::Error> {
    let mut count = QueryBuilder::<Postgres>::new("SELECT COUNT(*) FROM epp_transactions t");
    let mut data = QueryBuilder::<Postgres>::new(
        "SELECT t.id, t.session_id, t.registrar_id, r.handle AS registrar_handle, r.name AS registrar_name, t.command, t.cl_trid, t.sv_trid, NULL::text AS request_xml, NULL::text AS response_xml, t.response_code, t.delivery_status, NULL::text AS delivery_error, t.started_at, t.finished_at, t.duration_ms FROM epp_transactions t LEFT JOIN registrars r ON r.id = t.registrar_id",
    );
    let mut filters = TxFilters { first: true };
    add_tx_filters(
        &mut count,
        &mut filters,
        session_id,
        registrar_id,
        command,
        response_code,
        delivery,
        trid,
    );
    let total: i64 = count.build_query_scalar().fetch_one(pool).await?;
    let mut filters = TxFilters { first: true };
    add_tx_filters(
        &mut data,
        &mut filters,
        session_id,
        registrar_id,
        command,
        response_code,
        delivery,
        trid,
    );
    data.push(" ORDER BY t.started_at DESC LIMIT ")
        .push_bind(page_size)
        .push(" OFFSET ")
        .push_bind((page - 1) * page_size);
    Ok((data.build_query_as().fetch_all(pool).await?, total))
}

struct TxFilters {
    first: bool,
}
#[allow(clippy::too_many_arguments)]
fn add_tx_filters(
    q: &mut QueryBuilder<'_, Postgres>,
    f: &mut TxFilters,
    session_id: Option<Uuid>,
    registrar_id: Option<Uuid>,
    command: Option<&str>,
    response_code: Option<i32>,
    delivery: Option<&str>,
    trid: Option<&str>,
) {
    fn add(q: &mut QueryBuilder<'_, Postgres>, f: &mut TxFilters, sql: &str) {
        q.push(if f.first { " WHERE " } else { " AND " }).push(sql);
        f.first = false;
    }
    if let Some(v) = session_id {
        add(q, f, "t.session_id = ");
        q.push_bind(v);
    }
    if let Some(v) = registrar_id {
        add(q, f, "t.registrar_id = ");
        q.push_bind(v);
    }
    if let Some(v) = command {
        add(q, f, "t.command = ");
        q.push_bind(v.to_owned());
    }
    if let Some(v) = response_code {
        add(q, f, "t.response_code = ");
        q.push_bind(v);
    }
    if let Some(v) = delivery {
        add(q, f, "t.delivery_status = ");
        q.push_bind(v.to_owned());
    }
    if let Some(v) = trid {
        add(q, f, "(t.cl_trid ILIKE ");
        q.push_bind(format!("%{v}%"));
        q.push(" OR t.sv_trid ILIKE ");
        q.push_bind(format!("%{v}%"));
        q.push(")");
    }
}

pub(crate) async fn get_transaction(
    pool: &PgPool,
    id: Uuid,
) -> Result<Option<TransactionRow>, sqlx::Error> {
    sqlx::query_as("SELECT t.id, t.session_id, t.registrar_id, r.handle AS registrar_handle, r.name AS registrar_name, t.command, t.cl_trid, t.sv_trid, t.request_xml, t.response_xml, t.response_code, t.delivery_status, t.delivery_error, t.started_at, t.finished_at, t.duration_ms FROM epp_transactions t LEFT JOIN registrars r ON r.id = t.registrar_id WHERE t.id = $1").bind(id).fetch_optional(pool).await
}
