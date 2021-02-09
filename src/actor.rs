use crate::{error::MyError, spotify};
use actix::prelude::*;
use diesel::prelude::*;
use github::GithubAccessToken;
use log::{debug, error, info};
use model::SpotifyGithub;
use spotify::SpotifyToken;
use std::env;
use std::time::Duration;

use crate::github;
use crate::model;

fn get_db_connection() -> SqliteConnection {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL");
    SqliteConnection::establish(&database_url).expect("Failed to connect to database.")
}
pub struct Scheduler;

impl Actor for Scheduler {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Context<Self>) {
        debug!("Scheduler is alive.");
        ctx.run_later(Self::duration_until_next(), move |this, ctx| {
            this.schedule_task(ctx)
        });
    }
    fn stopped(&mut self, _ctx: &mut Context<Self>) {
        debug!("Scheduler is stopped.");
    }
}

impl Scheduler {
    fn schedule_task(&self, ctx: &mut Context<Self>) {
        let conn = get_db_connection();
        let spotify_githubs = match model::SpotifyGithub::load_all(&conn) {
            Ok(k) => k,
            Err(e) => {
                error!("{}", e);
                Vec::<SpotifyGithub>::new()
            }
        };
        info!("Scheduler sending {} tasks...", spotify_githubs.len());
        let addr = SpotifyGithubWorker.start();
        for item in spotify_githubs {
            addr.do_send(item);
        }
        ctx.run_later(Self::duration_until_next(), move |this, ctx| {
            this.schedule_task(ctx)
        });
    }
    fn duration_until_next() -> Duration {
        Duration::from_secs(60)
    }
}

struct SpotifyGithubWorker;

impl Actor for SpotifyGithubWorker {
    type Context = Context<Self>;

    fn started(&mut self, _ctx: &mut Context<Self>) {
        debug!("SpotifyGithubWorker is alive.");
    }
    fn stopped(&mut self, _ctx: &mut Context<Self>) {
        debug!("SpotifyGithubWorker is stopped.");
    }
}

impl Message for SpotifyGithub {
    type Result = Result<(), ()>;
}
impl Handler<SpotifyGithub> for SpotifyGithubWorker {
    type Result = Result<(), ()>;

    fn handle(&mut self, msg: SpotifyGithub, _ctx: &mut Context<Self>) -> Self::Result {
        debug!("{:?}", msg);

        let execution = async {
            let spotify_token = SpotifyToken {
                access_token: msg.spotify_access_token.clone(),
                refresh_token: Some(msg.spotify_refresh_token.clone()),
            };
            info!("Refreshing spotify token...");
            let spotify_token = match spotify_token.refresh_token().await {
                Ok(k) => k,
                Err(MyError::SpotifyExpiredTokenError) | Err(MyError::SpotifyTokenError) => {
                    info!("Deleting bad spotify token...");
                    let conn = get_db_connection();
                    msg.delete(&conn).unwrap();
                    return;
                }
                _ => {
                    info!("Error happened...");
                    return;
                }
            };

            info!("Saving new spotify token...");
            let mut spotify_github = msg.clone();
            spotify_github.spotify_access_token = spotify_token.access_token.clone();
            spotify_github.save(&get_db_connection()).unwrap();

            info!("Getting current playing track...");
            let current_playing_item = spotify_token.get_current_playing_item().await;
            let name = match current_playing_item {
                Ok(k) => k.name,
                _ => "".to_string(),
            };
            let github_token = GithubAccessToken {
                username: msg.github_username,
                access_token: msg.github_access_token,
            };
            let bio;
            if name.is_empty() {
                bio = "".to_string();
            } else {
                bio = format!("🎵 {}", name);
            }
            match github_token.update_user_bio(&bio).await {
                Err(MyError::GithubRequestError) => {}
                _ => return,
            }
        };
        Ok(Arbiter::spawn(execution))
    }
}
