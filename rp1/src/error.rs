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
    #[error("Could not parse integer: {0}")]
    IntError(ParseIntError),

    #[error("Could not parse boolean: {0}")]
    BoolError(ParseBoolError),

    #[error("Could not parse date/time value: {0}")]
    TimeError(::time::Error),

    #[error("An infallible conversion somehow failed")]
    Infallible,

    #[error("Unknown operator '{0}'")]
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

impl From<::time::Error> for ParseError {
    fn from(e: ::time::Error) -> Self {
        ParseError::TimeError(e)
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

#[derive(thiserror::Error, Debug)]
pub enum CrudError {
    #[error("Not Found")]
    NotFound,
    #[error("Forbidden")]
    Forbidden,
    #[error("Error from the database layer: {0}")]
    DbError(::diesel::result::Error),
    #[cfg(feature = "validation")]
    #[error("There are validation errors: {0}")]
    ValidationErrors(::validator::ValidationErrors),
    #[error("Field {0} is not allowed to be changed")]
    UnchangeableField(String),
}

impl From<::diesel::result::Error> for CrudError {
    fn from(e: ::diesel::result::Error) -> Self {
        CrudError::DbError(e)
    }
}

#[cfg(feature = "validation")]
impl From<::validator::ValidationErrors> for CrudError {
    fn from(e: ::validator::ValidationErrors) -> Self {
        CrudError::ValidationErrors(e)
    }
}

impl CrudError {
    fn status(&self) -> Status {
        match self {
            CrudError::NotFound => Status::NotFound,
            CrudError::Forbidden => Status::Forbidden,
            CrudError::DbError(::diesel::result::Error::NotFound) => Status::NotFound,
            CrudError::DbError(_) => Status::InternalServerError,
            #[cfg(feature = "validation")]
            CrudError::ValidationErrors(_) => Status::BadRequest,
            CrudError::UnchangeableField(_) => Status::BadRequest,
        }
    }
}

impl<'r> Responder<'r, 'static> for CrudError {
    fn respond_to(self, _: &'r rocket::Request<'_>) -> rocket::response::Result<'static> {
        let status = self.status();
        let body: String = ::serde_json::json!({
            "error": status.code,
            "message": self.to_string(),
        })
        .to_string();
        Response::build()
            .status(status)
            .header(ContentType::JSON)
            .sized_body(body.as_bytes().len(), Cursor::new(body))
            .ok()
    }
}
