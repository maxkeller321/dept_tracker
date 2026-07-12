use argon2::password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString};
use argon2::Argon2;
use chrono::{Duration, Utc};
use sqlx::SqlitePool;
use uuid::Uuid;

pub const SESSION_COOKIE: &str = "dept_tracker_session";

pub async fn bootstrap_auth(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    let row: Option<(Option<String>,)> =
        sqlx::query_as("SELECT auth_password_hash FROM settings WHERE id = 1")
            .fetch_optional(pool)
            .await?;
    let existing = row.and_then(|(h,)| h);
    if existing.as_deref().unwrap_or("").is_empty() {
        if let (Ok(user), Ok(pass)) = (
            std::env::var("AUTH_USERNAME"),
            std::env::var("AUTH_PASSWORD"),
        ) {
            if !user.is_empty() && !pass.is_empty() {
                set_credentials(pool, &user, &pass).await?;
            }
        }
    }
    Ok(())
}

pub async fn auth_enabled(pool: &SqlitePool) -> Result<bool, sqlx::Error> {
    let row: Option<(Option<String>,)> =
        sqlx::query_as("SELECT auth_password_hash FROM settings WHERE id = 1")
            .fetch_optional(pool)
            .await?;
    Ok(row
        .and_then(|(h,)| h)
        .map(|s| !s.is_empty())
        .unwrap_or(false))
}

pub async fn set_credentials(
    pool: &SqlitePool,
    username: &str,
    password: &str,
) -> Result<(), sqlx::Error> {
    let hash = hash_password(password)?;
    sqlx::query(
        "UPDATE settings SET auth_username = ?, auth_password_hash = ? WHERE id = 1",
    )
    .bind(username)
    .bind(hash)
    .execute(pool)
    .await?;
    Ok(())
}

/// Create the first (and only) local account. Fails if credentials already exist.
pub async fn register(
    pool: &SqlitePool,
    username: &str,
    password: &str,
) -> Result<String, &'static str> {
    if auth_enabled(pool).await.map_err(|_| "internal error")? {
        return Err("account already exists");
    }
    let user = username.trim();
    if user.len() < 2 {
        return Err("username too short");
    }
    if user.len() > 64 {
        return Err("username too long");
    }
    if password.len() < 6 {
        return Err("password too short");
    }
    set_credentials(pool, user, password)
        .await
        .map_err(|_| "internal error")?;
    login(pool, user, password).await
}

pub async fn login(
    pool: &SqlitePool,
    username: &str,
    password: &str,
) -> Result<String, &'static str> {
    let row: Option<(Option<String>, Option<String>)> = sqlx::query_as(
        "SELECT auth_username, auth_password_hash FROM settings WHERE id = 1",
    )
    .fetch_optional(pool)
    .await
    .map_err(|_| "internal error")?;

    let Some((Some(stored_user), Some(hash))) = row else {
        return Err("auth not configured");
    };
    if stored_user != username {
        return Err("invalid credentials");
    }
    if !verify_password(password, &hash) {
        return Err("invalid credentials");
    }

    let token = Uuid::new_v4().to_string();
    let expires = (Utc::now() + Duration::days(30)).to_rfc3339();
    sqlx::query(
        "UPDATE settings SET auth_session_token = ?, auth_session_expires = ? WHERE id = 1",
    )
    .bind(&token)
    .bind(&expires)
    .execute(pool)
    .await
    .map_err(|_| "internal error")?;
    Ok(token)
}

pub async fn logout(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE settings SET auth_session_token = NULL, auth_session_expires = NULL WHERE id = 1",
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn session_valid(pool: &SqlitePool, token: &str) -> Result<bool, sqlx::Error> {
    if token.is_empty() {
        return Ok(false);
    }
    let row: Option<(Option<String>, Option<String>)> = sqlx::query_as(
        "SELECT auth_session_token, auth_session_expires FROM settings WHERE id = 1",
    )
    .fetch_optional(pool)
    .await?;

    let Some((Some(stored), Some(expires))) = row else {
        return Ok(false);
    };
    if stored != token {
        return Ok(false);
    }
    if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(&expires) {
        Ok(dt > Utc::now())
    } else {
        Ok(false)
    }
}

fn hash_password(password: &str) -> Result<String, sqlx::Error> {
    let salt = SaltString::generate(&mut argon2::password_hash::rand_core::OsRng);
    let argon2 = Argon2::default();
    let hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| sqlx::Error::Protocol(e.to_string().into()))?;
    Ok(hash.to_string())
}

fn verify_password(password: &str, hash: &str) -> bool {
    let parsed = PasswordHash::new(hash).ok();
    parsed
        .map(|h| Argon2::default().verify_password(password.as_bytes(), &h).is_ok())
        .unwrap_or(false)
}
