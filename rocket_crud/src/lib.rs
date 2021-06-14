pub use rocket_crud_macros::crud;

pub trait CrudInsertableMarker {}

pub trait CrudUpdatableMarker {}

pub use either::Either;

#[derive(serde::Deserialize, serde::Serialize, rocket::FromFormField, Debug)]
pub enum SortDirection {
  Asc,
  Desc,
}

#[derive(Debug)]
pub struct SortSpec<T> {
  pub field: T,
  pub direction: SortDirection,
}

impl<'v, T: rocket::form::FromFormField<'v>> SortSpec<T> {
  fn from_str(mut data: &'v str, name: &rocket::form::name::NameView<'v>) -> rocket::form::Result<'v, SortSpec<T>> {
    let mut direction = SortDirection::Asc;
    if data.chars().nth(0) == Some('-') {
      data = &data[1..];
      direction = SortDirection::Desc;
    } else if data.chars().nth(0) == Some('+') {
      data = &data[1..];
    }

    let field = rocket::form::FromFormField::from_value(rocket::form::ValueField {
      name: name.clone(),
      value: data,
    })?;

    Ok(SortSpec {
      field,
      direction,
    })
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
          Err((None, Some(limit)))?;
      }
      let bytes = bytes.into_inner();
      let bytes = rocket::request::local_cache!(field.request, bytes);
      let data = std::str::from_utf8(bytes)?;

      SortSpec::from_str(data, &field.name)
    }

    fn default() -> Option<Self> { None }
}
