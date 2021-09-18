use std::convert::TryFrom;
use std::io::Write;
use std::ops::{Deref, DerefMut};
use std::str::FromStr;

use diesel::data_types::PgDate;
use diesel::deserialize::FromSql;
use diesel::pg::Pg;
use diesel::serialize::{Output, ToSql};
use diesel::sql_types;
use rocket::form::{FromFormField, ValueField};
use serde::de::Visitor;
use serde::{Deserialize, Deserializer};
use time;
use time::macros::format_description;

/// A Date from [time] wrapper that can be used in diesel, serde and rocket contexts.
/// See [time::Date] for more details on how to use the date itself.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, FromSqlRow, AsExpression)]
#[sql_type = "diesel::sql_types::Date"]
pub struct Date(time::Date);

impl Deref for Date {
    type Target = time::Date;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Date {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl AsRef<time::Date> for Date {
    fn as_ref(&self) -> &time::Date {
        &self.0
    }
}

impl AsMut<time::Date> for Date {
    fn as_mut(&mut self) -> &mut time::Date {
        &mut self.0
    }
}

impl ToSql<sql_types::Date, Pg> for Date {
    fn to_sql<W: Write>(&self, out: &mut Output<W, Pg>) -> diesel::serialize::Result {
        let difference = (self.0 - time::macros::date!(2001 - 01 - 01)).whole_days();
        let postgres_diff = PgDate(i32::try_from(difference)?);
        ToSql::<sql_types::Date, Pg>::to_sql(&postgres_diff, out)
    }
}

impl FromSql<sql_types::Date, Pg> for Date {
    fn from_sql(bytes: Option<&[u8]>) -> diesel::deserialize::Result<Self> {
        let pg_date: PgDate = FromSql::<sql_types::Date, Pg>::from_sql(bytes)?;
        let pg_duration = time::ext::NumericalDuration::days(pg_date.0 as i64);
        Ok(Date(time::macros::date!(2000 - 01 - 01) + pg_duration))
    }
}

impl<'v> FromFormField<'v> for Date {
    fn from_value(field: ValueField<'v>) -> rocket::form::Result<'v, Self> {
        let dt = Self::from_str(field.value)
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send>)?;
        Ok(dt)
    }
}

impl FromStr for Date {
    type Err = time::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        time::Date::parse(s, &format_description!("[year]-[month]-[day]"))
            .map(|p| Date(p))
            .map_err(time::Error::from)
    }
}

impl serde::Serialize for Date {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(
            &self
                .0
                .format(&format_description!("[year]-[month]-[day]"))
                .unwrap(),
        )
    }
}

struct DateVisitor;

impl<'de> Visitor<'de> for DateVisitor {
    type Value = Date;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a date in [year]-[month]-[day] format")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Date::from_str(v).map_err(|e| E::custom(format!("invalid date: {}", e)))
    }

    fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Date::from_str(&v).map_err(|e| E::custom(format!("invalid date: {}", e)))
    }
}

impl<'de> Deserialize<'de> for Date {
    fn deserialize<D>(deserializer: D) -> Result<Date, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(DateVisitor)
    }
}

impl From<time::Date> for Date {
    fn from(d: time::Date) -> Self {
        Self(d)
    }
}

impl Into<time::Date> for Date {
    fn into(self) -> time::Date {
        self.0
    }
}
