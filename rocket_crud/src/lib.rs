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
use std::str::FromStr;

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
            data = data.trim_start_matches('-');
            direction = SortDirection::Desc;
        } else if data.starts_with('+') {
            data = data.trim_start_matches('+');
        }

        // in practice, a `sort=+username` will give us a data that starts with a space ` username`
        data = data.trim_start_matches(' ');

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

use diesel::result::Error as DieselError;
use validator::ValidationErrors;
use rocket::http::Status;
use rocket::serde::json::{json, Json};

pub fn validation_error_to_response<T>(errors: ValidationErrors) -> RocketCrudResponse<T> {
    let field_errors = errors.field_errors();
    (
        Status::BadRequest,
        Either::Right(json!({
            "error": 400,
            "message": format!("{:?}", field_errors),
        })),
    )
}

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

#[derive(Debug)]
pub enum FilterOperator<T> {
    Eq(T),
    Ne(T),
    Gt(T),
    Ge(T),
    Lt(T),
    Le(T),
    EqAny(Vec<T>),
}

fn parse_single_operand<T: FromStr>(input: &str) -> Result<T, ParseError>
where
    <T as FromStr>::Err: Into<ParseError>,
{
    input.parse().map_err(|e: <T as FromStr>::Err| e.into())
}

fn parse_vec_operand<T: FromStr>(input: &str) -> Result<Vec<T>, ParseError>
where
    <T as FromStr>::Err: Into<ParseError>,
{
    input
        .split(",")
        .map(|segment| segment.parse())
        .collect::<Result<Vec<T>, <T as FromStr>::Err>>()
        .map_err(|e: <T as FromStr>::Err| e.into())
}

impl<T: FromStr> FilterOperator<Option<T>>
where
    <T as FromStr>::Err: Into<ParseError>,
{
    pub fn from_none(op: &str) -> Result<Self, ParseError> {
        match op {
            "eq" => Ok(FilterOperator::Eq(None)),
            "ne" => Ok(FilterOperator::Ne(None)),
            "gt" => Ok(FilterOperator::Gt(None)),
            "ge" => Ok(FilterOperator::Ge(None)),
            "lt" => Ok(FilterOperator::Lt(None)),
            "le" => Ok(FilterOperator::Le(None)),
            "in" => Ok(FilterOperator::EqAny(vec![])),
            _ => Err(ParseError::UnknownOperator(op.to_owned())),
        }
    }

    pub fn try_parse_option(op: &str, value: &str) -> Result<Self, ParseError> {
        let value: FilterOperator<T> = FilterOperator::try_parse(op, value)?;
        Ok(match value {
            FilterOperator::Eq(v) => FilterOperator::Eq(Some(v)),
            FilterOperator::Ne(v) => FilterOperator::Ne(Some(v)),
            FilterOperator::Gt(v) => FilterOperator::Ne(Some(v)),
            FilterOperator::Ge(v) => FilterOperator::Ne(Some(v)),
            FilterOperator::Lt(v) => FilterOperator::Ne(Some(v)),
            FilterOperator::Le(v) => FilterOperator::Ne(Some(v)),
            FilterOperator::EqAny(v) => {
                FilterOperator::EqAny(v.into_iter().map(|v| Some(v)).collect())
            }
        })
    }
}

impl<T: FromStr> FilterOperator<T>
where
    <T as FromStr>::Err: Into<ParseError>,
{
    pub fn try_parse(op: &str, value: &str) -> Result<Self, ParseError> {
        match op {
            "eq" => Ok(FilterOperator::Eq(parse_single_operand(value)?)),
            "ne" => Ok(FilterOperator::Ne(parse_single_operand(value)?)),
            "gt" => Ok(FilterOperator::Gt(parse_single_operand(value)?)),
            "ge" => Ok(FilterOperator::Ge(parse_single_operand(value)?)),
            "lt" => Ok(FilterOperator::Lt(parse_single_operand(value)?)),
            "le" => Ok(FilterOperator::Le(parse_single_operand(value)?)),
            "in" => Ok(FilterOperator::EqAny(parse_vec_operand(value)?)),
            _ => Err(ParseError::UnknownOperator(op.to_owned())),
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Unknown field for filter")]
    FilterUnknownField { field: String },

    #[error("Unknown operation on field")]
    FilterUnknownOperation { op: String },
}

#[derive(thiserror::Error, Debug)]
pub enum ParseError {
    #[error("Could not parse integer")]
    IntError(::std::num::ParseIntError),

    #[error("Could not parse boolean")]
    BoolError(::std::str::ParseBoolError),

    #[error("Could not parse timestamp")]
    ChronoError(::chrono::ParseError),

    #[error("An infallible conversion somehow failed")]
    Infallible,

    #[error("Unknown operator")]
    UnknownOperator(String),
}

impl From<::std::num::ParseIntError> for ParseError {
    fn from(e: ::std::num::ParseIntError) -> Self {
        ParseError::IntError(e)
    }
}

impl From<::std::str::ParseBoolError> for ParseError {
    fn from(e: ::std::str::ParseBoolError) -> Self {
        ParseError::BoolError(e)
    }
}

impl From<::chrono::ParseError> for ParseError {
    fn from(e: ::chrono::ParseError) -> Self {
        ParseError::ChronoError(e)
    }
}

impl From<::std::convert::Infallible> for ParseError {
    fn from(_: ::std::convert::Infallible) -> Self {
        ParseError::Infallible
    }
}

impl<'v> From<ParseError> for ::rocket::form::error::ErrorKind<'v> {
    fn from(_: ParseError) -> Self {
        ::rocket::form::error::ErrorKind::Unknown
    }
}

impl<'v> From<Error> for ::rocket::form::error::ErrorKind<'v> {
    fn from(_: Error) -> Self {
        // TODO: a better implementation of this
        ::rocket::form::error::ErrorKind::Unknown
    }
}
