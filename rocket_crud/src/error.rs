use chrono::ParseError as ChronoParseError;
use rocket::form::error::ErrorKind as FormErrorKind;
use std::convert::Infallible;
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
