use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

pub(crate) async fn create(
    pool: &PgPool,
    registrar_id: Uuid,
    certificate_id: Uuid,
    remote_addr: &str,
) -> Result<Uuid, sqlx::Error> {
    let id = Uuid::new_v4();
    sqlx::query("INSERT INTO epp_sessions (id, registrar_id, certificate_id, remote_addr, connected_at) VALUES ($1,$2,$3,$4,$5)")
        .bind(id).bind(registrar_id).bind(certificate_id).bind(remote_addr).bind(Utc::now()).execute(pool).await?;
    Ok(id)
}

pub(crate) async fn disconnect(
    pool: &PgPool,
    session_id: Uuid,
    reason: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE epp_sessions SET disconnected_at = $2, disconnect_reason = $3 WHERE id = $1",
    )
    .bind(session_id)
    .bind(Utc::now())
    .bind(reason)
    .execute(pool)
    .await?;
    Ok(())
}

pub(crate) async fn mark_authenticated(pool: &PgPool, session_id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE epp_sessions SET authenticated_at = $2 WHERE id = $1")
        .bind(session_id)
        .bind(Utc::now())
        .execute(pool)
        .await?;
    Ok(())
}

pub(crate) async fn create_transaction(
    pool: &PgPool,
    session_id: Uuid,
    registrar_id: Option<Uuid>,
    command: &str,
    cl_trid: Option<&str>,
    sv_trid: &str,
    request_xml: &str,
) -> Result<Uuid, sqlx::Error> {
    let id = Uuid::new_v4();
    sqlx::query("INSERT INTO epp_transactions (id, session_id, registrar_id, command, cl_trid, sv_trid, request_xml, started_at, delivery_status) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,'unknown')")
        .bind(id).bind(session_id).bind(registrar_id).bind(command).bind(cl_trid).bind(sv_trid).bind(request_xml).bind(Utc::now()).execute(pool).await?;
    Ok(id)
}

pub(crate) async fn finish_transaction(
    pool: &PgPool,
    transaction_id: Uuid,
    response_xml: Option<&str>,
    response_code: Option<i32>,
    duration_ms: i64,
) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE epp_transactions SET response_xml = $2, response_code = $3, finished_at = $4, duration_ms = $5, delivery_status = 'delivered', delivery_error = NULL WHERE id = $1")
        .bind(transaction_id).bind(response_xml).bind(response_code).bind(Utc::now()).bind(duration_ms).execute(pool).await?;
    Ok(())
}

#[allow(dead_code)]
pub(crate) async fn mark_delivery_failed(
    pool: &PgPool,
    transaction_id: Uuid,
    error: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE epp_transactions SET delivery_status = 'failed', delivery_error = $2, finished_at = COALESCE(finished_at, $3) WHERE id = $1")
        .bind(transaction_id)
        .bind(error)
        .bind(Utc::now())
        .execute(pool)
        .await?;
    Ok(())
}
