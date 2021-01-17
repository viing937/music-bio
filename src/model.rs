use diesel::{RunQueryDsl, SqliteConnection};
use serde::Serialize;

use crate::schema::spotify_github;

#[derive(Debug, Clone, Serialize, Queryable, Insertable)]
#[table_name = "spotify_github"]
pub struct SpotifyGithub {
    pub id: Option<i32>,
    pub github_username: String,
    pub github_access_token: String,
    pub spotify_access_token: String,
    pub spotify_refresh_token: String,
}

impl SpotifyGithub {
    pub fn save(self, conn: &SqliteConnection) -> Result<SpotifyGithub, diesel::result::Error> {
        use crate::schema::spotify_github::dsl::*;
        diesel::replace_into(spotify_github)
            .values(&self)
            .execute(conn)?;
        Ok(self)
    }
}
