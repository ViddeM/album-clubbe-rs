use sqlx::{
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
    SqlitePool,
};
use std::str::FromStr;

const INITIAL_MEMBERS: &[&str] = &[
    "Swexbe", "Nox", "Karro", "Vidde", "Stasia", "Dino", "Yoda", "Carl", "Arvid",
];

pub async fn init_pool(db_url: &str) -> Result<SqlitePool, sqlx::Error> {
    let options = SqliteConnectOptions::from_str(db_url)?.create_if_missing(true);

    let pool = SqlitePoolOptions::new().connect_with(options).await?;

    run_migrations(&pool).await?;
    seed_members(&pool).await?;

    Ok(pool)
}

async fn run_migrations(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS members (
            name TEXT NOT NULL PRIMARY KEY
        )",
    )
    .execute(pool)
    .await?;

    // Single table for both the current entry and history.
    // is_current = 1 means this is the active entry.
    // The partial unique index below enforces that at most one row may have is_current = 1.
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS meetings (
            id                TEXT NOT NULL PRIMARY KEY,
            is_current        INTEGER NOT NULL DEFAULT 0,
            album_id          TEXT NOT NULL,
            album_name        TEXT NOT NULL,
            album_artist      TEXT NOT NULL,
            album_art_url     TEXT NOT NULL,
            album_spotify_url TEXT NOT NULL,
            picker            TEXT NOT NULL,
            meeting_date      TEXT,
            meeting_time      TEXT,
            meeting_location  TEXT,
            recorded_at       TEXT NOT NULL DEFAULT (datetime('now'))
        )",
    )
    .execute(pool)
    .await?;

    // Partial unique index: only one row may be the current entry.
    sqlx::query(
        "CREATE UNIQUE INDEX IF NOT EXISTS only_one_current ON meetings (is_current)
         WHERE is_current = 1",
    )
    .execute(pool)
    .await?;

    Ok(())
}

async fn seed_members(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM members")
        .fetch_one(pool)
        .await?;

    if count == 0 {
        for member in INITIAL_MEMBERS {
            sqlx::query("INSERT OR IGNORE INTO members (name) VALUES (?)")
                .bind(member)
                .execute(pool)
                .await?;
        }
    }

    Ok(())
}
