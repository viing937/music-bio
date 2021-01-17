#[macro_use]
extern crate diesel;

use actix_web::error::BlockingError;
use actix_web::middleware::Logger;
use actix_web::{http, web, App, Error, HttpResponse, HttpServer, Responder};
use diesel::r2d2::{self, ConnectionManager};
use diesel::SqliteConnection;
use dotenv::dotenv;
use log::{debug, info};
use serde::Deserialize;
use std::env;

mod spotify;
use spotify::SpotifyToken;
mod github;
use github::GithubAccessToken;
mod cipher;
mod error;
use error::MyError;
mod model;
mod schema;

type DbPool = r2d2::Pool<ConnectionManager<SqliteConnection>>;

#[actix_web::get("/healthz")]
async fn healthz() -> impl Responder {
    HttpResponse::Ok().body("ok.")
}

#[derive(Deserialize, Debug)]
struct AuthInfo {
    github_username: String,
    github_access_token: String,
}

#[actix_web::get("/auth")]
async fn auth(info: web::Query<AuthInfo>) -> Result<HttpResponse, Error> {
    debug!("{:?}", info);
    let state = serde_json::to_string(&GithubAccessToken {
        username: info.github_username.clone(),
        access_token: info.github_access_token.clone(),
    })
    .map_err(|e| MyError::SerdeJsonError(e))?;
    let state = cipher::encrypt(&state).await?;
    let url = SpotifyToken::get_auth_uri(&state)?;
    Ok(HttpResponse::Found()
        .set_header(http::header::LOCATION, url.to_string())
        .finish())
}

#[derive(Deserialize, Debug)]
struct AuthCallbackInfo {
    code: String,
    state: String,
}

#[actix_web::get("/callback")]
async fn auth_callback(
    pool: web::Data<DbPool>,
    info: web::Query<AuthCallbackInfo>,
) -> Result<HttpResponse, Error> {
    debug!("{:?}", info);
    let token_info = SpotifyToken::new(spotify::GrantType::AuthorizationCode, &info.code).await?;
    debug!("{:?}", token_info);
    let current_playing_item = token_info.get_current_playing_item().await?;
    debug!("{:?}", current_playing_item);

    let state = cipher::decrypt(&info.state).await?;
    let github_access_token: GithubAccessToken = serde_json::from_str(&state)?;

    let conn = pool.get().map_err(|e| MyError::DbPoolError(e))?;
    let spotify_github = model::SpotifyGithub {
        id: None,
        github_username: github_access_token.username.clone(),
        github_access_token: github_access_token.access_token.clone(),
        spotify_access_token: token_info.access_token.clone(),
        spotify_refresh_token: token_info.refresh_token.clone(),
    };
    let spotify_github = match web::block(move || spotify_github.save(&conn)).await {
        Ok(v) => Ok(v),
        Err(BlockingError::Canceled) => Err(MyError::BlockingCancledError),
        Err(BlockingError::Error(e)) => Err(MyError::DbResultError(e)),
    }?;
    debug!("{:?}", spotify_github);

    github_access_token
        .update_user_bio(&format!("🎵 {}", current_playing_item.name))
        .await?;
    Ok(HttpResponse::Ok().body("ok."))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    env_logger::init();

    info!("Setting up database connection pool...");
    let connspec = std::env::var("DATABASE_URL").expect("DATABASE_URL");
    let manager = ConnectionManager::<SqliteConnection>::new(connspec);
    let pool = r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create pool.");

    info!("Starting web server...");
    HttpServer::new(move || {
        App::new()
            .data(pool.clone())
            .wrap(Logger::default())
            .service(healthz)
            .service(auth)
            .service(auth_callback)
    })
    .bind(env::var("BIND_ADDR").expect("BIND_ADDR"))?
    .run()
    .await
}
