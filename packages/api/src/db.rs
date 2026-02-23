use sqlx::{
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
    Row, SqlitePool,
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
    // Track applied migrations.
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS schema_migrations (
            name       TEXT NOT NULL PRIMARY KEY,
            applied_at TEXT NOT NULL DEFAULT (datetime('now'))
        )",
    )
    .execute(pool)
    .await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS members (
            name       TEXT    NOT NULL PRIMARY KEY,
            sort_order INTEGER NOT NULL DEFAULT 0
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
            meeting_date      TEXT NOT NULL,
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

    // Migration: make meeting_date NOT NULL by rebuilding the existing table if needed.
    let migrated: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM schema_migrations WHERE name = 'meeting_date_not_null'",
    )
    .fetch_one(pool)
    .await?;

    if migrated == 0 {
        let pragma_rows = sqlx::query("PRAGMA table_info(meetings)")
            .fetch_all(pool)
            .await?;

        let needs_rebuild = pragma_rows.iter().any(|r| {
            r.try_get::<&str, _>("name").unwrap_or("") == "meeting_date"
                && r.try_get::<i64, _>("notnull").unwrap_or(0) == 0
        });

        if needs_rebuild {
            sqlx::query(
                "CREATE TABLE meetings_new (
                    id                TEXT NOT NULL PRIMARY KEY,
                    is_current        INTEGER NOT NULL DEFAULT 0,
                    album_id          TEXT NOT NULL,
                    album_name        TEXT NOT NULL,
                    album_artist      TEXT NOT NULL,
                    album_art_url     TEXT NOT NULL,
                    album_spotify_url TEXT NOT NULL,
                    picker            TEXT NOT NULL,
                    meeting_date      TEXT NOT NULL,
                    meeting_time      TEXT,
                    meeting_location  TEXT,
                    recorded_at       TEXT NOT NULL DEFAULT (datetime('now'))
                )",
            )
            .execute(pool)
            .await?;

            // Backfill NULL meeting_dates with recorded_at so NOT NULL is satisfied.
            sqlx::query(
                "INSERT INTO meetings_new
                 SELECT id, is_current, album_id, album_name, album_artist,
                        album_art_url, album_spotify_url, picker,
                        COALESCE(meeting_date, recorded_at),
                        meeting_time, meeting_location, recorded_at
                 FROM meetings",
            )
            .execute(pool)
            .await?;

            sqlx::query("DROP TABLE meetings").execute(pool).await?;
            sqlx::query("ALTER TABLE meetings_new RENAME TO meetings")
                .execute(pool)
                .await?;
            sqlx::query(
                "CREATE UNIQUE INDEX IF NOT EXISTS only_one_current ON meetings (is_current)
                 WHERE is_current = 1",
            )
            .execute(pool)
            .await?;
        }

        sqlx::query(
            "INSERT OR IGNORE INTO schema_migrations (name) VALUES ('meeting_date_not_null')",
        )
        .execute(pool)
        .await?;
    }

    Ok(())
}

async fn seed_members(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM members")
        .fetch_one(pool)
        .await?;

    if count == 0 {
        for (i, member) in INITIAL_MEMBERS.iter().enumerate() {
            sqlx::query("INSERT OR IGNORE INTO members (name, sort_order) VALUES (?, ?)")
                .bind(member)
                .bind(i as i64)
                .execute(pool)
                .await?;
        }
    }

    Ok(())
}
