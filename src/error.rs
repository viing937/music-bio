use actix_web::{dev::HttpResponseBuilder, http::StatusCode, HttpResponse, ResponseError};
use derive_more::Display;

#[derive(Debug, Display)]
pub enum MyError {
    #[display(fmt = "UnknownError: {}", _0)]
    UnknownError(String),
    #[display(fmt = "UriParseError: {}", _0)]
    UriParseError(url::ParseError),
}

impl ResponseError for MyError {
    fn error_response(&self) -> HttpResponse {
        HttpResponseBuilder::new(self.status_code()).body(self.to_string())
    }
    fn status_code(&self) -> StatusCode {
        match *self {
            MyError::UriParseError(_) => StatusCode::BAD_REQUEST,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}
