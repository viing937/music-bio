use crate::error::MyError;
use actix_web::client::Client;
use log::{debug, info};
use serde::{Deserialize, Serialize};
use std::env;
use url::Url;

pub enum GrantType {
    AuthorizationCode,
}

#[derive(Deserialize, Debug)]
pub struct SpotifyToken {
    pub access_token: String,
    token_type: String,
    scope: String,
    expires_in: i64,
    pub refresh_token: String,
}

impl SpotifyToken {
    const API_URL_PREFIX: &'static str = "https://api.spotify.com";
    const AUTH_URL_PREFIX: &'static str = "https://accounts.spotify.com";

    fn get_client_id() -> String {
        env::var("SPOTIFY_CLIENT_ID").expect("SPOTIFY_CLIENT_ID")
    }

    fn get_client_secret() -> String {
        env::var("SPOTIFY_CLIENT_SECRET").expect("SPOTIFY_CLIENT_SECRET")
    }

    fn get_callback_url() -> String {
        env::var("CALLBACK_URI").expect("CALLBACK_URI")
    }

    pub fn get_auth_uri(state: &str) -> Result<Url, MyError> {
        Url::parse_with_params(
            &format!("{}/en/authorize", SpotifyToken::AUTH_URL_PREFIX),
            &[
                ("client_id", SpotifyToken::get_client_id()),
                ("redirect_uri", SpotifyToken::get_callback_url()),
                ("response_type", "code".to_string()),
                ("scope", "user-read-playback-state".to_string()),
                ("state", state.to_string()),
            ],
        )
        .map_err(|e| MyError::UriParseError(e))
    }

    pub async fn new(grant_type: GrantType, code: &str) -> Result<SpotifyToken, MyError> {
        info!("Request access token using code {:?}...", code);
        let grant_type = match grant_type {
            GrantType::AuthorizationCode => "authorization_code",
        };
        let client = Client::default();
        let mut resp = client
            .post(format!("{}/api/token", SpotifyToken::AUTH_URL_PREFIX))
            .send_form(&RequestTokenInfo {
                grant_type: grant_type.to_string(),
                code: code.to_string(),
                redirect_uri: SpotifyToken::get_callback_url().to_string(),
                client_id: SpotifyToken::get_client_id().to_string(),
                client_secret: SpotifyToken::get_client_secret().to_string(),
            })
            .await
            .map_err(|e| MyError::SendRequestError(e))?;
        debug!("{:?}", resp);
        if !resp.status().is_success() {
            return Err(MyError::SpotifyRequestError);
        }
        let body = resp.body().await.map_err(|e| MyError::PayloadError(e))?;
        let body = std::str::from_utf8(&body).map_err(|_e| MyError::UnknownError)?;
        let token_info: SpotifyToken =
            serde_json::from_str(body).map_err(|e| MyError::SerdeJsonError(e))?;
        Ok(token_info)
    }

    pub async fn refresh_token(&self) -> Result<SpotifyToken, MyError> {
        let token = SpotifyToken::new(GrantType::AuthorizationCode, &self.refresh_token).await?;
        Ok(token)
    }

    pub async fn get_current_playing_item(&self) -> Result<CurrentPlayingItem, MyError> {
        info!("Request current playing item...");
        let client = Client::default();
        let mut resp = client
            .get(format!(
                "{}/v1/me/player/currently-playing?market=from_token",
                SpotifyToken::API_URL_PREFIX,
            ))
            .bearer_auth(&self.access_token)
            .send()
            .await
            .map_err(|e| MyError::SendRequestError(e))?;
        debug!("{:?}", resp);
        if !resp.status().is_success() {
            return Err(MyError::SpotifyRequestError);
        }
        let body = resp.body().await.map_err(|e| MyError::PayloadError(e))?;
        let body = std::str::from_utf8(&body).map_err(|_e| MyError::UnknownError)?;
        let current_playing_info: CurrentPlayingInfo =
            serde_json::from_str(body).map_err(|e| MyError::SerdeJsonError(e))?;
        Ok(current_playing_info.item)
    }
}

#[derive(Serialize, Debug)]
struct RequestTokenInfo {
    grant_type: String,
    code: String,
    redirect_uri: String,
    client_id: String,
    client_secret: String,
}

#[derive(Deserialize, Debug)]
pub struct CurrentPlayingItem {
    pub id: String,
    pub name: String,
}

#[derive(Deserialize, Debug)]
pub struct CurrentPlayingInfo {
    pub is_playing: bool,
    pub item: CurrentPlayingItem,
}
