use chrono::{DateTime, Utc};
use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

#[derive(Debug, sqlx::FromRow)]
pub(crate) struct DomainRow {
    pub id: Uuid,
    pub name: String,
    pub roid: String,
    pub zone_id: Uuid,
    pub sponsoring_registrar_id: Uuid,
    pub auth_info_ciphertext: String,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_by: Option<Uuid>,
    pub updated_at: Option<DateTime<Utc>>,
    pub expires_at: DateTime<Utc>,
    pub transferred_at: Option<DateTime<Utc>>,
}

#[derive(Debug, sqlx::FromRow)]
pub(crate) struct DomainContactRow {
    pub domain_id: Uuid,
    pub role: String,
    pub contact_id: Uuid,
    pub position: i16,
}

#[derive(Debug, sqlx::FromRow)]
pub(crate) struct DomainNameserverRow {
    pub domain_id: Uuid,
    pub position: i16,
    pub hostname: String,
}

#[derive(Debug, sqlx::FromRow)]
pub(crate) struct DomainStatusRow {
    pub domain_id: Uuid,
    pub status: String,
    pub source: String,
}

pub(crate) struct NewDomain<'a> {
    pub row: &'a DomainRow,
    pub contacts: &'a [DomainContactRow],
    pub nameservers: &'a [DomainNameserverRow],
    pub statuses: &'a [DomainStatusRow],
}

pub(crate) async fn exists_by_name(pool: &PgPool, name: &str) -> Result<bool, sqlx::Error> {
    sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM domains WHERE name = $1)")
        .bind(name)
        .fetch_one(pool)
        .await
}

pub(crate) async fn find(pool: &PgPool, id: Uuid) -> Result<Option<DomainRow>, sqlx::Error> {
    sqlx::query_as("SELECT id, name, roid, zone_id, sponsoring_registrar_id, auth_info_ciphertext, created_by, created_at, updated_by, updated_at, expires_at, transferred_at FROM domains WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await
}

pub(crate) async fn find_by_name(
    pool: &PgPool,
    name: &str,
) -> Result<Option<DomainRow>, sqlx::Error> {
    sqlx::query_as("SELECT id, name, roid, zone_id, sponsoring_registrar_id, auth_info_ciphertext, created_by, created_at, updated_by, updated_at, expires_at, transferred_at FROM domains WHERE name = $1")
        .bind(name)
        .fetch_optional(pool)
        .await
}

pub(crate) async fn list_contacts(
    pool: &PgPool,
    domain_id: Uuid,
) -> Result<Vec<DomainContactRow>, sqlx::Error> {
    sqlx::query_as("SELECT domain_id, role, contact_id, position FROM domain_contacts WHERE domain_id = $1 ORDER BY role, position")
        .bind(domain_id)
        .fetch_all(pool)
        .await
}

pub(crate) async fn list_nameservers(
    pool: &PgPool,
    domain_id: Uuid,
) -> Result<Vec<DomainNameserverRow>, sqlx::Error> {
    sqlx::query_as("SELECT domain_id, position, hostname FROM domain_nameservers WHERE domain_id = $1 ORDER BY position")
        .bind(domain_id)
        .fetch_all(pool)
        .await
}

pub(crate) async fn list_statuses(
    pool: &PgPool,
    domain_id: Uuid,
) -> Result<Vec<DomainStatusRow>, sqlx::Error> {
    sqlx::query_as("SELECT domain_id, status, source FROM domain_statuses WHERE domain_id = $1 ORDER BY source, status")
        .bind(domain_id)
        .fetch_all(pool)
        .await
}

pub(crate) async fn create(pool: &PgPool, new_domain: NewDomain<'_>) -> Result<(), sqlx::Error> {
    let mut tx = pool.begin().await?;
    insert_domain(&mut tx, new_domain.row).await?;
    insert_contacts(&mut tx, new_domain.contacts).await?;
    insert_nameservers(&mut tx, new_domain.nameservers).await?;
    insert_statuses(&mut tx, new_domain.statuses).await?;
    tx.commit().await
}

pub(crate) async fn delete(pool: &PgPool, id: Uuid) -> Result<bool, sqlx::Error> {
    let result = sqlx::query("DELETE FROM domains WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected() == 1)
}

pub(crate) async fn has_contact_links(
    pool: &PgPool,
    contact_id: Uuid,
) -> Result<bool, sqlx::Error> {
    sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM domain_contacts WHERE contact_id = $1)")
        .bind(contact_id)
        .fetch_one(pool)
        .await
}

async fn insert_domain(
    tx: &mut Transaction<'_, Postgres>,
    row: &DomainRow,
) -> Result<(), sqlx::Error> {
    sqlx::query("INSERT INTO domains (id, name, roid, zone_id, sponsoring_registrar_id, auth_info_ciphertext, created_by, created_at, updated_by, updated_at, expires_at, transferred_at) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12)")
        .bind(row.id)
        .bind(&row.name)
        .bind(&row.roid)
        .bind(row.zone_id)
        .bind(row.sponsoring_registrar_id)
        .bind(&row.auth_info_ciphertext)
        .bind(row.created_by)
        .bind(row.created_at)
        .bind(row.updated_by)
        .bind(row.updated_at)
        .bind(row.expires_at)
        .bind(row.transferred_at)
        .execute(&mut **tx)
        .await
        .map(|_| ())
}

async fn insert_contacts(
    tx: &mut Transaction<'_, Postgres>,
    rows: &[DomainContactRow],
) -> Result<(), sqlx::Error> {
    for row in rows {
        sqlx::query("INSERT INTO domain_contacts (domain_id, role, contact_id, position) VALUES ($1,$2,$3,$4)")
            .bind(row.domain_id)
            .bind(&row.role)
            .bind(row.contact_id)
            .bind(row.position)
            .execute(&mut **tx)
            .await?;
    }
    Ok(())
}

async fn insert_nameservers(
    tx: &mut Transaction<'_, Postgres>,
    rows: &[DomainNameserverRow],
) -> Result<(), sqlx::Error> {
    for row in rows {
        sqlx::query(
            "INSERT INTO domain_nameservers (domain_id, position, hostname) VALUES ($1,$2,$3)",
        )
        .bind(row.domain_id)
        .bind(row.position)
        .bind(&row.hostname)
        .execute(&mut **tx)
        .await?;
    }
    Ok(())
}

async fn insert_statuses(
    tx: &mut Transaction<'_, Postgres>,
    rows: &[DomainStatusRow],
) -> Result<(), sqlx::Error> {
    for row in rows {
        sqlx::query("INSERT INTO domain_statuses (domain_id, status, source) VALUES ($1,$2,$3)")
            .bind(row.domain_id)
            .bind(&row.status)
            .bind(&row.source)
            .execute(&mut **tx)
            .await?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use chrono::{Duration, Utc};
    use sqlx::PgPool;
    use uuid::Uuid;

    use super::*;

    async fn insert_registrar(pool: &PgPool, id: Uuid, suffix: &str) {
        let now = Utc::now();
        sqlx::query("INSERT INTO registrars (id, handle, name, client_id, password_hash, status, created_at, updated_at) VALUES ($1, $2, $3, $4, 'not-used', 'active', $5, $5)")
            .bind(id)
            .bind(format!("REG-{suffix}"))
            .bind("Registrar")
            .bind(format!("client-{suffix}"))
            .bind(now)
            .execute(pool)
            .await
            .unwrap();
    }

    async fn insert_zone(pool: &PgPool, id: Uuid, registrar_id: Uuid, name: &str) {
        let now = Utc::now();
        sqlx::query("INSERT INTO zones (id, ascii_name, unicode_name, status, created_at, updated_at) VALUES ($1, $2, $2, 'active', $3, $3)")
            .bind(id)
            .bind(name)
            .bind(now)
            .execute(pool)
            .await
            .unwrap();
        sqlx::query("INSERT INTO zone_contact_policies (zone_id, registrant_requirement, admin_requirement, tech_requirement, billing_requirement, updated_at) VALUES ($1, 'required', 'optional', 'optional', 'optional', $2)")
            .bind(id)
            .bind(now)
            .execute(pool)
            .await
            .unwrap();
        let _ = registrar_id;
    }

    #[ignore = "requires PostgreSQL; run through just test-with-db"]
    #[sqlx::test(migrations = "../backend/migrations")]
    async fn domain_round_trip_preserves_children_and_order(pool: PgPool) {
        let registrar_id = Uuid::new_v4();
        let zone_id = Uuid::new_v4();
        let domain_id = Uuid::new_v4();
        insert_registrar(&pool, registrar_id, "domain-roundtrip").await;
        insert_zone(&pool, zone_id, registrar_id, "com").await;

        let now = Utc::now();
        let row = DomainRow {
            id: domain_id,
            name: "example.com".into(),
            roid: "D123-EXAMPLE".into(),
            zone_id,
            sponsoring_registrar_id: registrar_id,
            auth_info_ciphertext: "domain-ciphertext".into(),
            created_by: registrar_id,
            created_at: now,
            updated_by: None,
            updated_at: None,
            expires_at: now + Duration::days(365),
            transferred_at: None,
        };
        let contacts = [DomainContactRow {
            domain_id,
            role: "registrant".into(),
            contact_id: Uuid::new_v4(),
            position: 1,
        }];
        let nameservers = [
            DomainNameserverRow {
                domain_id,
                position: 1,
                hostname: "ns1.example.net".into(),
            },
            DomainNameserverRow {
                domain_id,
                position: 2,
                hostname: "ns2.example.net".into(),
            },
        ];
        let statuses = [DomainStatusRow {
            domain_id,
            status: "clientHold".into(),
            source: "client".into(),
        }];

        // The contact must exist before the association can be inserted.
        let contact_id = contacts[0].contact_id;
        sqlx::query("INSERT INTO contacts (id, roid, sponsoring_registrar_id, created_by, created_at, updated_by, updated_at, auth_info_ciphertext, disclose_flag) VALUES ($1, $2, $3, $3, $4, $3, $4, 'contact-ciphertext', 'private')")
            .bind(contact_id)
            .bind("C-DOMAIN-1")
            .bind(registrar_id)
            .bind(now)
            .execute(&pool)
            .await
            .unwrap();

        create(
            &pool,
            NewDomain {
                row: &row,
                contacts: &contacts,
                nameservers: &nameservers,
                statuses: &statuses,
            },
        )
        .await
        .unwrap();
        let stored = find(&pool, domain_id).await.unwrap().unwrap();
        assert_eq!(stored.name, row.name);
        assert_eq!(stored.auth_info_ciphertext, row.auth_info_ciphertext);
        assert_eq!(
            list_nameservers(&pool, domain_id)
                .await
                .unwrap()
                .into_iter()
                .map(|ns| ns.hostname)
                .collect::<Vec<_>>(),
            ["ns1.example.net", "ns2.example.net"]
        );
        assert_eq!(list_contacts(&pool, domain_id).await.unwrap().len(), 1);
        assert_eq!(list_statuses(&pool, domain_id).await.unwrap().len(), 1);
    }

    #[ignore = "requires PostgreSQL; run through just test-with-db"]
    #[sqlx::test(migrations = "../backend/migrations")]
    async fn domain_delete_cascades_owned_rows_but_contact_link_blocks_contact_delete(
        pool: PgPool,
    ) {
        let registrar_id = Uuid::new_v4();
        let zone_id = Uuid::new_v4();
        let domain_id = Uuid::new_v4();
        let contact_id = Uuid::new_v4();
        insert_registrar(&pool, registrar_id, "domain-delete").await;
        insert_zone(&pool, zone_id, registrar_id, "com").await;
        let now = Utc::now();
        sqlx::query("INSERT INTO contacts (id, roid, sponsoring_registrar_id, created_by, created_at, updated_by, updated_at, auth_info_ciphertext, disclose_flag) VALUES ($1, 'C-DOMAIN-2', $2, $2, $3, $2, $3, 'contact-ciphertext', 'private')")
            .bind(contact_id)
            .bind(registrar_id)
            .bind(now)
            .execute(&pool)
            .await
            .unwrap();
        let row = DomainRow {
            id: domain_id,
            name: "delete.example.com".into(),
            roid: "D123-DELETE".into(),
            zone_id,
            sponsoring_registrar_id: registrar_id,
            auth_info_ciphertext: "ciphertext".into(),
            created_by: registrar_id,
            created_at: now,
            updated_by: None,
            updated_at: None,
            expires_at: now + Duration::days(365),
            transferred_at: None,
        };
        let contacts = [DomainContactRow {
            domain_id,
            role: "registrant".into(),
            contact_id,
            position: 1,
        }];
        create(
            &pool,
            NewDomain {
                row: &row,
                contacts: &contacts,
                nameservers: &[],
                statuses: &[],
            },
        )
        .await
        .unwrap();
        assert!(has_contact_links(&pool, contact_id).await.unwrap());
        assert!(
            sqlx::query("DELETE FROM contacts WHERE id = $1")
                .bind(contact_id)
                .execute(&pool)
                .await
                .is_err()
        );
        assert!(delete(&pool, domain_id).await.unwrap());
        assert!(!has_contact_links(&pool, contact_id).await.unwrap());
        assert!(find(&pool, domain_id).await.unwrap().is_none());
    }
}
