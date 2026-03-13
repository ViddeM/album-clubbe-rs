//! Review-related server function implementations.

use dioxus::prelude::ServerFnError;
use sqlx::Row;
use uuid::Uuid;

use crate::api_models::{AlbumReview, Reviews, TrackReview};

use super::{get_db, members::verify_member_password_internal, IntoServerError};

pub async fn get_reviews_impl(meeting_id: String) -> Result<Reviews, ServerFnError> {
    tracing::debug!("get_reviews meeting_id=\"{meeting_id}\"");
    let pool = get_db().await?;

    let album_rows =
        sqlx::query("SELECT member_name, score FROM album_reviews WHERE meeting_id = ?")
            .bind(&meeting_id)
            .fetch_all(pool)
            .await
            .server_err()?;

    let album_reviews = album_rows
        .into_iter()
        .map(|r| AlbumReview {
            member_name: r.get("member_name"),
            score: r.get::<i64, _>("score") as u8,
        })
        .collect();

    let track_rows = sqlx::query(
        "SELECT member_name, track_id, score FROM track_reviews WHERE meeting_id = ?",
    )
    .bind(&meeting_id)
    .fetch_all(pool)
    .await
    .server_err()?;

    let track_reviews = track_rows
        .into_iter()
        .map(|r| TrackReview {
            member_name: r.get("member_name"),
            track_id: r.get("track_id"),
            score: r.get::<i64, _>("score") as u8,
        })
        .collect();

    Ok(Reviews {
        album_reviews,
        track_reviews,
    })
}

pub async fn submit_album_review_impl(
    member_name: String,
    password: String,
    meeting_id: String,
    score: u8,
) -> Result<Reviews, ServerFnError> {
    tracing::info!(
        "submit_album_review member=\"{member_name}\" meeting=\"{meeting_id}\" score={score}"
    );
    verify_member_password_internal(&member_name, &password).await?;

    let pool = get_db().await?;

    sqlx::query(
        "INSERT INTO album_reviews (id, meeting_id, member_name, score)
         VALUES (?, ?, ?, ?)
         ON CONFLICT(meeting_id, member_name)
         DO UPDATE SET score = excluded.score, updated_at = datetime('now')",
    )
    .bind(Uuid::new_v4().to_string())
    .bind(&meeting_id)
    .bind(&member_name)
    .bind(score as i64)
    .execute(pool)
    .await
    .server_err()?;

    tracing::info!("submit_album_review → ok");
    get_reviews_impl(meeting_id).await
}

pub async fn submit_track_review_impl(
    member_name: String,
    password: String,
    meeting_id: String,
    track_id: String,
    score: u8,
) -> Result<Reviews, ServerFnError> {
    tracing::info!("Reviewing track by {member_name} :: {track_id} :: {score}");
    tracing::info!("submit_track_review member=\"{member_name}\" track=\"{track_id}\" meeting=\"{meeting_id}\" score={score}");
    verify_member_password_internal(&member_name, &password).await?;

    let pool = get_db().await?;

    sqlx::query(
        "INSERT INTO track_reviews (id, meeting_id, member_name, track_id, score)
         VALUES (?, ?, ?, ?, ?)
         ON CONFLICT(meeting_id, member_name, track_id)
         DO UPDATE SET score = excluded.score, updated_at = datetime('now')",
    )
    .bind(Uuid::new_v4().to_string())
    .bind(&meeting_id)
    .bind(&member_name)
    .bind(&track_id)
    .bind(score as i64)
    .execute(pool)
    .await
    .server_err()?;

    tracing::info!("submit_track_review → ok");
    get_reviews_impl(meeting_id).await
}
