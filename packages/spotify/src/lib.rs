use serde::Deserialize;
use std::time::{Duration, Instant};

const SPOTIFY_CLIENT_ID_ENV: &str = "SPOTIFY_CLIENT_ID";
const SPOTIFY_CLIENT_SECRET_ENV: &str = "SPOTIFY_CLIENT_SECRET";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AlbumTrackItem {
    pub id: String,
    pub name: String,
    pub track_number: u32,
    pub duration_ms: Option<u64>,
    pub spotify_url: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AlbumSearchItem {
    pub id: String,
    pub name: String,
    pub artists: String,
    pub image_url: Option<String>,
    pub spotify_url: String,
}

#[derive(Debug)]
pub struct SpotifyError(String);

impl std::fmt::Display for SpotifyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for SpotifyError {}

pub struct SpotifyClient {
    http_client: reqwest::Client,
    client_id: String,
    client_secret: String,
    access_token: Option<String>,
    access_token_expires_at: Option<Instant>,
}

impl SpotifyClient {
    pub fn from_env() -> Result<Self, SpotifyError> {
        let client_id = std::env::var(SPOTIFY_CLIENT_ID_ENV).map_err(|_| {
            SpotifyError("SPOTIFY_CLIENT_ID is not configured on the server".to_string())
        })?;
        let client_secret = std::env::var(SPOTIFY_CLIENT_SECRET_ENV).map_err(|_| {
            SpotifyError("SPOTIFY_CLIENT_SECRET is not configured on the server".to_string())
        })?;

        Ok(Self {
            http_client: reqwest::Client::new(),
            client_id,
            client_secret,
            access_token: None,
            access_token_expires_at: None,
        })
    }

    pub async fn get_album_tracks(
        &mut self,
        album_id: &str,
    ) -> Result<Vec<AlbumTrackItem>, SpotifyError> {
        self.ensure_access_token().await?;

        let response = self.get_album_tracks_request(album_id).await?;
        let status = response.status();

        if status.is_success() {
            return Self::parse_album_tracks_response(response).await;
        }

        let error_body = response.text().await.unwrap_or_default();

        if Self::is_expired_token_response(status, &error_body) {
            self.refresh_access_token().await?;
            let retried = self.get_album_tracks_request(album_id).await?;
            if retried.status().is_success() {
                return Self::parse_album_tracks_response(retried).await;
            }
            let s = retried.status();
            let b = retried.text().await.unwrap_or_default();
            return Err(SpotifyError(format!(
                "Spotify album tracks failed after token refresh (status {s}): {b}"
            )));
        }

        Err(SpotifyError(format!(
            "Spotify album tracks failed with status {}: {}",
            status, error_body
        )))
    }

    async fn get_album_tracks_request(
        &self,
        album_id: &str,
    ) -> Result<reqwest::Response, SpotifyError> {
        let access_token = self
            .access_token
            .as_deref()
            .ok_or_else(|| SpotifyError("Spotify access token is not available".to_string()))?;

        self.http_client
            .get(format!(
                "https://api.spotify.com/v1/albums/{album_id}/tracks"
            ))
            .query(&[("limit", "50")])
            .bearer_auth(access_token)
            .send()
            .await
            .map_err(|e| SpotifyError(format!("Spotify request failed: {e}")))
    }

    async fn parse_album_tracks_response(
        response: reqwest::Response,
    ) -> Result<Vec<AlbumTrackItem>, SpotifyError> {
        let body: SpotifyTracksResponse = response
            .json()
            .await
            .map_err(|e| SpotifyError(format!("Failed to parse Spotify tracks response: {e}")))?;

        Ok(body
            .items
            .into_iter()
            .map(|t| AlbumTrackItem {
                id: t.id,
                name: t.name,
                track_number: t.track_number,
                duration_ms: Some(t.duration_ms),
                spotify_url: t.external_urls.map(|u| u.spotify),
            })
            .collect())
    }

    pub async fn search_albums(
        &mut self,
        query: &str,
    ) -> Result<Vec<AlbumSearchItem>, SpotifyError> {
        let search_term = query.trim();
        if search_term.is_empty() {
            return Ok(Vec::new());
        }

        self.ensure_access_token().await?;

        let response = self.search_albums_request(search_term).await?;
        let status = response.status();

        if status.is_success() {
            return Self::parse_album_search_response(response).await;
        }

        let error_body = response.text().await.unwrap_or_default();

        if Self::is_expired_token_response(status, &error_body) {
            self.refresh_access_token().await?;

            let retried_response = self.search_albums_request(search_term).await?;
            if retried_response.status().is_success() {
                return Self::parse_album_search_response(retried_response).await;
            }

            let retried_status = retried_response.status();
            let retried_error_body = retried_response.text().await.unwrap_or_default();
            return Err(SpotifyError(format!(
                "Spotify search failed after token refresh (status {}): {}",
                retried_status, retried_error_body
            )));
        }

        Err(SpotifyError(format!(
            "Spotify search failed with status {}: {}",
            status, error_body
        )))
    }

    async fn search_albums_request(&self, query: &str) -> Result<reqwest::Response, SpotifyError> {
        let access_token = self
            .access_token
            .as_deref()
            .ok_or_else(|| SpotifyError("Spotify access token is not available".to_string()))?;

        self.http_client
            .get("https://api.spotify.com/v1/search")
            .query(&[("q", query), ("type", "album"), ("limit", "10")])
            .bearer_auth(access_token)
            .send()
            .await
            .map_err(|e| SpotifyError(format!("Spotify request failed: {e}")))
    }

    async fn ensure_access_token(&mut self) -> Result<(), SpotifyError> {
        if self.has_valid_access_token() {
            return Ok(());
        }

        self.refresh_access_token().await
    }

    fn has_valid_access_token(&self) -> bool {
        match (self.access_token.as_ref(), self.access_token_expires_at) {
            (Some(_), Some(expires_at)) => Instant::now() < expires_at,
            _ => false,
        }
    }

    async fn refresh_access_token(&mut self) -> Result<(), SpotifyError> {
        let token_response = self
            .http_client
            .post("https://accounts.spotify.com/api/token")
            .basic_auth(&self.client_id, Some(&self.client_secret))
            .form(&[("grant_type", "client_credentials")])
            .send()
            .await
            .map_err(|e| SpotifyError(format!("Spotify token request failed: {e}")))?;

        if !token_response.status().is_success() {
            let status = token_response.status();
            let body = token_response.text().await.unwrap_or_default();
            return Err(SpotifyError(format!(
                "Spotify token request failed with status {}: {}",
                status, body
            )));
        }

        let token_body: SpotifyTokenResponse = token_response
            .json()
            .await
            .map_err(|e| SpotifyError(format!("Failed to parse Spotify token response: {e}")))?;

        let expires_in = token_body.expires_in.max(30);
        let expires_at = Instant::now() + Duration::from_secs(expires_in - 10);

        self.access_token = Some(token_body.access_token);
        self.access_token_expires_at = Some(expires_at);

        Ok(())
    }

    async fn parse_album_search_response(
        response: reqwest::Response,
    ) -> Result<Vec<AlbumSearchItem>, SpotifyError> {
        let body: SpotifySearchResponse = response
            .json()
            .await
            .map_err(|e| SpotifyError(format!("Failed to parse Spotify response: {e}")))?;

        Ok(body
            .albums
            .items
            .into_iter()
            .map(|album| AlbumSearchItem {
                id: album.id,
                name: album.name,
                artists: album
                    .artists
                    .into_iter()
                    .map(|artist| artist.name)
                    .collect::<Vec<_>>()
                    .join(", "),
                image_url: album.images.first().map(|image| image.url.clone()),
                spotify_url: album.external_urls.spotify,
            })
            .collect::<Vec<_>>())
    }

    fn is_expired_token_response(status: reqwest::StatusCode, body: &str) -> bool {
        if status == reqwest::StatusCode::UNAUTHORIZED {
            return true;
        }

        let lower = body.to_ascii_lowercase();
        lower.contains("access token expired") || lower.contains("token expired")
    }
}

#[derive(Debug, Deserialize)]
struct SpotifyTracksResponse {
    items: Vec<SpotifyTrack>,
}

#[derive(Debug, Deserialize)]
struct SpotifyTrack {
    id: String,
    name: String,
    track_number: u32,
    duration_ms: u64,
    external_urls: Option<SpotifyExternalUrls>,
}

#[derive(Debug, Deserialize)]
struct SpotifyTokenResponse {
    access_token: String,
    expires_in: u64,
}

#[derive(Debug, Deserialize)]
struct SpotifySearchResponse {
    albums: SpotifyAlbums,
}

#[derive(Debug, Deserialize)]
struct SpotifyAlbums {
    items: Vec<SpotifyAlbum>,
}

#[derive(Debug, Deserialize)]
struct SpotifyAlbum {
    id: String,
    name: String,
    artists: Vec<SpotifyArtist>,
    images: Vec<SpotifyImage>,
    external_urls: SpotifyExternalUrls,
}

#[derive(Debug, Deserialize)]
struct SpotifyArtist {
    name: String,
}

#[derive(Debug, Deserialize)]
struct SpotifyImage {
    url: String,
}

#[derive(Debug, Deserialize)]
struct SpotifyExternalUrls {
    spotify: String,
}
