use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

pub(crate) async fn list(pool: &PgPool) -> Result<Vec<RegistrarRow>, sqlx::Error> {
    sqlx::query_as::<_, RegistrarRow>("SELECT id, handle, name, client_id, status, created_at, updated_at FROM registrars ORDER BY handle")
        .fetch_all(pool).await
}

pub(crate) async fn find(pool: &PgPool, id: Uuid) -> Result<Option<RegistrarRow>, sqlx::Error> {
    sqlx::query_as::<_, RegistrarRow>("SELECT id, handle, name, client_id, status, created_at, updated_at FROM registrars WHERE id = $1")
        .bind(id).fetch_optional(pool).await
}

pub(crate) async fn create(
    pool: &PgPool,
    handle: &str,
    name: &str,
    client_id: &str,
    password_hash: &str,
) -> Result<RegistrarRow, sqlx::Error> {
    sqlx::query_as::<_, RegistrarRow>("INSERT INTO registrars (id, handle, name, client_id, password_hash, status, created_at, updated_at) VALUES ($1,$2,$3,$4,$5,'active',$6,$6) RETURNING id, handle, name, client_id, status, created_at, updated_at")
        .bind(Uuid::new_v4()).bind(handle).bind(name).bind(client_id).bind(password_hash).bind(Utc::now())
        .fetch_one(pool).await
}

#[derive(sqlx::FromRow)]
pub(crate) struct AuthenticationRow {
    pub id: Uuid,
    pub password_hash: String,
}

pub(crate) async fn find_active_by_client_id(
    pool: &PgPool,
    client_id: &str,
) -> Result<Option<AuthenticationRow>, sqlx::Error> {
    sqlx::query_as(
        "SELECT id, password_hash FROM registrars WHERE client_id = $1 AND status = 'active'",
    )
    .bind(client_id)
    .fetch_optional(pool)
    .await
}

#[derive(sqlx::FromRow)]
pub(crate) struct RegistrarRow {
    pub id: Uuid,
    pub handle: String,
    pub name: String,
    pub client_id: String,
    pub status: String,
    #[allow(dead_code)]
    pub created_at: DateTime<Utc>,
    #[allow(dead_code)]
    pub updated_at: DateTime<Utc>,
}
