use actix_web::client::Client;
use actix_web::http;
use log::{debug, info};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct GithubAccessToken {
    pub username: String,
    pub access_token: String,
}

impl GithubAccessToken {
    const USER_AGENT: &'static str = "Actix-web";
    const HEADER_ACCEPT: &'static str = "application/vnd.github.v3+json";
    const API_URL_PREFIX: &'static str = "https://api.github.com";

    pub async fn update_user_bio(&self, bio: &str) -> Result<(), ()> {
        info!("Update github bio...");
        let client = Client::default();
        let resp = client
            .patch(format!("{}/user", GithubAccessToken::API_URL_PREFIX))
            .basic_auth(self.username.clone(), Some(&self.access_token))
            .set_header(http::header::ACCEPT, GithubAccessToken::HEADER_ACCEPT)
            .set_header(http::header::USER_AGENT, GithubAccessToken::USER_AGENT)
            .send_json(&UserInfo {
                bio: bio.to_string(),
            })
            .await
            .unwrap();
        debug!("{:?}", resp);
        if !resp.status().is_success() {
            return Err(());
        }
        Ok(())
    }
}

#[derive(Serialize, Debug)]
struct UserInfo {
    bio: String,
}
