-- The previous hand-rolled migration copied data into meetings_new but never
-- completed the final DROP/RENAME, leaving all data stranded there.
-- Ensure the table exists (no-op on fresh DBs), rescue any rows, then clean up.
CREATE TABLE IF NOT EXISTS meetings_new (
    id                TEXT    NOT NULL PRIMARY KEY,
    is_current        INTEGER NOT NULL DEFAULT 0,
    album_id          TEXT    NOT NULL,
    album_name        TEXT    NOT NULL,
    album_artist      TEXT    NOT NULL,
    album_art_url     TEXT    NOT NULL,
    album_spotify_url TEXT    NOT NULL,
    picker            TEXT    NOT NULL,
    meeting_date      TEXT    NOT NULL,
    meeting_time      TEXT,
    meeting_location  TEXT,
    recorded_at       TEXT    NOT NULL DEFAULT (datetime('now'))
);

INSERT OR IGNORE INTO meetings
SELECT id, is_current, album_id, album_name, album_artist,
       album_art_url, album_spotify_url, picker,
       meeting_date, meeting_time, meeting_location, recorded_at
FROM meetings_new;

DROP TABLE meetings_new;

-- Drop the old hand-rolled migration tracking table; sqlx uses _sqlx_migrations.
DROP TABLE IF EXISTS schema_migrations;
