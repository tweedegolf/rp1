use chrono::ParseError as ChronoParseError;
use rocket::form::error::ErrorKind as FormErrorKind;
use rocket::http::{ContentType, Status};
use rocket::response::Responder;
use rocket::Response;
use std::convert::Infallible;
use std::io::Cursor;
use std::num::ParseIntError;
use std::str::ParseBoolError;

/// Indicates an error while trying to parse a filter value in the query string.
#[derive(thiserror::Error, Debug)]
pub enum ParseError {
    #[error("Could not parse integer")]
    IntError(ParseIntError),

    #[error("Could not parse boolean")]
    BoolError(ParseBoolError),

    #[error("Could not parse timestamp")]
    ChronoError(ChronoParseError),

    #[error("An infallible conversion somehow failed")]
    Infallible,

    #[error("Unknown operator")]
    UnknownOperator(String),
}

impl From<ParseIntError> for ParseError {
    fn from(e: ParseIntError) -> Self {
        ParseError::IntError(e)
    }
}

impl From<ParseBoolError> for ParseError {
    fn from(e: ParseBoolError) -> Self {
        ParseError::BoolError(e)
    }
}

impl From<ChronoParseError> for ParseError {
    fn from(e: ChronoParseError) -> Self {
        ParseError::ChronoError(e)
    }
}

impl From<Infallible> for ParseError {
    fn from(_: Infallible) -> Self {
        ParseError::Infallible
    }
}

impl<'v> From<ParseError> for FormErrorKind<'v> {
    fn from(_: ParseError) -> Self {
        FormErrorKind::Unknown
    }
}

pub enum CrudError {
    NotFound,
    Forbidden,
    Example,
    DbError(::diesel::result::Error),
    #[cfg(feature = "validator")]
    ValidationErrors(::validator::ValidationErrors),
    #[cfg(not(feature = "validator"))]
    ValidationErrors(()),
}

impl From<::diesel::result::Error> for CrudError {
    fn from(e: ::diesel::result::Error) -> Self {
        CrudError::DbError(e)
    }
}

#[cfg(feature = "validator")]
impl From<::validator::ValidationErrors> for CrudError {
    fn from(e: ::validator::ValidationErrors) -> Self {
        CrudError::ValidationErrors(e)
    }
}

impl CrudError {
    fn status(&self) -> Status {
        match self {
            CrudError::Example => Status::InternalServerError,
            CrudError::NotFound => Status::NotFound,
            CrudError::Forbidden => Status::Forbidden,
            CrudError::DbError(::diesel::result::Error::NotFound) => Status::NotFound,
            CrudError::DbError(_) => Status::InternalServerError,
            CrudError::ValidationErrors(_) => Status::BadRequest,
        }
    }
}

impl<'r> Responder<'r, 'static> for CrudError {
    fn respond_to(self, _: &'r rocket::Request<'_>) -> rocket::response::Result<'static> {
        let status = self.status();
        let body: String = ::serde_json::json!({
            "error": status.code,
        })
        .to_string();
        Response::build()
            .status(status)
            .header(ContentType::JSON)
            .sized_body(body.as_bytes().len(), Cursor::new(body))
            .ok()
    }
}
