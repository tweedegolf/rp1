use std::fmt::{self, Display, Formatter};
use std::str::from_utf8;

use rocket::data::ToByteUnit;
use rocket::form::{self, name::NameView, DataField, FromFormField, ValueField};

/// Specifies in which direction a sorting operation should occur.
///
/// See the docs for [SortSpec] for a detailed description of the query syntax.
#[derive(serde::Deserialize, serde::Serialize, rocket::FromFormField, Debug, PartialEq, Eq)]
pub enum SortDirection {
    /// Sort the field in ascending order.
    #[serde(rename = "asc")]
    Asc,
    /// Sort the field in descending order.
    #[serde(rename = "desc")]
    Desc,
}

/// Sort specification, contains a field and direction.
///
/// A field enum is generated based on the fields in the struct.
#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct SortSpec<T> {
    pub field: T,
    pub direction: SortDirection,
}

impl<T: Display> Display for SortSpec<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if self.direction == SortDirection::Desc {
            write!(f, "-")?;
        }
        write!(f, "{}", self.field)
    }
}

impl<'v, T: FromFormField<'v>> SortSpec<T> {
    fn from_str(mut data: &'v str, name: &NameView<'v>) -> form::Result<'v, SortSpec<T>> {
        let mut direction = SortDirection::Asc;
        if data.starts_with('-') {
            data = data.trim_start_matches('-');
            direction = SortDirection::Desc;
        } else if data.starts_with('+') {
            data = data.trim_start_matches('+');
        }

        // in practice, a `sort=+username` will give us a data that starts with a space ` username`
        data = data.trim_start_matches(' ');

        let field = FromFormField::from_value(ValueField {
            name: *name,
            value: data,
        })?;

        Ok(SortSpec { field, direction })
    }
}

#[rocket::async_trait]
impl<'v, T: FromFormField<'v>> FromFormField<'v> for SortSpec<T> {
    fn from_value(field: ValueField<'v>) -> form::Result<'v, Self> {
        SortSpec::from_str(field.value, &field.name)
    }

    async fn from_data(field: DataField<'v, '_>) -> form::Result<'v, Self> {
        let limit = 256.kibibytes();
        let bytes = field.data.open(limit).into_bytes().await?;
        if !bytes.is_complete() {
            return Err((None, Some(limit)).into());
        }
        let bytes = bytes.into_inner();
        let bytes = rocket::request::local_cache!(field.request, bytes);
        let data = from_utf8(bytes)?;

        SortSpec::from_str(data, &field.name)
    }

    fn default() -> Option<Self> {
        None
    }
}
