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
               p.tech_requirement, p.billing_requirement
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
               p.tech_requirement, p.billing_requirement
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
