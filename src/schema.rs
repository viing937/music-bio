table! {
    spotify_github (id) {
        id -> Nullable<Integer>,
        github_username -> Text,
        github_access_token -> Text,
        spotify_access_token -> Text,
        spotify_refresh_token -> Text,
    }
}
