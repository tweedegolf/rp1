#![doc = include_str!("../docs/lib.md")]

#[macro_use]
extern crate diesel;

mod error;
mod filter;
mod sort;
pub mod datetime;

pub mod helper;

pub mod access_control;

use ::rocket::serde::json::Json;

pub use access_control::*;
pub use error::*;
pub use filter::*;
pub use sort::*;

pub use rp1_macros::crud;

/// This trait is implemented on the main struct and indicates the type that
/// is the diesel table struct.
pub trait CrudStruct {
    type TableType;
}

/// This trait is implemented on the main struct and indicates the type that
/// is used for insert operations.
///
/// Note that if the `create` action is disabled this type will be implemented
/// with a unit (`()`) insert type.
pub trait CrudInsertable {
    type InsertType;
}

/// This trait is implemented on the main struct and indicates the type that
/// is used for update operations.
///
/// Note that if the `update` action is disabled this type will be implemented
/// with a unit (`()`) update type.
pub trait CrudUpdatable {
    type UpdateType;
}

/// This trait is implemented on the main struct and indicates the type that
/// is used for filtering on properties.
pub trait CrudFilterSpec {
    type FilterSpecType;
}

/// Return type for all handler functions generated by the [crud] macro.
pub type CrudResult<T, E = crate::CrudError> = Result<T, E>;

/// Return type for all handler functions that return json generated by the
/// [crud] macro.
pub type CrudJsonResult<T, E = crate::CrudError> = Result<Json<T>, E>;
