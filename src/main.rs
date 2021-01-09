use actix_web::middleware::Logger;
use actix_web::{http, web, App, HttpResponse, HttpServer, Responder};
use dotenv::dotenv;
use log::{debug, info};
use serde::Deserialize;
use std::env;

mod spotify;
use spotify::SpotifyToken;

mod github;
use github::GithubAccessToken;

#[actix_web::get("/healthz")]
async fn healthz() -> impl Responder {
    HttpResponse::Ok().body("ok!")
}

#[derive(Deserialize, Debug)]
struct AuthInfo {
    github_username: String,
    github_access_token: String,
}

#[actix_web::get("/auth")]
async fn auth(info: web::Query<AuthInfo>) -> impl Responder {
    debug!("{:?}", info);
    let state = serde_json::to_string(&GithubAccessToken {
        username: info.github_username.clone(),
        access_token: info.github_access_token.clone(),
    })
    .unwrap();
    let url = SpotifyToken::get_auth_uri(&state).unwrap();
    HttpResponse::Found()
        .set_header(http::header::LOCATION, url.to_string())
        .finish()
}

#[derive(Deserialize, Debug)]
struct AuthCallbackInfo {
    code: String,
    state: String,
}

#[actix_web::get("/callback")]
async fn auth_callback(info: web::Query<AuthCallbackInfo>) -> impl Responder {
    debug!("{:?}", info);
    let token_info = SpotifyToken::new(spotify::GrantType::AuthorizationCode, &info.code)
        .await
        .unwrap();
    debug!("{:?}", token_info);
    let current_playing_item = token_info.get_current_playing_item().await.unwrap();
    debug!("{:?}", current_playing_item);
    let github_access_token: GithubAccessToken = serde_json::from_str(&info.state).unwrap();
    github_access_token
        .update_user_bio(&format!("🎵 {}", current_playing_item.name))
        .await
        .unwrap();
    HttpResponse::Ok().body("ok.")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    env_logger::init();
    info!("Web server starting...");
    HttpServer::new(|| {
        App::new()
            .wrap(Logger::default())
            .service(healthz)
            .service(auth)
            .service(auth_callback)
    })
    .bind(env::var("BIND_ADDR").unwrap())?
    .run()
    .await
}
