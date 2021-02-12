use std::{str::Utf8Error, string::FromUtf8Error};

use actix_web::{
    client::SendRequestError, dev::HttpResponseBuilder, error::PayloadError, http::StatusCode,
    HttpResponse, ResponseError,
};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CustomError {
    #[error("Utf8Error: {0}")]
    Utf8Error(#[from] Utf8Error),
    #[error("UriParseError: {0}")]
    UriParseError(#[from] url::ParseError),
    #[error("DbPoolError: {0}")]
    DbPoolError(#[from] r2d2::Error),
    #[error("DbResultError: {0}")]
    DbResultError(#[from] diesel::result::Error),
    #[error("SerdeJsonError: {0}")]
    SerdeJsonError(#[from] serde_json::Error),
    #[error("BlockCancledError")]
    BlockingCancledError,
    #[error("OpensslStackError: {0}")]
    OpensslStackError(#[from] openssl::error::ErrorStack),
    #[error("FromUtf8Error: {0}")]
    FromUtf8Error(#[from] FromUtf8Error),
    #[error("PayloadError: {0}")]
    PayloadError(#[from] PayloadError),
    #[error("SendRequestError: {0}")]
    SendRequestError(#[from] SendRequestError),
    #[error("SpotifyRequestError")]
    SpotifyRequestError,
    #[error("SpotifyTokenError")]
    SpotifyTokenError,
    #[error("SpotifyExpiredTokenError")]
    SpotifyExpiredTokenError,
    #[error("SpotifyNotPlayingError")]
    SpotifyNotPlayingError,
    #[error("GithubRequestError")]
    GithubRequestError,
}

impl ResponseError for CustomError {
    fn error_response(&self) -> HttpResponse {
        HttpResponseBuilder::new(self.status_code()).body(self.to_string())
    }
    fn status_code(&self) -> StatusCode {
        match *self {
            CustomError::SpotifyRequestError => StatusCode::BAD_REQUEST,
            CustomError::GithubRequestError => StatusCode::BAD_REQUEST,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}
