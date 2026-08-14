use chrono::{DateTime, Utc};
use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

use crate::domain::zone::{
    ContactRequirement, ContactUsagePolicy, Zone, ZoneId, ZoneName, ZoneStatus,
};

#[allow(dead_code)]
#[derive(Debug, sqlx::FromRow)]
pub(crate) struct ZoneRow {
    pub id: Uuid,
    pub ascii_name: String,
    pub unicode_name: Option<String>,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub enabled_extensions_count: i64,
    pub registrant_requirement: String,
    pub admin_requirement: String,
    pub tech_requirement: String,
    pub billing_requirement: String,
}

#[allow(dead_code)]
pub(crate) async fn list(pool: &PgPool) -> Result<Vec<ZoneRow>, sqlx::Error> {
    sqlx::query_as(
        r#"
        SELECT z.id, z.ascii_name, z.unicode_name, z.status,
               p.registrant_requirement, p.admin_requirement,
               p.tech_requirement, p.billing_requirement,
               z.created_at, z.updated_at,
               (SELECT COUNT(*) FROM zone_extensions e
                WHERE e.zone_id = z.id AND e.enabled) AS enabled_extensions_count
        FROM zones z
        JOIN zone_contact_policies p ON p.zone_id = z.id
        ORDER BY z.ascii_name
        "#,
    )
    .fetch_all(pool)
    .await
}

#[allow(dead_code)]
pub(crate) async fn find(pool: &PgPool, id: Uuid) -> Result<Option<ZoneRow>, sqlx::Error> {
    sqlx::query_as(
        r#"
        SELECT z.id, z.ascii_name, z.unicode_name, z.status,
               p.registrant_requirement, p.admin_requirement,
               p.tech_requirement, p.billing_requirement,
               z.created_at, z.updated_at,
               (SELECT COUNT(*) FROM zone_extensions e
                WHERE e.zone_id = z.id AND e.enabled) AS enabled_extensions_count
        FROM zones z
        JOIN zone_contact_policies p ON p.zone_id = z.id
        WHERE z.id = $1
        "#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await
}

#[allow(dead_code)]
pub(crate) async fn update_status(
    pool: &PgPool,
    id: Uuid,
    status: &str,
    now: DateTime<Utc>,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query("UPDATE zones SET status = $2, updated_at = $3 WHERE id = $1")
        .bind(id)
        .bind(status)
        .bind(now)
        .execute(pool)
        .await?;
    Ok(result.rows_affected() == 1)
}

#[allow(dead_code)]
pub(crate) async fn update_contact_policy(
    pool: &PgPool,
    zone_id: Uuid,
    policy: ContactUsagePolicy,
    now: DateTime<Utc>,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query(
        "UPDATE zone_contact_policies SET registrant_requirement = $2, admin_requirement = $3, tech_requirement = $4, billing_requirement = $5, updated_at = $6 WHERE zone_id = $1",
    )
    .bind(zone_id)
    .bind(requirement_value(policy.registrant))
    .bind(requirement_value(policy.admin))
    .bind(requirement_value(policy.tech))
    .bind(requirement_value(policy.billing))
    .bind(now)
    .execute(pool)
    .await?;
    Ok(result.rows_affected() == 1)
}

#[allow(dead_code)]
pub(crate) async fn create(
    pool: &PgPool,
    zone: &Zone,
    now: DateTime<Utc>,
) -> Result<(), sqlx::Error> {
    let mut transaction = pool.begin().await?;
    insert_zone(&mut transaction, zone, now).await?;
    insert_contact_policy(&mut transaction, zone, now).await?;
    transaction.commit().await
}

#[allow(dead_code)]
pub(crate) async fn list_extensions(
    pool: &PgPool,
    zone_id: Uuid,
) -> Result<Vec<ZoneExtensionRow>, sqlx::Error> {
    sqlx::query_as(
        "SELECT zone_id, extension_key, enabled FROM zone_extensions WHERE zone_id = $1 ORDER BY extension_key",
    )
    .bind(zone_id)
    .fetch_all(pool)
    .await
}

#[allow(dead_code)]
pub(crate) async fn set_extension(
    pool: &PgPool,
    zone_id: Uuid,
    extension_key: &str,
    enabled: bool,
    now: DateTime<Utc>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO zone_extensions (zone_id, extension_key, enabled, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $4)
        ON CONFLICT (zone_id, extension_key)
        DO UPDATE SET enabled = EXCLUDED.enabled, updated_at = EXCLUDED.updated_at
        "#,
    )
    .bind(zone_id)
    .bind(extension_key)
    .bind(enabled)
    .bind(now)
    .execute(pool)
    .await
    .map(|_| ())
}

#[allow(dead_code)]
#[derive(Debug, sqlx::FromRow)]
pub(crate) struct ZoneExtensionRow {
    pub zone_id: Uuid,
    pub extension_key: String,
    pub enabled: bool,
}

async fn insert_zone(
    transaction: &mut Transaction<'_, Postgres>,
    zone: &Zone,
    now: DateTime<Utc>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO zones (id, ascii_name, unicode_name, status, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $5)",
    )
    .bind(zone.id.into_uuid())
    .bind(zone.name.ascii())
    .bind(zone.name.unicode())
    .bind(status_value(zone.status))
    .bind(now)
    .execute(&mut **transaction)
    .await
    .map(|_| ())
}

async fn insert_contact_policy(
    transaction: &mut Transaction<'_, Postgres>,
    zone: &Zone,
    now: DateTime<Utc>,
) -> Result<(), sqlx::Error> {
    let policy = zone.contact_policy;
    sqlx::query(
        "INSERT INTO zone_contact_policies (zone_id, registrant_requirement, admin_requirement, tech_requirement, billing_requirement, updated_at) VALUES ($1, $2, $3, $4, $5, $6)",
    )
    .bind(zone.id.into_uuid())
    .bind(requirement_value(policy.registrant))
    .bind(requirement_value(policy.admin))
    .bind(requirement_value(policy.tech))
    .bind(requirement_value(policy.billing))
    .bind(now)
    .execute(&mut **transaction)
    .await
    .map(|_| ())
}

#[allow(dead_code)]
pub(crate) fn to_domain(row: ZoneRow) -> Result<Zone, String> {
    let name = ZoneName::parse(&row.ascii_name).map_err(|error| error.to_string())?;
    Ok(Zone {
        id: ZoneId::new(row.id),
        name,
        status: parse_status(&row.status)?,
        contact_policy: ContactUsagePolicy {
            registrant: parse_requirement(&row.registrant_requirement)?,
            admin: parse_requirement(&row.admin_requirement)?,
            tech: parse_requirement(&row.tech_requirement)?,
            billing: parse_requirement(&row.billing_requirement)?,
        },
    })
}

fn status_value(status: ZoneStatus) -> &'static str {
    match status {
        ZoneStatus::Active => "active",
        ZoneStatus::Disabled => "disabled",
    }
}

fn requirement_value(requirement: ContactRequirement) -> &'static str {
    match requirement {
        ContactRequirement::Forbidden => "forbidden",
        ContactRequirement::Optional => "optional",
        ContactRequirement::Required => "required",
    }
}

fn parse_status(value: &str) -> Result<ZoneStatus, String> {
    match value {
        "active" => Ok(ZoneStatus::Active),
        "disabled" => Ok(ZoneStatus::Disabled),
        _ => Err(format!("unknown zone status: {value}")),
    }
}

fn parse_requirement(value: &str) -> Result<ContactRequirement, String> {
    match value {
        "forbidden" => Ok(ContactRequirement::Forbidden),
        "optional" => Ok(ContactRequirement::Optional),
        "required" => Ok(ContactRequirement::Required),
        _ => Err(format!("unknown contact requirement: {value}")),
    }
}
