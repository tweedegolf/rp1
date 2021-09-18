use std::io::Write;
use std::ops::{Deref, DerefMut};
use std::str::FromStr;

use diesel::data_types::PgTime;
use diesel::deserialize::FromSql;
use diesel::pg::Pg;
use diesel::serialize::{Output, ToSql};
use diesel::sql_types;
use rocket::form::{FromFormField, ValueField};
use serde::de::Visitor;
use serde::{Deserialize, Deserializer};
use time;
use time::macros::format_description;

/// A Time from [time] wrapper that can be used in diesel, serde and rocket contexts.
/// See [time::Time] for more details on how to use the time itself.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, FromSqlRow, AsExpression)]
#[sql_type = "diesel::sql_types::Time"]
pub struct Time(time::Time);

impl Deref for Time {
    type Target = time::Time;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Time {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl AsRef<time::Time> for Time {
    fn as_ref(&self) -> &time::Time {
        &self.0
    }
}

impl AsMut<time::Time> for Time {
    fn as_mut(&mut self) -> &mut time::Time {
        &mut self.0
    }
}

impl ToSql<sql_types::Time, Pg> for Time {
    fn to_sql<W: Write>(&self, out: &mut Output<W, Pg>) -> diesel::serialize::Result {
        let micros = self.0.hour() as i64 * 3600 * 1_000_000
            + self.0.minute() as i64 * 60 * 1_000_000
            + self.0.second() as i64 * 1_000_000
            + self.0.microsecond() as i64;
        let postgres_diff = PgTime(micros);
        ToSql::<sql_types::Time, Pg>::to_sql(&postgres_diff, out)
    }
}

impl FromSql<sql_types::Time, Pg> for Time {
    fn from_sql(bytes: Option<&[u8]>) -> diesel::deserialize::Result<Self> {
        use time::ext::NumericalDuration;
        let pg_time: PgTime = FromSql::<sql_types::Time, Pg>::from_sql(bytes)?;
        let pg_duration = pg_time.0.microseconds();
        Ok(Time(time::Time::MIDNIGHT + pg_duration))
    }
}

impl<'v> FromFormField<'v> for Time {
    fn from_value(field: ValueField<'v>) -> rocket::form::Result<'v, Self> {
        let dt = Self::from_str(field.value)
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send>)?;
        Ok(dt)
    }
}

impl FromStr for Time {
    type Err = time::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        time::Time::parse(
            s,
            &format_description!("[hour]:[minute]:[second].[subsecond]"),
        )
        .or_else(|_| time::Time::parse(s, &format_description!("[hour]:[minute]:[second]")))
        .or_else(|_| time::Time::parse(s, &format_description!("[hour]:[minute]")))
        .map(|p| Time(p))
        .map_err(time::Error::from)
    }
}

impl serde::Serialize for Time {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        if self.microsecond() == 0 {
            serializer.serialize_str(
                &self
                    .0
                    .format(&format_description!("[hour]:[minute]:[second]"))
                    .unwrap(),
            )
        } else {
            serializer.serialize_str(
                &self
                    .0
                    .format(&format_description!("[hour]:[minute]:[second].[subsecond]"))
                    .unwrap(),
            )
        }
    }
}

struct TimeVisitor;

impl<'de> Visitor<'de> for TimeVisitor {
    type Value = Time;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a time in [hour]-[minute](-[second](.[subsecond])?)? format")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Time::from_str(v).map_err(|e| E::custom(format!("invalid time: {}", e)))
    }

    fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Time::from_str(&v).map_err(|e| E::custom(format!("invalid time: {}", e)))
    }
}

impl<'de> Deserialize<'de> for Time {
    fn deserialize<D>(deserializer: D) -> Result<Time, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(TimeVisitor)
    }
}

impl From<time::Time> for Time {
    fn from(t: time::Time) -> Self {
        Self(t)
    }
}

impl Into<time::Time> for Time {
    fn into(self) -> time::Time {
        self.0
    }
}
