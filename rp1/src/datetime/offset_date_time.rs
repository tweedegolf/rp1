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
use serde::de::Visitor;
use serde::{Deserialize, Deserializer};
use time;
use time::format_description::well_known::Rfc3339;
use time::macros::datetime;

/// An OffsetDateTime from [time] wrapper that can be used in diesel, serde and rocket contexts.
/// See [time::OffsetDateTime] for more details on how to use the datetime itself.
///
/// Note that you should prefer an `OffsetDateTime` over a [super::PrimitiveDateTime].
/// This type can only be implemented on an SQL column that stores timezone information.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, FromSqlRow, AsExpression)]
#[sql_type = "diesel::sql_types::Timestamptz"]
pub struct OffsetDateTime(time::OffsetDateTime);

impl Deref for OffsetDateTime {
    type Target = time::OffsetDateTime;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for OffsetDateTime {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl AsRef<time::OffsetDateTime> for OffsetDateTime {
    fn as_ref(&self) -> &time::OffsetDateTime {
        &self.0
    }
}

impl AsMut<time::OffsetDateTime> for OffsetDateTime {
    fn as_mut(&mut self) -> &mut time::OffsetDateTime {
        &mut self.0
    }
}

impl ToSql<sql_types::Timestamptz, Pg> for OffsetDateTime {
    fn to_sql<W: Write>(&self, out: &mut Output<W, Pg>) -> diesel::serialize::Result {
        let difference = (self.0 - datetime!(2000-01-01 00:00:00 UTC)).whole_microseconds();
        let postgres_diff = PgTimestamp(i64::try_from(difference)?);
        ToSql::<sql_types::Timestamp, Pg>::to_sql(&postgres_diff, out)
    }
}

impl FromSql<sql_types::Timestamptz, Pg> for OffsetDateTime {
    fn from_sql(bytes: Option<&[u8]>) -> diesel::deserialize::Result<Self> {
        use time::ext::NumericalDuration;
        let pg_timestamp: PgTimestamp = FromSql::<sql_types::Timestamp, Pg>::from_sql(bytes)?;
        let pg_duration = pg_timestamp.0.microseconds();
        Ok(OffsetDateTime(
            (datetime!(2000-01-01 00:00:00 UTC) + pg_duration),
        ))
    }
}

impl<'v> FromFormField<'v> for OffsetDateTime {
    fn from_value(field: ValueField<'v>) -> rocket::form::Result<'v, Self> {
        let dt = Self::from_str(field.value)
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send>)?;
        Ok(dt)
    }
}

impl FromStr for OffsetDateTime {
    type Err = time::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        time::OffsetDateTime::parse(s, &Rfc3339)
            .map(|p| OffsetDateTime(p))
            .map_err(time::Error::from)
    }
}

impl serde::Serialize for OffsetDateTime {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.0.format(&Rfc3339).unwrap())
    }
}

struct OffsetDateTimeVisitor;

impl<'de> Visitor<'de> for OffsetDateTimeVisitor {
    type Value = OffsetDateTime;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a datetime in RFC3339 format")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        OffsetDateTime::from_str(v).map_err(|e| E::custom(format!("invalid datetime: {}", e)))
    }

    fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        OffsetDateTime::from_str(&v).map_err(|e| E::custom(format!("invalid datetime: {}", e)))
    }
}

impl<'de> Deserialize<'de> for OffsetDateTime {
    fn deserialize<D>(deserializer: D) -> Result<OffsetDateTime, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(OffsetDateTimeVisitor)
    }
}

impl From<time::OffsetDateTime> for OffsetDateTime {
    fn from(d: time::OffsetDateTime) -> Self {
        Self(d)
    }
}

impl Into<time::OffsetDateTime> for OffsetDateTime {
    fn into(self) -> time::OffsetDateTime {
        self.0
    }
}
