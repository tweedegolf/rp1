//! `rocket_crud` is a [diesel] based CRUD for [rocket]. `rocket_crud` is
//! intented as a quick starting point. It allows you to quickly scaffold a
//! basic application and should implement all basic functionality you may
//! expect from a basic Create-Read-Update-Delete cycle.
//!
//! Writing an application using `rocket_crud` starts by defining your database
//! schema using [diesel]. Based on this schema and a model struct (which
//! in diesel are normally intended for querying) `rocket_crud` will generate
//! some routes and handlers that you can directly plug into your rocket
//! application.
//!
//! To get started, read the details of the [crud] macro to see some basic
//! usage and an explanation of all parameters.

mod error;
mod filter;
mod sort;

pub mod helper;

#[cfg(feature = "crud-casbin")]
pub mod access_control;

use ::rocket::serde::json::{Json, Value};
use rocket::http::Status;

pub use error::*;
pub use filter::*;
pub use sort::*;

pub use rocket_crud_macros::crud;

pub use either::Either;

/// All generated structs that are used for inserting implement this marker trait.
pub trait CrudInsertableMarker {}

/// All generated structs that are used for updating implement this marker trait.
pub trait CrudUpdatableMarker {}

/// Rocket Crud responses always return a HTTP status code, and then either add
/// a json value to that or a to-be-json encoded struct.
pub type RocketCrudResponse<T> = (Status, Either<Json<T>, Value>);
