//! Meeting-related server function implementations.

use dioxus::prelude::ServerFnError;
use sqlx::Row;
use uuid::Uuid;

use crate::api_models::{Data, HistoryEntry, SetCurrentRequest};

use super::{ensure_admin_token, get_db, IntoServerError};

pub async fn get_current_impl() -> Result<Data, ServerFnError> {
    tracing::debug!("GET /api/info");

    let pool = get_db().await?;

    let members: Vec<String> =
        sqlx::query_scalar("SELECT name FROM members ORDER BY sort_order")
            .fetch_all(pool)
            .await
            .server_err()?;

    let row = sqlx::query(
        "SELECT id, album_id, album_name, album_artist, album_art_url, album_spotify_url,
                picker, meeting_date, meeting_time, meeting_location
         FROM meetings WHERE is_current = 1",
    )
    .fetch_optional(pool)
    .await
    .server_err()?;

    let member_names: Vec<crate::api_models::Name> =
        members.into_iter().map(|m| m.into()).collect();

    match row {
        None => Ok(Data {
            current_meeting_id: None,
            current_album: None,
            next_meeting: None,
            current_person: None,
            members: member_names,
        }),
        Some(row) => {
            let meeting_date: String = row.get("meeting_date");
            let meeting_time: Option<String> = row.get("meeting_time");
            let meeting_location: Option<String> = row.get("meeting_location");

            let next_meeting = Some(crate::api_models::Meeting {
                date: meeting_date,
                time: meeting_time,
                location: meeting_location,
            });

            Ok(Data {
                current_meeting_id: Some(row.get("id")),
                current_album: Some(crate::api_models::Album {
                    id: row.get("album_id"),
                    name: row.get("album_name"),
                    artist: row.get("album_artist"),
                    album_art: row.get("album_art_url"),
                    spotify_url: row.get("album_spotify_url"),
                }),
                next_meeting,
                current_person: Some(row.get::<String, _>("picker").into()),
                members: member_names,
            })
        }
    }
}

pub async fn get_history_impl() -> Result<Vec<HistoryEntry>, ServerFnError> {
    tracing::debug!("GET /api/history");

    let pool = get_db().await?;

    let rows = sqlx::query(
        "SELECT id, album_name, album_artist, album_art_url, album_spotify_url, picker, recorded_at,
                meeting_date, meeting_time, meeting_location
         FROM meetings
         WHERE is_current = 0
         ORDER BY meeting_date ASC",
    )
    .fetch_all(pool)
    .await
    .server_err()?;

    tracing::debug!("GET /api/history → {} entries", rows.len());
    Ok(rows
        .into_iter()
        .map(|row| HistoryEntry {
            id: row.get("id"),
            album_name: row.get("album_name"),
            album_artist: row.get("album_artist"),
            album_art: row.get("album_art_url"),
            spotify_url: row.get("album_spotify_url"),
            picker: row.get("picker"),
            recorded_at: row.get("recorded_at"),
            meeting_date: row.get("meeting_date"),
            meeting_time: row.get("meeting_time"),
            meeting_location: row.get("meeting_location"),
        })
        .collect())
}

pub async fn admin_set_current_impl(
    admin_token: String,
    req: SetCurrentRequest,
) -> Result<(), ServerFnError> {
    ensure_admin_token(&admin_token)?;
    tracing::info!(
        "POST /api/admin/set-current album=\"{}\" picker=\"{}\" date=\"{}\"",
        req.album_name,
        req.picker,
        req.meeting_date
    );

    let pool = get_db().await?;
    let mut tx = pool.begin().await.server_err()?;

    sqlx::query("UPDATE meetings SET is_current = 0 WHERE is_current = 1")
        .execute(&mut *tx)
        .await
        .server_err()?;

    sqlx::query(
        "INSERT INTO meetings
            (id, is_current, album_id, album_name, album_artist, album_art_url,
             album_spotify_url, picker, meeting_date, meeting_time, meeting_location)
         VALUES (?, 1, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(Uuid::new_v4().to_string())
    .bind(req.album_id)
    .bind(req.album_name)
    .bind(req.album_artist)
    .bind(req.album_art_url)
    .bind(req.album_spotify_url)
    .bind(req.picker)
    .bind(req.meeting_date)
    .bind(req.meeting_time)
    .bind(req.meeting_location)
    .execute(&mut *tx)
    .await
    .server_err()?;

    tx.commit().await.server_err()?;

    tracing::info!("POST /api/admin/set-current → ok");
    Ok(())
}

pub async fn admin_update_current_impl(
    admin_token: String,
    req: SetCurrentRequest,
) -> Result<(), ServerFnError> {
    ensure_admin_token(&admin_token)?;
    tracing::info!(
        "POST /api/admin/update-current album=\"{}\" picker=\"{}\" date=\"{}\"",
        req.album_name,
        req.picker,
        req.meeting_date
    );

    let pool = get_db().await?;

    sqlx::query(
        "UPDATE meetings
         SET album_id = ?, album_name = ?, album_artist = ?, album_art_url = ?,
             album_spotify_url = ?, picker = ?, meeting_date = ?, meeting_time = ?,
             meeting_location = ?
         WHERE is_current = 1",
    )
    .bind(req.album_id)
    .bind(req.album_name)
    .bind(req.album_artist)
    .bind(req.album_art_url)
    .bind(req.album_spotify_url)
    .bind(req.picker)
    .bind(req.meeting_date)
    .bind(req.meeting_time)
    .bind(req.meeting_location)
    .execute(pool)
    .await
    .server_err()?;

    tracing::info!("POST /api/admin/update-current → ok");
    Ok(())
}

pub async fn admin_delete_history_entry_impl(
    admin_token: String,
    id: String,
) -> Result<(), ServerFnError> {
    ensure_admin_token(&admin_token)?;
    tracing::info!("POST /api/admin/history/delete id=\"{id}\"");

    let pool = get_db().await?;

    sqlx::query("DELETE FROM meetings WHERE id = ? AND is_current = 0")
        .bind(&id)
        .execute(pool)
        .await
        .server_err()?;

    tracing::info!("POST /api/admin/history/delete id=\"{id}\" → ok");
    Ok(())
}

pub async fn admin_reorder_members_impl(
    admin_token: String,
    ordered_names: Vec<String>,
) -> Result<(), ServerFnError> {
    ensure_admin_token(&admin_token)?;
    tracing::info!(
        "POST /api/admin/reorder-members {} members",
        ordered_names.len()
    );

    let pool = get_db().await?;
    let mut tx = pool.begin().await.server_err()?;

    for (i, name) in ordered_names.iter().enumerate() {
        sqlx::query("UPDATE members SET sort_order = ? WHERE name = ?")
            .bind(i as i64)
            .bind(name)
            .execute(&mut *tx)
            .await
            .server_err()?;
    }

    tx.commit().await.server_err()?;

    tracing::info!("POST /api/admin/reorder-members → ok");
    Ok(())
}
