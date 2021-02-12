use crate::error::CustomError;
use actix_web::{client::Client, http::StatusCode};
use log::{debug, info};
use serde::{Deserialize, Serialize};
use std::env;
use url::Url;

pub enum GrantType {
    RefreshToken,
    AuthorizationCode,
}

#[derive(Deserialize, Debug)]
pub struct SpotifyToken {
    pub access_token: String,
    pub refresh_token: Option<String>,
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

    pub fn get_auth_uri(state: &str) -> Result<Url, CustomError> {
        let auth_url = Url::parse_with_params(
            &format!("{}/en/authorize", SpotifyToken::AUTH_URL_PREFIX),
            &[
                ("client_id", SpotifyToken::get_client_id()),
                ("redirect_uri", SpotifyToken::get_callback_url()),
                ("response_type", "code".to_string()),
                ("scope", "user-read-playback-state".to_string()),
                ("state", state.to_string()),
            ],
        )?;
        Ok(auth_url)
    }

    pub async fn new(grant_type: GrantType, code: &str) -> Result<SpotifyToken, CustomError> {
        info!("Request access token using code {:?}...", code);
        let client = Client::default();
        let request_body = match grant_type {
            GrantType::RefreshToken => RequestTokenInfo {
                grant_type: "refresh_token".to_string(),
                code: None,
                redirect_uri: None,
                refresh_token: Some(code.to_string()),
            },
            GrantType::AuthorizationCode => RequestTokenInfo {
                grant_type: "authorization_code".to_string(),
                code: Some(code.to_string()),
                redirect_uri: Some(SpotifyToken::get_callback_url().to_string()),
                refresh_token: None,
            },
        };
        let mut resp = client
            .post(format!("{}/api/token", SpotifyToken::AUTH_URL_PREFIX))
            .basic_auth(
                SpotifyToken::get_client_id(),
                Some(&SpotifyToken::get_client_secret()),
            )
            .send_form(&request_body)
            .await?;
        debug!("{:?}", resp);
        match resp.status() {
            StatusCode::BAD_REQUEST => return Err(CustomError::SpotifyTokenError),
            StatusCode::UNAUTHORIZED => return Err(CustomError::SpotifyExpiredTokenError),
            _ => (),
        };
        if !resp.status().is_success() {
            return Err(CustomError::SpotifyRequestError);
        }
        let body = resp.body().await?;
        let body = std::str::from_utf8(&body)?;
        let token_info: SpotifyToken = serde_json::from_str(body)?;
        Ok(token_info)
    }

    pub async fn refresh_token(mut self) -> Result<SpotifyToken, CustomError> {
        info!("Refresh spotify token...");
        let token =
            SpotifyToken::new(GrantType::RefreshToken, &self.refresh_token.unwrap()).await?;
        self.access_token = token.access_token;
        self.refresh_token = token.refresh_token;
        Ok(self)
    }

    pub async fn get_current_playing_item(&self) -> Result<CurrentPlayingItem, CustomError> {
        info!("Request current playing item...");
        let client = Client::default();
        let mut resp = client
            .get(format!(
                "{}/v1/me/player/currently-playing?market=from_token",
                SpotifyToken::API_URL_PREFIX,
            ))
            .bearer_auth(&self.access_token)
            .send()
            .await?;
        debug!("{:?}", resp);
        match resp.status() {
            StatusCode::BAD_REQUEST => return Err(CustomError::SpotifyTokenError),
            StatusCode::NO_CONTENT => return Err(CustomError::SpotifyNotPlayingError),
            _ => (),
        };
        if !resp.status().is_success() {
            return Err(CustomError::SpotifyRequestError);
        }
        let body = resp.body().await?;
        let body = std::str::from_utf8(&body)?;
        let current_playing_info: CurrentPlayingInfo = serde_json::from_str(body)?;
        Ok(current_playing_info.item)
    }
}

#[derive(Serialize, Debug)]
struct RequestTokenInfo {
    grant_type: String,
    code: Option<String>,
    redirect_uri: Option<String>,
    refresh_token: Option<String>,
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
