use std::convert::TryFrom;
use std::io::Write;
use std::ops::{Deref, DerefMut};
use std::str::FromStr;

use diesel::data_types::PgTimestamp;
use diesel::deserialize::FromSql;
use diesel::pg::Pg;
use diesel::serialize::{Output, ToSql};
use diesel::sql_types;
use rocket::form::{FromFormField, ValueField};
use time;
use time::format_description::well_known::Rfc3339;
use time::macros::datetime;

/// A PrimitiveDateTime from [time] wrapper that can be used in diesel, serde and rocket contexts.
/// See [time::PrimitiveDateTime] for more details on how to use the datetime itself.
///
/// Note that you should prefer an `PrimitiveDateTime` over a [super::OffsetDateTime].
#[derive(serde::Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, FromSqlRow, AsExpression)]
#[sql_type = "diesel::sql_types::Timestamp"]
#[sql_type = "diesel::sql_types::Timestamptz"]
pub struct PrimitiveDateTime(time::PrimitiveDateTime);

impl Deref for PrimitiveDateTime {
    type Target = time::PrimitiveDateTime;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for PrimitiveDateTime {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl AsRef<time::PrimitiveDateTime> for PrimitiveDateTime {
    fn as_ref(&self) -> &time::PrimitiveDateTime {
        &self.0
    }
}

impl AsMut<time::PrimitiveDateTime> for PrimitiveDateTime {
    fn as_mut(&mut self) -> &mut time::PrimitiveDateTime {
        &mut self.0
    }
}

impl ToSql<sql_types::Timestamp, Pg> for PrimitiveDateTime {
    fn to_sql<W: Write>(&self, out: &mut Output<W, Pg>) -> diesel::serialize::Result {
        let difference = (self.0 - datetime!(2000-01-01 00:00:00)).whole_microseconds();
        let postgres_diff = PgTimestamp(i64::try_from(difference)?);
        ToSql::<sql_types::Timestamp, Pg>::to_sql(&postgres_diff, out)
    }
}

impl FromSql<sql_types::Timestamp, Pg> for PrimitiveDateTime {
    fn from_sql(bytes: Option<&[u8]>) -> diesel::deserialize::Result<Self> {
        use time::ext::NumericalDuration;
        let pg_timestamp: PgTimestamp = FromSql::<sql_types::Timestamp, Pg>::from_sql(bytes)?;
        let pg_duration = pg_timestamp.0.microseconds();
        Ok(PrimitiveDateTime(
            datetime!(2000-01-01 00:00:00) + pg_duration,
        ))
    }
}

impl<'v> FromFormField<'v> for PrimitiveDateTime {
    fn from_value(field: ValueField<'v>) -> rocket::form::Result<'v, Self> {
        let dt = Self::from_str(field.value)
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send>)?;
        Ok(dt)
    }
}

impl FromStr for PrimitiveDateTime {
    type Err = time::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        time::PrimitiveDateTime::parse(s, &Rfc3339)
            .map(|p| PrimitiveDateTime(p))
            .map_err(time::Error::from)
    }
}

impl serde::Serialize for PrimitiveDateTime {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.0.format(&Rfc3339).unwrap())
    }
}

impl From<time::PrimitiveDateTime> for PrimitiveDateTime {
    fn from(d: time::PrimitiveDateTime) -> Self {
        Self(d)
    }
}

impl Into<time::PrimitiveDateTime> for PrimitiveDateTime {
    fn into(self) -> time::PrimitiveDateTime {
        self.0
    }
}
