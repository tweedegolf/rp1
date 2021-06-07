#![feature(extended_key_value_attributes)]
#[macro_use]
extern crate diesel;

extern crate rocket;

mod schema;
use schema::*;

use diesel::prelude::*;

use rocket_sync_db_pools::database;

#[database("diesel")]
struct Db(diesel::PgConnection);

// #[crud_db = Db]
#[derive(
    serde::Serialize, Queryable, Insertable, rocket_crud::CrudInsertable, rocket_crud::CrudCreate,
)]
#[table_name = "foo"]
struct CruddedFoo {
    #[primary_key]
    #[serde(rename = "foo")]
    id: i32,
    name: String,
    other: String,
}

#[derive(Queryable, Insertable)]
#[table_name = "foo"]
struct Foo {
    id: i32,

    name: String,
    other: String,
}

#[derive(Insertable)]
#[table_name = "foo"]
struct NewFoo {
    id: Option<i32>,
    name: String,
    other: String,
}

#[allow(dead_code)]
async fn create<T>(db: Db, value: NewCruddedFoo) -> Foo {
    db.run(move |conn| {
        diesel::insert_into(schema::foo::table)
            .values(&value)
            .get_result(conn)
    })
    .await
    .unwrap()
}

pub fn main() {
    println!("hello world");
}
