-- Add the Spotify track URL to the cached track listing.
-- Existing rows will have NULL; they'll be refreshed on next page load.
ALTER TABLE album_tracks ADD COLUMN spotify_url TEXT;
