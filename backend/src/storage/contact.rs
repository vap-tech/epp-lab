use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

#[allow(dead_code)]
#[derive(Debug, sqlx::FromRow)]
pub(crate) struct ContactIdentityRow {
    pub id: Uuid,
    pub roid: String,
    pub sponsoring_registrar_id: Uuid,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_by: Uuid,
    pub updated_at: DateTime<Utc>,
    pub transferred_at: Option<DateTime<Utc>>,
    pub auth_info_ciphertext: String,
    pub disclose_flag: String,
}

#[allow(dead_code)]
pub(crate) async fn exists(pool: &PgPool, id: Uuid) -> Result<bool, sqlx::Error> {
    sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM contacts WHERE id = $1)")
        .bind(id)
        .fetch_one(pool)
        .await
}

#[allow(dead_code)]
pub(crate) async fn find_identity(
    pool: &PgPool,
    id: Uuid,
) -> Result<Option<ContactIdentityRow>, sqlx::Error> {
    sqlx::query_as(
        "SELECT id, roid, sponsoring_registrar_id, created_by, created_at, updated_by, updated_at, transferred_at, auth_info_ciphertext, disclose_flag FROM contacts WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(pool)
    .await
}

#[allow(dead_code)]
pub(crate) async fn create_identity(
    pool: &PgPool,
    row: &ContactIdentityRow,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO contacts (id, roid, sponsoring_registrar_id, created_by, created_at, updated_by, updated_at, transferred_at, auth_info_ciphertext, disclose_flag) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10)",
    )
    .bind(row.id)
    .bind(&row.roid)
    .bind(row.sponsoring_registrar_id)
    .bind(row.created_by)
    .bind(row.created_at)
    .bind(row.updated_by)
    .bind(row.updated_at)
    .bind(row.transferred_at)
    .bind(&row.auth_info_ciphertext)
    .bind(&row.disclose_flag)
    .execute(pool)
    .await
    .map(|_| ())
}
