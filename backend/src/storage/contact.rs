use chrono::{DateTime, Utc};
use sqlx::{PgPool, QueryBuilder};
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
    pub created_by_handle: Option<String>,
    pub updated_by_handle: Option<String>,
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
    pub localized_name: Option<String>,
    pub localized_organization: Option<String>,
    pub localized_streets: Vec<String>,
    pub localized_city: Option<String>,
    pub localized_state_province: Option<String>,
    pub localized_postal_code: Option<String>,
    pub localized_country_code: Option<String>,
    pub disclose_flag: String,
    pub disclosure_fields: Vec<String>,
    pub statuses: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub transferred_at: Option<DateTime<Utc>>,
}

pub(crate) struct ContactListQuery<'a> {
    pub page: i64,
    pub page_size: i64,
    pub registrar_id: Option<Uuid>,
    pub status: Option<&'a str>,
    pub search: Option<&'a str>,
    pub created_from: Option<DateTime<Utc>>,
    pub created_to: Option<DateTime<Utc>>,
}

fn push_contact_filters<'a>(query: &mut QueryBuilder<'a, Postgres>, params: &ContactListQuery<'a>) {
    if let Some(registrar_id) = params.registrar_id {
        query
            .push(" AND c.sponsoring_registrar_id = ")
            .push_bind(registrar_id);
    }
    if let Some(status) = params.status {
        query
            .push(" AND EXISTS (SELECT 1 FROM contact_statuses cs WHERE cs.contact_id = c.id AND cs.status = ")
            .push_bind(status)
            .push(")");
    }
    if let Some(search) = params.search {
        let pattern = format!("%{search}%");
        query
            .push(" AND (c.roid ILIKE ")
            .push_bind(pattern.clone())
            .push(" OR p.email ILIKE ")
            .push_bind(pattern)
            .push(")");
    }
    if let Some(created_from) = params.created_from {
        query.push(" AND c.created_at >= ").push_bind(created_from);
    }
    if let Some(created_to) = params.created_to {
        query.push(" AND c.created_at <= ").push_bind(created_to);
    }
}

pub(crate) async fn list_summaries(
    pool: &PgPool,
    query_params: ContactListQuery<'_>,
) -> Result<(Vec<ContactSummaryRow>, i64), sqlx::Error> {
    let mut count = QueryBuilder::<Postgres>::new(
        "SELECT COUNT(*) FROM contacts c JOIN contact_phones p ON p.contact_id = c.id WHERE TRUE",
    );
    push_contact_filters(&mut count, &query_params);
    let total: i64 = count.build_query_scalar().fetch_one(pool).await?;

    let mut query = QueryBuilder::<Postgres>::new(
        r#"SELECT c.id, c.roid, c.sponsoring_registrar_id, r.handle AS registrar_handle, p.email,
                  COALESCE(array_agg(DISTINCT s.status) FILTER (WHERE s.status IS NOT NULL), '{}') AS statuses,
                  c.created_at, c.updated_at
           FROM contacts c JOIN registrars r ON r.id = c.sponsoring_registrar_id
           JOIN contact_phones p ON p.contact_id = c.id
           LEFT JOIN contact_statuses s ON s.contact_id = c.id
        "#,
    );
    query.push(" WHERE TRUE");
    push_contact_filters(&mut query, &query_params);
    query.push(" GROUP BY c.id, c.roid, c.sponsoring_registrar_id, r.handle, p.email, c.created_at, c.updated_at ORDER BY c.created_at DESC LIMIT ")
        .push_bind(query_params.page_size)
        .push(" OFFSET ")
        .push_bind((query_params.page - 1) * query_params.page_size);
    let rows = query.build_query_as().fetch_all(pool).await?;
    Ok((rows, total))
}

pub(crate) async fn find_detail(
    pool: &PgPool,
    id: Uuid,
) -> Result<Option<ContactDetailRow>, sqlx::Error> {
    sqlx::query_as(
        r#"SELECT c.id, c.roid, c.sponsoring_registrar_id, r.handle AS registrar_handle,
                  cr.handle AS created_by_handle, up.handle AS updated_by_handle,
                  p.email, p.voice, p.voice_extension, p.fax, p.fax_extension,
                  pi.name, pi.organization,
                  COALESCE((SELECT array_agg(ps.street ORDER BY ps.position) FROM contact_postal_streets ps WHERE ps.contact_id = c.id AND ps.info_type = 'international'), '{}') AS streets,
                  pi.city, pi.state_province, pi.postal_code, pi.country_code,
                  lpi.name AS localized_name, lpi.organization AS localized_organization,
                  COALESCE((SELECT array_agg(ps.street ORDER BY ps.position) FROM contact_postal_streets ps WHERE ps.contact_id = c.id AND ps.info_type = 'localized'), '{}') AS localized_streets,
                  lpi.city AS localized_city, lpi.state_province AS localized_state_province,
                  lpi.postal_code AS localized_postal_code, lpi.country_code AS localized_country_code,
                  c.disclose_flag,
                  COALESCE((SELECT array_agg(df.field ORDER BY df.field) FROM contact_disclosure_fields df WHERE df.contact_id = c.id), '{}') AS disclosure_fields,
                  COALESCE((SELECT array_agg(DISTINCT s.status) FROM contact_statuses s WHERE s.contact_id = c.id), '{}') AS statuses,
                  c.created_at, c.updated_at, c.transferred_at
           FROM contacts c JOIN registrars r ON r.id = c.sponsoring_registrar_id
           JOIN registrars cr ON cr.id = c.created_by
           JOIN registrars up ON up.id = c.updated_by
           JOIN contact_phones p ON p.contact_id = c.id
           JOIN contact_postal_info pi ON pi.contact_id = c.id AND pi.info_type = 'international'
           LEFT JOIN contact_postal_info lpi ON lpi.contact_id = c.id AND lpi.info_type = 'localized'
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

pub(crate) async fn has_status(pool: &PgPool, id: Uuid, status: &str) -> Result<bool, sqlx::Error> {
    sqlx::query_scalar(
        "SELECT EXISTS (SELECT 1 FROM contact_statuses WHERE contact_id = $1 AND status = $2)",
    )
    .bind(id)
    .bind(status)
    .fetch_one(pool)
    .await
}

pub(crate) struct ContactUpdate<'a> {
    pub id: Uuid,
    pub updated_by: Uuid,
    pub auth_info_ciphertext: Option<&'a str>,
    pub email: Option<&'a str>,
    pub voice: Option<&'a str>,
    pub fax: Option<Option<&'a str>>,
    pub organization: Option<Option<&'a str>>,
    pub city: Option<&'a str>,
    pub state_province: Option<Option<&'a str>>,
    pub postal_code: Option<Option<&'a str>>,
    pub country_code: Option<&'a str>,
    pub streets: &'a [&'a str],
    pub add_statuses: &'a [&'a str],
    pub remove_statuses: &'a [&'a str],
    pub disclose_flag: Option<&'a str>,
    pub disclosure_fields: Option<&'a [&'a str]>,
}

pub(crate) async fn apply_update(
    pool: &PgPool,
    update: ContactUpdate<'_>,
) -> Result<bool, sqlx::Error> {
    let mut tx = pool.begin().await?;
    let result = sqlx::query("UPDATE contacts SET auth_info_ciphertext = COALESCE($2, auth_info_ciphertext), disclose_flag = COALESCE($3, disclose_flag), updated_by = $4, updated_at = NOW() WHERE id = $1")
        .bind(update.id)
        .bind(update.auth_info_ciphertext)
        .bind(update.disclose_flag)
        .bind(update.updated_by)
        .execute(&mut *tx)
        .await?;
    if result.rows_affected() == 0 {
        return Ok(false);
    }
    if update.email.is_some() || update.voice.is_some() || update.fax.is_some() {
        sqlx::query("UPDATE contact_phones SET email = COALESCE($2, email), voice = COALESCE($3, voice), fax = CASE WHEN $4 THEN $5 ELSE fax END WHERE contact_id = $1")
            .bind(update.id)
            .bind(update.email)
            .bind(update.voice)
            .bind(update.fax.is_some())
            .bind(update.fax.flatten())
            .execute(&mut *tx)
            .await?;
    }
    if update.organization.is_some()
        || update.city.is_some()
        || update.state_province.is_some()
        || update.postal_code.is_some()
        || update.country_code.is_some()
    {
        sqlx::query("UPDATE contact_postal_info SET organization = CASE WHEN $2 THEN $3 ELSE organization END, city = COALESCE($4, city), state_province = CASE WHEN $5 THEN $6 ELSE state_province END, postal_code = CASE WHEN $7 THEN $8 ELSE postal_code END, country_code = COALESCE($9, country_code) WHERE contact_id = $1 AND info_type = 'international'")
            .bind(update.id)
            .bind(update.organization.is_some())
            .bind(update.organization.flatten())
            .bind(update.city)
            .bind(update.state_province.is_some())
            .bind(update.state_province.flatten())
            .bind(update.postal_code.is_some())
            .bind(update.postal_code.flatten())
            .bind(update.country_code)
            .execute(&mut *tx)
            .await?;
    }
    if !update.streets.is_empty() {
        sqlx::query(
            "DELETE FROM contact_postal_streets WHERE contact_id = $1 AND info_type = 'international'",
        )
        .bind(update.id)
        .execute(&mut *tx)
        .await?;
        for (position, street) in update.streets.iter().enumerate() {
            sqlx::query("INSERT INTO contact_postal_streets (contact_id, info_type, position, street) VALUES ($1, 'international', $2, $3)")
                .bind(update.id)
                .bind((position + 1) as i16)
                .bind(street)
                .execute(&mut *tx)
                .await?;
        }
    }
    for status in update.remove_statuses {
        sqlx::query(
            "DELETE FROM contact_statuses WHERE contact_id = $1 AND status = $2 AND source = 'client'",
        )
        .bind(update.id)
        .bind(status)
        .execute(&mut *tx)
        .await?;
    }
    for status in update.add_statuses {
        sqlx::query("INSERT INTO contact_statuses (contact_id, status, source) VALUES ($1, $2, 'client') ON CONFLICT DO NOTHING")
            .bind(update.id)
            .bind(status)
            .execute(&mut *tx)
            .await?;
    }
    if let Some(fields) = update.disclosure_fields {
        sqlx::query("DELETE FROM contact_disclosure_fields WHERE contact_id = $1")
            .bind(update.id)
            .execute(&mut *tx)
            .await?;
        for field in fields {
            sqlx::query(
                "INSERT INTO contact_disclosure_fields (contact_id, field) VALUES ($1, $2)",
            )
            .bind(update.id)
            .bind(field)
            .execute(&mut *tx)
            .await?;
        }
    }
    tx.commit().await?;
    Ok(true)
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
        .filter(|status| !matches!(status, crate::domain::contact::ContactStatus::Ok))
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
    status.as_str()
}

#[cfg(test)]
mod tests {
    use chrono::Utc;

    use super::*;

    async fn insert_test_contact(pool: &PgPool) -> (Uuid, Uuid) {
        let registrar_id = Uuid::new_v4();
        let contact_id = Uuid::new_v4();
        let now = Utc::now();
        sqlx::query(
            "INSERT INTO registrars (id, handle, name, client_id, password_hash, status, created_at, updated_at) VALUES ($1, 'REG-1', 'Registrar', 'client-1', 'not-used', 'active', $2, $2)",
        )
        .bind(registrar_id)
        .bind(now)
        .execute(pool)
        .await
        .unwrap();
        sqlx::query(
            "INSERT INTO contacts (id, roid, sponsoring_registrar_id, created_by, created_at, updated_by, updated_at, auth_info_ciphertext, disclose_flag) VALUES ($1, 'SH8013-EXAMPLE', $2, $2, $3, $2, $3, 'old-ciphertext', 'private')",
        )
        .bind(contact_id)
        .bind(registrar_id)
        .bind(now)
        .execute(pool)
        .await
        .unwrap();
        sqlx::query(
            "INSERT INTO contact_postal_info (contact_id, info_type, name, city, country_code) VALUES ($1, 'international', 'Jane Doe', 'Moscow', 'RU')",
        )
        .bind(contact_id)
        .execute(pool)
        .await
        .unwrap();
        sqlx::query(
            "INSERT INTO contact_phones (contact_id, voice, email) VALUES ($1, '+7.4951234567', 'old@example.test')",
        )
        .bind(contact_id)
        .execute(pool)
        .await
        .unwrap();
        sqlx::query(
            "INSERT INTO contact_disclosure_fields (contact_id, field) VALUES ($1, 'email')",
        )
        .bind(contact_id)
        .execute(pool)
        .await
        .unwrap();
        (registrar_id, contact_id)
    }

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

    #[ignore = "requires PostgreSQL; run through just test-with-db"]
    #[sqlx::test(migrations = "../backend/migrations")]
    async fn update_is_atomic_when_a_late_statement_fails(pool: PgPool) {
        let (registrar_id, contact_id) = insert_test_contact(&pool).await;
        let streets = ["New street"];
        let statuses = ["clientUpdateProhibited"];
        let fields = ["invalid"];

        let error = apply_update(
            &pool,
            ContactUpdate {
                id: contact_id,
                updated_by: registrar_id,
                auth_info_ciphertext: Some("new-ciphertext"),
                email: Some("new@example.test"),
                voice: None,
                fax: None,
                organization: None,
                city: None,
                state_province: None,
                postal_code: None,
                country_code: None,
                streets: &streets,
                add_statuses: &statuses,
                remove_statuses: &[],
                disclose_flag: Some("public"),
                disclosure_fields: Some(&fields),
            },
        )
        .await
        .unwrap_err();
        assert!(error.as_database_error().is_some());

        let auth_info: String =
            sqlx::query_scalar("SELECT auth_info_ciphertext FROM contacts WHERE id = $1")
                .bind(contact_id)
                .fetch_one(&pool)
                .await
                .unwrap();
        let email: String =
            sqlx::query_scalar("SELECT email FROM contact_phones WHERE contact_id = $1")
                .bind(contact_id)
                .fetch_one(&pool)
                .await
                .unwrap();
        let fields: Vec<String> = sqlx::query_scalar(
            "SELECT field FROM contact_disclosure_fields WHERE contact_id = $1 ORDER BY field",
        )
        .bind(contact_id)
        .fetch_all(&pool)
        .await
        .unwrap();
        let status_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM contact_statuses WHERE contact_id = $1 AND status = 'clientUpdateProhibited'",
        )
        .bind(contact_id)
        .fetch_one(&pool)
        .await
        .unwrap();

        assert_eq!(auth_info, "old-ciphertext");
        assert_eq!(email, "old@example.test");
        assert_eq!(fields, ["email"]);
        assert_eq!(status_count, 0);
    }

    #[ignore = "requires PostgreSQL; run through just test-with-db"]
    #[sqlx::test(migrations = "../backend/migrations")]
    async fn update_clears_optional_contact_fields(pool: PgPool) {
        let (registrar_id, contact_id) = insert_test_contact(&pool).await;
        sqlx::query("UPDATE contact_phones SET fax = '+7.4950000000' WHERE contact_id = $1")
            .bind(contact_id)
            .execute(&pool)
            .await
            .unwrap();
        sqlx::query(
            "UPDATE contact_postal_info SET organization = 'Example', state_province = 'Moscow', postal_code = '101000' WHERE contact_id = $1 AND info_type = 'international'",
        )
        .bind(contact_id)
        .execute(&pool)
        .await
        .unwrap();

        apply_update(
            &pool,
            ContactUpdate {
                id: contact_id,
                updated_by: registrar_id,
                auth_info_ciphertext: None,
                email: None,
                voice: None,
                fax: Some(None),
                organization: Some(None),
                city: None,
                state_province: Some(None),
                postal_code: Some(None),
                country_code: None,
                streets: &[],
                add_statuses: &[],
                remove_statuses: &[],
                disclose_flag: None,
                disclosure_fields: None,
            },
        )
        .await
        .unwrap();

        let fax: Option<String> =
            sqlx::query_scalar("SELECT fax FROM contact_phones WHERE contact_id = $1")
                .bind(contact_id)
                .fetch_one(&pool)
                .await
                .unwrap();
        let row: (Option<String>, Option<String>, Option<String>) = sqlx::query_as(
            "SELECT organization, state_province, postal_code FROM contact_postal_info WHERE contact_id = $1 AND info_type = 'international'",
        )
        .bind(contact_id)
        .fetch_one(&pool)
        .await
        .unwrap();

        assert_eq!(fax, None);
        assert_eq!(row, (None, None, None));
    }

    #[ignore = "requires PostgreSQL; run through just test-with-db"]
    #[sqlx::test(migrations = "../backend/migrations")]
    async fn reads_localized_postal_info(pool: PgPool) {
        let (_, contact_id) = insert_test_contact(&pool).await;
        sqlx::query(
            "INSERT INTO contact_postal_info (contact_id, info_type, name, organization, city, country_code) VALUES ($1, 'localized', 'Локальное имя', 'Компания', 'Москва', 'RU')",
        )
        .bind(contact_id)
        .execute(&pool)
        .await
        .unwrap();
        sqlx::query(
            "INSERT INTO contact_postal_streets (contact_id, info_type, position, street) VALUES ($1, 'localized', 1, 'Улица 1')",
        )
        .bind(contact_id)
        .execute(&pool)
        .await
        .unwrap();

        let contact = find_detail(&pool, contact_id).await.unwrap().unwrap();
        assert_eq!(contact.localized_name.as_deref(), Some("Локальное имя"));
        assert_eq!(contact.localized_organization.as_deref(), Some("Компания"));
        assert_eq!(contact.localized_streets, ["Улица 1"]);
        assert_eq!(contact.localized_city.as_deref(), Some("Москва"));
    }
}
