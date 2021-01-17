-- Your SQL goes here
CREATE TABLE spotify_github (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    github_username TEXT UNIQUE NOT NULL,
    github_access_token TEXT NOT NULL,
    spotify_access_token TEXT NOT NULL,
    spotify_refresh_token TEXT NOT NULL
);
