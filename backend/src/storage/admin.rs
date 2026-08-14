use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(sqlx::FromRow)]
pub(crate) struct AdminUserRow {
    pub id: Uuid,
    pub username: String,
    pub password_hash: String,
}

#[derive(sqlx::FromRow)]
pub(crate) struct AdminSessionRow {
    pub id: Uuid,
    pub admin_user_id: Uuid,
    pub username: String,
    pub csrf_token_hash: String,
}

pub(crate) async fn find_active_user(
    pool: &PgPool,
    username: &str,
) -> Result<Option<AdminUserRow>, sqlx::Error> {
    sqlx::query_as("SELECT id, username, password_hash FROM admin_users WHERE username = $1 AND status = 'active'")
        .bind(username)
        .fetch_optional(pool)
        .await
}

pub(crate) async fn create_session(
    pool: &PgPool,
    user_id: Uuid,
    token_hash: &str,
    csrf_token_hash: &str,
    now: DateTime<Utc>,
    expires_at: DateTime<Utc>,
) -> Result<Uuid, sqlx::Error> {
    let id = Uuid::new_v4();
    sqlx::query("INSERT INTO admin_sessions (id, admin_user_id, token_hash, csrf_token_hash, created_at, last_seen_at, expires_at) VALUES ($1,$2,$3,$4,$5,$5,$6)")
        .bind(id).bind(user_id).bind(token_hash).bind(csrf_token_hash).bind(now).bind(expires_at)
        .execute(pool).await?;
    Ok(id)
}

pub(crate) async fn find_session(
    pool: &PgPool,
    token_hash: &str,
    now: DateTime<Utc>,
) -> Result<Option<AdminSessionRow>, sqlx::Error> {
    sqlx::query_as("SELECT s.id, s.admin_user_id, u.username, s.csrf_token_hash FROM admin_sessions s JOIN admin_users u ON u.id = s.admin_user_id WHERE s.token_hash = $1 AND s.revoked_at IS NULL AND s.expires_at > $2 AND u.status = 'active'")
        .bind(token_hash).bind(now).fetch_optional(pool).await
}

pub(crate) async fn rotate_csrf(
    pool: &PgPool,
    session_id: Uuid,
    csrf_token_hash: &str,
    now: DateTime<Utc>,
) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE admin_sessions SET csrf_token_hash = $2, last_seen_at = $3 WHERE id = $1")
        .bind(session_id)
        .bind(csrf_token_hash)
        .bind(now)
        .execute(pool)
        .await?;
    Ok(())
}

pub(crate) async fn revoke(
    pool: &PgPool,
    token_hash: &str,
    now: DateTime<Utc>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE admin_sessions SET revoked_at = $2 WHERE token_hash = $1 AND revoked_at IS NULL",
    )
    .bind(token_hash)
    .bind(now)
    .execute(pool)
    .await?;
    Ok(())
}
