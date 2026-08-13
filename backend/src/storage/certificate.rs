use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(sqlx::FromRow)]
pub(crate) struct CertificateRow {
    pub id: Uuid,
    pub fingerprint_sha256: String,
    pub subject: String,
    pub serial_number: Option<String>,
    pub not_before: DateTime<Utc>,
    pub not_after: DateTime<Utc>,
    pub status: String,
    pub created_at: DateTime<Utc>,
}

pub(crate) async fn list(
    pool: &PgPool,
    registrar_id: Uuid,
) -> Result<Vec<CertificateRow>, sqlx::Error> {
    sqlx::query_as("SELECT id, fingerprint_sha256, subject, serial_number, not_before, not_after, status, created_at FROM registrar_certificates WHERE registrar_id = $1 ORDER BY created_at DESC").bind(registrar_id).fetch_all(pool).await
}

pub(crate) async fn create(
    pool: &PgPool,
    registrar_id: Uuid,
    fingerprint: &str,
    subject: &str,
    serial: &str,
    not_before: DateTime<Utc>,
    not_after: DateTime<Utc>,
) -> Result<CertificateRow, sqlx::Error> {
    sqlx::query_as("INSERT INTO registrar_certificates (id, registrar_id, fingerprint_sha256, subject, serial_number, not_before, not_after, status, created_at) VALUES ($1,$2,$3,$4,$5,$6,$7,'active',$8) RETURNING id, fingerprint_sha256, subject, serial_number, not_before, not_after, status, created_at")
        .bind(Uuid::new_v4()).bind(registrar_id).bind(fingerprint).bind(subject).bind(serial).bind(not_before).bind(not_after).bind(Utc::now()).fetch_one(pool).await
}

#[derive(sqlx::FromRow)]
pub(crate) struct IdentityRow {
    pub certificate_id: Uuid,
    pub registrar_id: Uuid,
    pub fingerprint_sha256: String,
}

pub(crate) async fn find_active_identity(
    pool: &PgPool,
    fingerprint: &str,
) -> Result<Option<IdentityRow>, sqlx::Error> {
    sqlx::query_as(
        "SELECT c.id AS certificate_id, c.registrar_id, c.fingerprint_sha256
         FROM registrar_certificates c
         JOIN registrars r ON r.id = c.registrar_id
         WHERE c.fingerprint_sha256 = $1
           AND c.status = 'active'
           AND r.status = 'active'
           AND c.not_before <= NOW()
           AND c.not_after >= NOW()",
    )
    .bind(fingerprint)
    .fetch_optional(pool)
    .await
}
