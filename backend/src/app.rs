use std::sync::Arc;

use anyhow::{Context, Result};
use sqlx::{PgPool, migrate::Migrator, postgres::PgPoolOptions};

use crate::config::Settings;

static MIGRATOR: Migrator = sqlx::migrate!("../backend/migrations");

#[derive(Clone)]
pub(crate) struct AppState {
    pub db: PgPool,
    pub settings: Settings,
    pub extension_registry: Arc<crate::domain::extension::ExtensionRegistry>,
}

pub(crate) async fn build_state(settings: Settings) -> Result<Arc<AppState>> {
    let db = PgPoolOptions::new()
        .max_connections(10)
        .connect(&settings.database_url)
        .await
        .context("failed to connect to PostgreSQL")?;
    MIGRATOR
        .run(&db)
        .await
        .context("failed to run migrations")?;
    Ok(Arc::new(AppState {
        db,
        settings,
        extension_registry: Arc::new(crate::domain::extension::ExtensionRegistry::empty()),
    }))
}
