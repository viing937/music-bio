use std::string::FromUtf8Error;

use actix_web::{
    client::SendRequestError, dev::HttpResponseBuilder, error::PayloadError, http::StatusCode,
    HttpResponse, ResponseError,
};
use derive_more::Display;

#[derive(Debug, Display)]
pub enum MyError {
    #[display(fmt = "UnknownError")]
    UnknownError,
    #[display(fmt = "UriParseError: {}", _0)]
    UriParseError(url::ParseError),
    #[display(fmt = "DbPoolError: {}", _0)]
    DbPoolError(r2d2::Error),
    #[display(fmt = "DbResultError: {}", _0)]
    DbResultError(diesel::result::Error),
    #[display(fmt = "SerdeJsonError: {}", _0)]
    SerdeJsonError(serde_json::Error),
    #[display(fmt = "BlockCancledError")]
    BlockingCancledError,
    #[display(fmt = "OpensslError: {}", _0)]
    OpensslError(openssl::error::ErrorStack),
    #[display(fmt = "FromUtf8Error: {}", _0)]
    FromUtf8Error(FromUtf8Error),
    #[display(fmt = "PayloadError: {}", _0)]
    PayloadError(PayloadError),
    #[display(fmt = "SendRequestError: {}", _0)]
    SendRequestError(SendRequestError),
    #[display(fmt = "SpotifyRequestError")]
    SpotifyRequestError,
    #[display(fmt = "GithubRequestError")]
    GithubRequestError,
}

impl ResponseError for MyError {
    fn error_response(&self) -> HttpResponse {
        HttpResponseBuilder::new(self.status_code()).body(self.to_string())
    }
    fn status_code(&self) -> StatusCode {
        match *self {
            MyError::SpotifyRequestError => StatusCode::BAD_REQUEST,
            MyError::GithubRequestError => StatusCode::BAD_REQUEST,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}
