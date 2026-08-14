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
    pub registrar_handle: Option<String>,
    pub email: String,
    pub statuses: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, sqlx::FromRow)]
pub(crate) struct ContactDetailRow {
    pub id: Uuid,
    pub roid: String,
    pub sponsoring_registrar_id: Uuid,
    pub registrar_handle: Option<String>,
    pub email: String,
    pub voice: String,
    pub voice_extension: Option<String>,
    pub fax: Option<String>,
    pub fax_extension: Option<String>,
    pub name: String,
    pub organization: Option<String>,
    pub streets: Vec<String>,
    pub city: String,
    pub state_province: Option<String>,
    pub postal_code: Option<String>,
    pub country_code: String,
    pub disclose_flag: String,
    pub disclosure_fields: Vec<String>,
    pub statuses: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub(crate) async fn list_summaries(pool: &PgPool) -> Result<Vec<ContactSummaryRow>, sqlx::Error> {
    sqlx::query_as(
        r#"SELECT c.id, c.roid, c.sponsoring_registrar_id, r.handle AS registrar_handle, p.email,
                  COALESCE(array_agg(DISTINCT s.status) FILTER (WHERE s.status IS NOT NULL), '{}') AS statuses,
                  c.created_at, c.updated_at
           FROM contacts c JOIN registrars r ON r.id = c.sponsoring_registrar_id
           JOIN contact_phones p ON p.contact_id = c.id
           LEFT JOIN contact_statuses s ON s.contact_id = c.id
           GROUP BY c.id, c.roid, c.sponsoring_registrar_id, r.handle, p.email, c.created_at, c.updated_at
           ORDER BY c.created_at DESC"#,
    )
    .fetch_all(pool)
    .await
}

pub(crate) async fn find_detail(
    pool: &PgPool,
    id: Uuid,
) -> Result<Option<ContactDetailRow>, sqlx::Error> {
    sqlx::query_as(
        r#"SELECT c.id, c.roid, c.sponsoring_registrar_id, r.handle AS registrar_handle,
                  p.email, p.voice, p.voice_extension, p.fax, p.fax_extension,
                  pi.name, pi.organization,
                  COALESCE((SELECT array_agg(ps.street ORDER BY ps.position) FROM contact_postal_streets ps WHERE ps.contact_id = c.id AND ps.info_type = 'international'), '{}') AS streets,
                  pi.city, pi.state_province, pi.postal_code, pi.country_code,
                  c.disclose_flag,
                  COALESCE((SELECT array_agg(df.field ORDER BY df.field) FROM contact_disclosure_fields df WHERE df.contact_id = c.id), '{}') AS disclosure_fields,
                  COALESCE((SELECT array_agg(DISTINCT s.status) FROM contact_statuses s WHERE s.contact_id = c.id), '{}') AS statuses,
                  c.created_at, c.updated_at
           FROM contacts c JOIN registrars r ON r.id = c.sponsoring_registrar_id
           JOIN contact_phones p ON p.contact_id = c.id
           JOIN contact_postal_info pi ON pi.contact_id = c.id AND pi.info_type = 'international'
           WHERE c.id = $1"#,
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

pub(crate) async fn find_identity_by_roid(
    pool: &PgPool,
    roid: &str,
) -> Result<Option<ContactIdentityRow>, sqlx::Error> {
    sqlx::query_as(
        "SELECT id, roid, sponsoring_registrar_id, created_by, created_at, updated_by, updated_at, transferred_at, auth_info_ciphertext, disclose_flag FROM contacts WHERE roid = $1",
    )
    .bind(roid)
    .fetch_optional(pool)
    .await
}

pub(crate) async fn delete(pool: &PgPool, id: Uuid) -> Result<bool, sqlx::Error> {
    let result = sqlx::query("DELETE FROM contacts WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected() == 1)
}

pub(crate) async fn update_email_auth(
    pool: &PgPool,
    id: Uuid,
    email: Option<&str>,
    auth_info_ciphertext: Option<&str>,
    voice: Option<&str>,
    fax: Option<&str>,
    organization: Option<&str>,
) -> Result<bool, sqlx::Error> {
    let mut tx = pool.begin().await?;
    let result = sqlx::query(
        "UPDATE contacts c SET auth_info_ciphertext = COALESCE($2, c.auth_info_ciphertext), updated_at = NOW() WHERE c.id = $1",
    )
    .bind(id)
    .bind(auth_info_ciphertext)
    .execute(&mut *tx)
    .await?;
    if result.rows_affected() == 0 {
        return Ok(false);
    }
    if email.is_some() || voice.is_some() || fax.is_some() {
        sqlx::query("UPDATE contact_phones SET email = COALESCE($2, email), voice = COALESCE($3, voice), fax = COALESCE($4, fax) WHERE contact_id = $1")
            .bind(id).bind(email).bind(voice).bind(fax).execute(&mut *tx).await?;
    }
    if let Some(organization) = organization {
        sqlx::query("UPDATE contact_postal_info SET organization = $2 WHERE contact_id = $1 AND info_type = 'international'")
            .bind(id).bind(organization).execute(&mut *tx).await?;
    }
    tx.commit().await?;
    Ok(true)
}

pub(crate) async fn update_client_statuses(
    pool: &PgPool,
    id: Uuid,
    add: &[&str],
    remove: &[&str],
) -> Result<(), sqlx::Error> {
    let mut tx = pool.begin().await?;
    for status in remove {
        sqlx::query("DELETE FROM contact_statuses WHERE contact_id = $1 AND status = $2 AND source = 'client'")
            .bind(id).bind(status).execute(&mut *tx).await?;
    }
    for status in add {
        sqlx::query("INSERT INTO contact_statuses (contact_id, status, source) VALUES ($1, $2, 'client') ON CONFLICT DO NOTHING")
            .bind(id).bind(status).execute(&mut *tx).await?;
    }
    sqlx::query("UPDATE contacts SET updated_at = NOW() WHERE id = $1")
        .bind(id)
        .execute(&mut *tx)
        .await?;
    tx.commit().await
}

pub(crate) async fn update_postal_info(
    pool: &PgPool,
    id: Uuid,
    city: Option<&str>,
    state_province: Option<&str>,
    postal_code: Option<&str>,
    country_code: Option<&str>,
    streets: &[&str],
) -> Result<(), sqlx::Error> {
    let mut tx = pool.begin().await?;
    sqlx::query("UPDATE contact_postal_info SET city = COALESCE($2, city), state_province = COALESCE($3, state_province), postal_code = COALESCE($4, postal_code), country_code = COALESCE($5, country_code) WHERE contact_id = $1 AND info_type = 'international'")
        .bind(id).bind(city).bind(state_province).bind(postal_code).bind(country_code)
        .execute(&mut *tx).await?;
    if !streets.is_empty() {
        sqlx::query("DELETE FROM contact_postal_streets WHERE contact_id = $1 AND info_type = 'international'")
            .bind(id).execute(&mut *tx).await?;
        for (position, street) in streets.iter().enumerate() {
            sqlx::query("INSERT INTO contact_postal_streets (contact_id, info_type, position, street) VALUES ($1, 'international', $2, $3)")
                .bind(id).bind((position + 1) as i16).bind(street).execute(&mut *tx).await?;
        }
    }
    sqlx::query("UPDATE contacts SET updated_at = NOW() WHERE id = $1")
        .bind(id)
        .execute(&mut *tx)
        .await?;
    tx.commit().await
}

pub(crate) async fn update_disclose_flag(
    pool: &PgPool,
    id: Uuid,
    flag: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE contacts SET disclose_flag = $2, updated_at = NOW() WHERE id = $1")
        .bind(id)
        .bind(flag)
        .execute(pool)
        .await?;
    Ok(())
}

pub(crate) async fn update_disclosure_fields(
    pool: &PgPool,
    id: Uuid,
    fields: &[&str],
) -> Result<(), sqlx::Error> {
    let mut tx = pool.begin().await?;
    sqlx::query("DELETE FROM contact_disclosure_fields WHERE contact_id = $1")
        .bind(id)
        .execute(&mut *tx)
        .await?;
    for field in fields {
        sqlx::query("INSERT INTO contact_disclosure_fields (contact_id, field) VALUES ($1, $2)")
            .bind(id)
            .bind(field)
            .execute(&mut *tx)
            .await?;
    }
    sqlx::query("UPDATE contacts SET updated_at = NOW() WHERE id = $1")
        .bind(id)
        .execute(&mut *tx)
        .await?;
    tx.commit().await
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
