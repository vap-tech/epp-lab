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

#[cfg(test)]
mod tests {
    use chrono::Utc;

    use super::*;

    #[ignore = "requires PostgreSQL; run through just test-with-db"]
    #[sqlx::test(migrations = "../backend/migrations")]
    async fn stores_and_reads_contact_identity_ciphertext(pool: PgPool) {
        let registrar_id = Uuid::new_v4();
        let now = Utc::now();
        sqlx::query(
            "INSERT INTO registrars (id, handle, name, client_id, password_hash, status, created_at, updated_at) VALUES ($1, 'REG-1', 'Registrar', 'client-1', 'not-used', 'active', $2, $2)",
        )
        .bind(registrar_id)
        .bind(now)
        .execute(&pool)
        .await
        .unwrap();

        let row = ContactIdentityRow {
            id: Uuid::new_v4(),
            roid: "SH8013-EXAMPLE".to_owned(),
            sponsoring_registrar_id: registrar_id,
            created_by: registrar_id,
            created_at: now,
            updated_by: registrar_id,
            updated_at: now,
            transferred_at: None,
            auth_info_ciphertext: "ciphertext-not-plaintext".to_owned(),
            disclose_flag: "private".to_owned(),
        };

        create_identity(&pool, &row).await.unwrap();
        let stored = find_identity(&pool, row.id).await.unwrap().unwrap();
        assert_eq!(stored.roid, row.roid);
        assert_eq!(stored.auth_info_ciphertext, row.auth_info_ciphertext);
        assert_ne!(stored.auth_info_ciphertext, "plain-auth-info");
        assert!(exists(&pool, row.id).await.unwrap());
    }
}
