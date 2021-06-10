#[macro_use]
extern crate diesel;

extern crate rocket;

mod schema;

use rocket_sync_db_pools::database;

use schema::foo;

#[database("diesel")]
struct Db(diesel::PgConnection);

#[rocket_crud::crud(database = "Db", schema = "schema", delete = false, table_name = "foo")]
#[derive(serde::Serialize, diesel::Queryable)]
struct CruddedFoo {
    #[primary_key]
    id: i32,
    name: String,
    // other: String,
}

#[rocket::launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/", CruddedFoo::get_routes())
        .attach(Db::fairing())
}
