use chrono::{DateTime, Utc};
use sqlx::PgPool;
use sqlx::{Postgres, Transaction};
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
#[derive(Debug, sqlx::FromRow)]
pub(crate) struct ContactSummaryRow {
    pub id: Uuid,
    pub roid: String,
    pub sponsoring_registrar_id: Uuid,
    pub email: String,
    pub statuses: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub(crate) async fn list_summaries(pool: &PgPool) -> Result<Vec<ContactSummaryRow>, sqlx::Error> {
    sqlx::query_as(
        r#"SELECT c.id, c.roid, c.sponsoring_registrar_id, p.email,
                  COALESCE(array_agg(DISTINCT s.status) FILTER (WHERE s.status IS NOT NULL), '{}') AS statuses,
                  c.created_at, c.updated_at
           FROM contacts c
           JOIN contact_phones p ON p.contact_id = c.id
           LEFT JOIN contact_statuses s ON s.contact_id = c.id
           GROUP BY c.id, c.roid, c.sponsoring_registrar_id, p.email, c.created_at, c.updated_at
           ORDER BY c.created_at DESC"#,
    )
    .fetch_all(pool)
    .await
}

pub(crate) async fn find_summary(
    pool: &PgPool,
    id: Uuid,
) -> Result<Option<ContactSummaryRow>, sqlx::Error> {
    sqlx::query_as(
        r#"SELECT c.id, c.roid, c.sponsoring_registrar_id, p.email,
                  COALESCE(array_agg(DISTINCT s.status) FILTER (WHERE s.status IS NOT NULL), '{}') AS statuses,
                  c.created_at, c.updated_at
           FROM contacts c
           JOIN contact_phones p ON p.contact_id = c.id
           LEFT JOIN contact_statuses s ON s.contact_id = c.id
           WHERE c.id = $1
           GROUP BY c.id, c.roid, c.sponsoring_registrar_id, p.email, c.created_at, c.updated_at"#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await
}

#[allow(dead_code)]
pub(crate) async fn exists(pool: &PgPool, id: Uuid) -> Result<bool, sqlx::Error> {
    sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM contacts WHERE id = $1)")
        .bind(id)
        .fetch_one(pool)
        .await
}

#[allow(dead_code)]
pub(crate) async fn exists_by_roid(pool: &PgPool, roid: &str) -> Result<bool, sqlx::Error> {
    sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM contacts WHERE roid = $1)")
        .bind(roid)
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

#[allow(dead_code)]
pub(crate) async fn create(
    pool: &PgPool,
    contact: &crate::domain::contact::Contact,
) -> Result<(), sqlx::Error> {
    let mut tx = pool.begin().await?;
    sqlx::query("INSERT INTO contacts (id, roid, sponsoring_registrar_id, created_by, created_at, updated_by, updated_at, transferred_at, auth_info_ciphertext, disclose_flag) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10)")
        .bind(contact.id.into_uuid()).bind(contact.roid.as_str())
        .bind(contact.sponsoring_registrar_id).bind(contact.created_by).bind(contact.created_at)
        .bind(contact.updated_by).bind(contact.updated_at).bind(contact.transferred_at)
        .bind(&contact.auth_info).bind(disclosure_flag(contact.disclose.flag))
        .execute(&mut *tx).await?;
    insert_postal_info(
        &mut tx,
        contact.id.into_uuid(),
        "international",
        &contact.postal_info.international,
    )
    .await?;
    if let Some(localized) = &contact.postal_info.localized {
        insert_postal_info(&mut tx, contact.id.into_uuid(), "localized", localized).await?;
    }
    sqlx::query("INSERT INTO contact_phones (contact_id, voice, voice_extension, fax, fax_extension, email) VALUES ($1,$2,$3,$4,$5,$6)")
        .bind(contact.id.into_uuid()).bind(&contact.voice.number).bind(&contact.voice.extension)
        .bind(contact.fax.as_ref().map(|phone| phone.number.as_str()))
        .bind(contact.fax.as_ref().and_then(|phone| phone.extension.as_deref()))
        .bind(contact.email.as_str()).execute(&mut *tx).await?;
    for status in contact
        .client_statuses
        .iter()
        .chain(contact.server_statuses.iter())
    {
        sqlx::query("INSERT INTO contact_statuses (contact_id, status, source) VALUES ($1,$2,$3)")
            .bind(contact.id.into_uuid())
            .bind(status_value(*status))
            .bind(if contact.client_statuses.contains(status) {
                "client"
            } else {
                "server"
            })
            .execute(&mut *tx)
            .await?;
    }
    for field in &contact.disclose.fields {
        sqlx::query("INSERT INTO contact_disclosure_fields (contact_id, field) VALUES ($1,$2)")
            .bind(contact.id.into_uuid())
            .bind(disclosure_field(*field))
            .execute(&mut *tx)
            .await?;
    }
    tx.commit().await
}

async fn insert_postal_info(
    tx: &mut Transaction<'_, Postgres>,
    contact_id: Uuid,
    info_type: &str,
    info: &crate::domain::contact::PostalInfo,
) -> Result<(), sqlx::Error> {
    sqlx::query("INSERT INTO contact_postal_info (contact_id, info_type, name, organization, city, state_province, postal_code, country_code) VALUES ($1,$2,$3,$4,$5,$6,$7,$8)")
        .bind(contact_id).bind(info_type).bind(&info.name).bind(&info.organization)
        .bind(&info.address.city).bind(&info.address.state_province).bind(&info.address.postal_code)
        .bind(info.address.country_code.as_str()).execute(&mut **tx).await?;
    for (position, street) in info.address.streets.iter().enumerate() {
        sqlx::query("INSERT INTO contact_postal_streets (contact_id, info_type, position, street) VALUES ($1,$2,$3,$4)")
            .bind(contact_id).bind(info_type).bind((position + 1) as i16).bind(street)
            .execute(&mut **tx).await?;
    }
    Ok(())
}

fn disclosure_flag(flag: crate::domain::contact::DisclosureFlag) -> &'static str {
    match flag {
        crate::domain::contact::DisclosureFlag::Public => "public",
        crate::domain::contact::DisclosureFlag::Private => "private",
    }
}
fn disclosure_field(field: crate::domain::contact::DisclosureField) -> &'static str {
    match field {
        crate::domain::contact::DisclosureField::Name => "name",
        crate::domain::contact::DisclosureField::Organization => "organization",
        crate::domain::contact::DisclosureField::Address => "address",
        crate::domain::contact::DisclosureField::Voice => "voice",
        crate::domain::contact::DisclosureField::Fax => "fax",
        crate::domain::contact::DisclosureField::Email => "email",
    }
}
fn status_value(status: crate::domain::contact::ContactStatus) -> &'static str {
    match status {
        crate::domain::contact::ContactStatus::ClientDeleteProhibited => "clientDeleteProhibited",
        crate::domain::contact::ContactStatus::ClientTransferProhibited => {
            "clientTransferProhibited"
        }
        crate::domain::contact::ContactStatus::ClientUpdateProhibited => "clientUpdateProhibited",
        crate::domain::contact::ContactStatus::Linked => "linked",
        crate::domain::contact::ContactStatus::Ok => "ok",
        crate::domain::contact::ContactStatus::PendingCreate => "pendingCreate",
        crate::domain::contact::ContactStatus::PendingDelete => "pendingDelete",
        crate::domain::contact::ContactStatus::PendingTransfer => "pendingTransfer",
        crate::domain::contact::ContactStatus::PendingUpdate => "pendingUpdate",
    }
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
        assert!(exists_by_roid(&pool, &row.roid).await.unwrap());
        assert!(!exists_by_roid(&pool, "SH404-NOT-FOUND").await.unwrap());
    }
}
