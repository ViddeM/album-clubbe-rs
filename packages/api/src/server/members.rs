//! Member-related server function implementations.

use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Argon2, PasswordHash, PasswordVerifier,
};
use dioxus::prelude::ServerFnError;
use rand::distributions::{Alphanumeric, DistString};
use sqlx::Row;

use crate::api_models::MemberEmail;

use super::{ensure_admin_token, get_db, IntoServerError};

pub async fn verify_member_password_internal(
    member_name: &str,
    password: &str,
) -> Result<(), ServerFnError> {
    let pool = get_db().await?;

    let row = sqlx::query("SELECT password_hash FROM members WHERE name = ?")
        .bind(member_name)
        .fetch_optional(pool)
        .await
        .server_err()?;

    let row = row.ok_or_else(|| ServerFnError::new("Unknown member"))?;
    let hash: Option<String> = row.get("password_hash");

    let hash = hash.ok_or_else(|| {
        ServerFnError::new("No password set for this member — ask an admin to generate one")
    })?;

    let parsed =
        PasswordHash::new(&hash).map_err(|_| ServerFnError::new("Stored hash is invalid"))?;

    Argon2::default()
        .verify_password(password.as_bytes(), &parsed)
        .map_err(|_| ServerFnError::new("Incorrect password"))
}

pub async fn admin_set_member_password_impl(
    admin_token: String,
    member_name: String,
) -> Result<String, ServerFnError> {
    ensure_admin_token(&admin_token)?;

    tracing::info!("POST /api/admin/member/set-password member=\"{member_name}\"");

    let plain: String = Alphanumeric.sample_string(&mut rand::thread_rng(), 16);

    let salt = SaltString::generate(&mut OsRng);
    let hash = Argon2::default()
        .hash_password(plain.as_bytes(), &salt)
        .map_err(|e| ServerFnError::new(format!("Failed to hash password: {e}")))?
        .to_string();

    let pool = get_db().await?;

    let rows_affected = sqlx::query("UPDATE members SET password_hash = ? WHERE name = ?")
        .bind(&hash)
        .bind(&member_name)
        .execute(pool)
        .await
        .server_err()?
        .rows_affected();

    if rows_affected == 0 {
        return Err(ServerFnError::new(format!(
            "Member \"{member_name}\" not found"
        )));
    }

    tracing::info!("POST /api/admin/member/set-password \"{member_name}\" → ok");
    Ok(plain)
}

pub async fn admin_delete_member_impl(
    admin_token: String,
    member_name: String,
) -> Result<(), ServerFnError> {
    ensure_admin_token(&admin_token)?;
    tracing::info!("POST /api/admin/member/delete member=\"{member_name}\"");

    let pool = get_db().await?;

    sqlx::query("UPDATE members SET deleted_at = datetime('now') WHERE name = ?")
        .bind(&member_name)
        .execute(pool)
        .await
        .server_err()?;

    tracing::info!("POST /api/admin/member/delete \"{member_name}\" → ok");
    Ok(())
}

fn validate_email(email: &str) -> Result<(), ServerFnError> {
    let valid = match email.split_once('@') {
        Some((local, domain)) => {
            !local.is_empty()
                && domain.contains('.')
                && !domain.contains('@')
                && !email
                    .chars()
                    .any(|c| c.is_whitespace() || c.is_control() || "<>,;\"".contains(c))
        }
        None => false,
    };
    if valid {
        Ok(())
    } else {
        Err(ServerFnError::new("Ogiltig e-postadress"))
    }
}

pub async fn admin_get_member_emails_impl(
    admin_token: String,
) -> Result<Vec<MemberEmail>, ServerFnError> {
    ensure_admin_token(&admin_token)?;
    tracing::debug!("POST /api/admin/member/emails");

    let pool = get_db().await?;
    let rows = sqlx::query(
        "SELECT name, email FROM members WHERE deleted_at IS NULL ORDER BY sort_order",
    )
    .fetch_all(pool)
    .await
    .server_err()?;

    Ok(rows
        .into_iter()
        .map(|row| MemberEmail {
            name: row.get("name"),
            email: row.get("email"),
        })
        .collect())
}

pub async fn admin_set_member_email_impl(
    admin_token: String,
    member_name: String,
    email: String,
) -> Result<(), ServerFnError> {
    ensure_admin_token(&admin_token)?;
    tracing::info!("POST /api/admin/member/set-email member=\"{member_name}\"");

    let email = email.trim();
    let email = if email.is_empty() {
        None
    } else {
        validate_email(email)?;
        Some(email)
    };

    let pool = get_db().await?;
    let rows_affected =
        sqlx::query("UPDATE members SET email = ? WHERE name = ? AND deleted_at IS NULL")
            .bind(email)
            .bind(&member_name)
            .execute(pool)
            .await
            .server_err()?
            .rows_affected();

    if rows_affected == 0 {
        return Err(ServerFnError::new(format!(
            "Member \"{member_name}\" not found"
        )));
    }

    tracing::info!("POST /api/admin/member/set-email \"{member_name}\" → ok");
    Ok(())
}
