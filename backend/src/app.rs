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
    #[allow(dead_code)]
    pub contact_authinfo_cipher: Option<Arc<dyn crate::security::SecretCipher>>,
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
    let contact_authinfo_cipher = settings
        .contact_authinfo_key_hex
        .as_deref()
        .map(crate::security::AesGcmSecretCipher::from_hex)
        .transpose()
        .context("invalid CONTACT_AUTHINFO_KEY_HEX")?
        .map(|cipher| Arc::new(cipher) as Arc<dyn crate::security::SecretCipher>);
    Ok(Arc::new(AppState {
        db,
        settings,
        extension_registry: Arc::new(crate::domain::extension::ExtensionRegistry::empty()),
        contact_authinfo_cipher,
    }))
}
