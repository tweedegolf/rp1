pub use rocket_crud_macros::crud;

pub trait CrudInsertableMarker {}

pub trait CrudUpdatableMarker {}

pub use either::Either;

use rocket::http::ContentType;
use rocket::request::{self, FromRequest, Outcome, Request};
use serde::de::{Deserialize, Deserializer};
use std::convert::Infallible;

pub type RocketCrudResponse<T> = (
    ::rocket::http::Status,
    Either<::rocket::serde::json::Json<T>, ::rocket::serde::json::Value>,
);

#[derive(serde::Deserialize, serde::Serialize, rocket::FromFormField, Debug, PartialEq, Eq)]
pub enum SortDirection {
    #[serde(rename = "asc")]
    Asc,
    #[serde(rename = "desc")]
    Desc,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct SortSpec<T> {
    pub field: T,
    pub direction: SortDirection,
}

impl<T: std::fmt::Display> std::fmt::Display for SortSpec<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.direction == SortDirection::Desc {
            write!(f, "-")?;
        }
        write!(f, "{}", self.field)
    }
}

impl<'v, T: rocket::form::FromFormField<'v>> SortSpec<T> {
    fn from_str(
        mut data: &'v str,
        name: &rocket::form::name::NameView<'v>,
    ) -> rocket::form::Result<'v, SortSpec<T>> {
        let mut direction = SortDirection::Asc;
        if data.starts_with('-') {
            data = &data[1..];
            direction = SortDirection::Desc;
        } else if data.starts_with('+') {
            data = &data[1..];
        }

        let field = rocket::form::FromFormField::from_value(rocket::form::ValueField {
            name: *name,
            value: data,
        })?;

        Ok(SortSpec { field, direction })
    }
}

#[rocket::async_trait]
impl<'v, T: rocket::form::FromFormField<'v>> rocket::form::FromFormField<'v> for SortSpec<T> {
    fn from_value(field: rocket::form::ValueField<'v>) -> rocket::form::Result<'v, Self> {
        SortSpec::from_str(field.value, &field.name)
    }

    async fn from_data(field: rocket::form::DataField<'v, '_>) -> rocket::form::Result<'v, Self> {
        use rocket::data::ToByteUnit;

        let limit = 256.kibibytes();
        let bytes = field.data.open(limit).into_bytes().await?;
        if !bytes.is_complete() {
            return Err((None, Some(limit)).into());
        }
        let bytes = bytes.into_inner();
        let bytes = rocket::request::local_cache!(field.request, bytes);
        let data = std::str::from_utf8(bytes)?;

        SortSpec::from_str(data, &field.name)
    }

    fn default() -> Option<Self> {
        None
    }
}

pub fn double_option<'de, T, D>(de: D) -> Result<Option<Option<T>>, D::Error>
where
    T: Deserialize<'de>,
    D: Deserializer<'de>,
{
    Deserialize::deserialize(de).map(Some)
}

use diesel::result::Error;
use rocket::http::Status;
use rocket::serde::json::{json, Json};

pub fn db_error_to_response<T>(error: Error) -> RocketCrudResponse<T> {
    match error {
        Error::NotFound => (
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

pub fn ok_to_response<T>(value: T) -> RocketCrudResponse<T> {
    (Status::Ok, Either::Left(Json(value)))
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum RequestContentType {
    Json,
    Other,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for RequestContentType {
    type Error = Infallible;

    async fn from_request(request: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        match request.content_type() {
            Some(ct) if ct == &ContentType::JSON => Outcome::Success(RequestContentType::Json),
            _ => Outcome::Success(RequestContentType::Other),
        }
    }
}
