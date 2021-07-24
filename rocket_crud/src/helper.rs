//! Contains some utilities mostly useful internally, but may be used in other
//! places a well.

use crate::{Either, RocketCrudResponse};

use diesel::result::Error as DieselError;
use rocket::http::Status;
use rocket::serde::json::{json, Json};
use serde::de::{Deserialize, Deserializer};

#[cfg(feature = "validator")]
use validator::ValidationErrors;

/// Deserialize helper to support double option types.
///
/// These kinds of types are currently being generated when we have a nullable
/// field in the database. In such a case we want to be able to identify the
/// difference between a value not being present and a value being explicitly
/// set to null. Using this deserialization we can identify the first case by
/// having the outer option being `None`, and the second case can be identified
/// by the inner option being `None`.
pub fn double_option<'de, T, D>(de: D) -> Result<Option<Option<T>>, D::Error>
where
    T: Deserialize<'de>,
    D: Deserializer<'de>,
{
    Deserialize::deserialize(de).map(Some)
}

/// When a `RocketCrudResponse<T>` is expected as a response type, this can be
/// used to quickly return a 200 response of the accepted type.
pub fn ok_to_response<T>(value: T) -> RocketCrudResponse<T> {
    (Status::Ok, Either::Left(Json(value)))
}

/// Convert a validation error to a rocket response.
#[cfg(feature = "validator")]
pub fn validation_error_to_response<T>(errors: ValidationErrors) -> RocketCrudResponse<T> {
    let field_errors = errors.field_errors();
    (
        Status::BadRequest,
        Either::Right(json!({
            "error": 400,
            "validation_error": field_errors,
        })),
    )
}

/// Convert a database error to a rocket response.
pub fn db_error_to_response<T>(error: DieselError) -> RocketCrudResponse<T> {
    match error {
        DieselError::NotFound => (
            Status::NotFound,
            Either::Right(json!({
                "error": 404,
            })),
        ),
        _ => (
            Status::InternalServerError,
            Either::Right(json!({
                "error": 500,
            })),
        ),
    }
}
