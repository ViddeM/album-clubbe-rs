CREATE TABLE IF NOT EXISTS members (
    name       TEXT    NOT NULL PRIMARY KEY,
    sort_order INTEGER NOT NULL DEFAULT 0
);

-- meetings holds both the current active entry (is_current = 1) and history.
-- The partial unique index enforces at most one current entry.
CREATE TABLE IF NOT EXISTS meetings (
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

CREATE UNIQUE INDEX IF NOT EXISTS only_one_current ON meetings (is_current)
WHERE is_current = 1;
