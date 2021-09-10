use std::ops::{Deref, DerefMut};
use std::io::Write;
use std::str::FromStr;
use std::convert::TryFrom;

use diesel::data_types::{PgDate, PgTime, PgTimestamp};
use diesel::deserialize::FromSql;
use diesel::serialize::{ToSql, Output};
use diesel::sql_types;
use diesel::pg::Pg;
use rocket::form::{FromFormField, ValueField};
use time;
use time::format_description::well_known::Rfc3339;

#[derive(serde::Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, FromSqlRow, AsExpression)]
#[sql_type = "diesel::sql_types::Timestamp"]
pub struct PrimitiveDateTime(time::PrimitiveDateTime);

fn pg_epoch() -> time::PrimitiveDateTime {
    time::macros::datetime!(2000-01-01 00:00:00)
}

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

impl From<time::PrimitiveDateTime> for PrimitiveDateTime {
    fn from(v: time::PrimitiveDateTime) -> Self {
        PrimitiveDateTime(v)
    }
}

impl ToSql<sql_types::Timestamp, Pg> for PrimitiveDateTime {
    fn to_sql<W: Write>(&self, out: &mut Output<W, Pg>) -> diesel::serialize::Result {
        let difference = (self.0 - pg_epoch()).whole_microseconds();
        let postgres_diff = PgTimestamp(i64::try_from(difference)?);
        ToSql::<sql_types::Timestamp, Pg>::to_sql(&postgres_diff, out)
    }
}

impl FromSql<sql_types::Timestamp, Pg> for PrimitiveDateTime {
    fn from_sql(bytes: Option<&[u8]>) -> diesel::deserialize::Result<Self> {
        use time::ext::NumericalDuration;
        let pg_timestamp: PgTimestamp = FromSql::<sql_types::Timestamp, Pg>::from_sql(bytes)?;
        let pg_duration = pg_timestamp.0.microseconds();
        Ok(PrimitiveDateTime(pg_epoch() + pg_duration))
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
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: serde::Serializer {
        serializer.serialize_str(&self.0.format(&Rfc3339).unwrap())
    }
}

#[derive(serde::Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, FromSqlRow, AsExpression)]
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
        let difference = (self.0 - pg_epoch().assume_utc()).whole_microseconds();
        let postgres_diff = PgTimestamp(i64::try_from(difference)?);
        ToSql::<sql_types::Timestamp, Pg>::to_sql(&postgres_diff, out)
    }
}

impl FromSql<sql_types::Timestamptz, Pg> for OffsetDateTime {
    fn from_sql(bytes: Option<&[u8]>) -> diesel::deserialize::Result<Self> {
        use time::ext::NumericalDuration;
        let pg_timestamp: PgTimestamp = FromSql::<sql_types::Timestamp, Pg>::from_sql(bytes)?;
        let pg_duration = pg_timestamp.0.microseconds();
        Ok(OffsetDateTime((pg_epoch() + pg_duration).assume_utc()))
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
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: serde::Serializer {
        serializer.serialize_str(&self.0.format(&Rfc3339).unwrap())
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, FromSqlRow, AsExpression)]
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
        let micros = self.0.hour() as i64 * 3600 * 1_000_000 +
            self.0.minute() as i64 * 60 * 1_000_000 +
            self.0.second() as i64 * 1_000_000 +
            self.0.microsecond() as i64;
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
        time::Time::parse(s, &Rfc3339)
            .map(|p| Time(p))
            .map_err(time::Error::from)
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, FromSqlRow, AsExpression)]
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
        let difference = (self.0 - time::macros::date!(2001-01-01)).whole_days();
        let postgres_diff = PgDate(i32::try_from(difference)?);
        ToSql::<sql_types::Date, Pg>::to_sql(&postgres_diff, out)
    }
}

impl FromSql<sql_types::Date, Pg> for Date {
    fn from_sql(bytes: Option<&[u8]>) -> diesel::deserialize::Result<Self> {
        let pg_date: PgDate = FromSql::<sql_types::Date, Pg>::from_sql(bytes)?;
        let pg_duration = time::ext::NumericalDuration::days(pg_date.0 as i64);
        Ok(Date(time::macros::date!(2000-01-01) + pg_duration))
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
        time::Date::parse(s, &Rfc3339)
            .map(|p| Date(p))
            .map_err(time::Error::from)
    }
}
