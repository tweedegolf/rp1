//! Contains some utilities mostly useful internally, but may be used in other
//! places a well.
use serde::de::{Deserialize, Deserializer};

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
