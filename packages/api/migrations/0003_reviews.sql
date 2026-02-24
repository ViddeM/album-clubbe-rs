-- Add an optional pre-shared password hash for each member.
-- NULL means no password has been set yet.
ALTER TABLE members ADD COLUMN password_hash TEXT;

-- Album-level reviews: one score (0-10) per member per meeting.
-- UNIQUE constraint lets us upsert without duplicate rows.
CREATE TABLE IF NOT EXISTS reviews (
    id           TEXT    NOT NULL PRIMARY KEY,
    meeting_id   TEXT    NOT NULL REFERENCES meetings(id) ON DELETE CASCADE,
    member_name  TEXT    NOT NULL REFERENCES members(name) ON DELETE CASCADE,
    score        INTEGER NOT NULL CHECK(score >= 0 AND score <= 10),
    created_at   TEXT    NOT NULL DEFAULT (datetime('now')),
    updated_at   TEXT    NOT NULL DEFAULT (datetime('now')),
    UNIQUE(meeting_id, member_name)
);

-- Cached track listing fetched from Spotify so we don't hit the API on every page load.
CREATE TABLE IF NOT EXISTS album_tracks (
    album_id     TEXT    NOT NULL,
    track_number INTEGER NOT NULL,
    track_id     TEXT    NOT NULL,
    track_name   TEXT    NOT NULL,
    duration_ms  INTEGER,
    PRIMARY KEY (album_id, track_id)
);

-- Per-track reviews: one score (0-10) per member per track per meeting.
CREATE TABLE IF NOT EXISTS track_reviews (
    id          TEXT    NOT NULL PRIMARY KEY,
    meeting_id  TEXT    NOT NULL REFERENCES meetings(id) ON DELETE CASCADE,
    member_name TEXT    NOT NULL REFERENCES members(name) ON DELETE CASCADE,
    track_id    TEXT    NOT NULL,
    score       INTEGER NOT NULL CHECK(score >= 0 AND score <= 10),
    created_at  TEXT    NOT NULL DEFAULT (datetime('now')),
    updated_at  TEXT    NOT NULL DEFAULT (datetime('now')),
    UNIQUE(meeting_id, member_name, track_id)
);
