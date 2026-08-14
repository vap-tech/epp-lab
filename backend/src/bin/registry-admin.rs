use std::{
    env,
    io::{self, Write},
};

use anyhow::{Context, Result};
use argon2::{
    Argon2, PasswordHasher,
    password_hash::{SaltString, rand_core::OsRng},
};
use sqlx::postgres::PgPoolOptions;
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<()> {
    let mut args = env::args().skip(1);
    let command = args
        .next()
        .context("usage: registry-admin create-admin <username>")?;
    if command != "create-admin" {
        anyhow::bail!("unknown command: {command}");
    }
    let username = args
        .next()
        .context("usage: registry-admin create-admin <username>")?;
    if username.trim().is_empty() || args.next().is_some() {
        anyhow::bail!("usage: registry-admin create-admin <username>");
    }
    let password = read_password()?;
    if password.is_empty() {
        anyhow::bail!("password must not be empty");
    }
    let salt = SaltString::generate(&mut OsRng);
    let password_hash = Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map_err(|error| anyhow::anyhow!("failed to hash password: {error}"))?
        .to_string();
    let database_url = env::var("DATABASE_URL").context("DATABASE_URL must be set")?;
    let pool = PgPoolOptions::new()
        .max_connections(1)
        .connect(&database_url)
        .await?;
    sqlx::query("INSERT INTO admin_users (id, username, password_hash, status, created_at, updated_at) VALUES ($1,$2,$3,'active',NOW(),NOW())")
        .bind(Uuid::new_v4()).bind(username).bind(password_hash).execute(&pool).await
        .context("failed to create admin user")?;
    println!("admin user created");
    Ok(())
}

fn read_password() -> Result<String> {
    eprint!("Password: ");
    io::stderr().flush()?;
    let mut password = String::new();
    io::stdin().read_line(&mut password)?;
    Ok(password.trim_end_matches(['\r', '\n']).to_owned())
}
