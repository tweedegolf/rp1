#[macro_use]
extern crate diesel;

extern crate rocket;

mod schema;

use rocket_sync_db_pools::database;

#[database("diesel")]
struct Db(diesel::PgConnection);

#[rocket_crud::crud]
#[derive(serde::Serialize, Queryable, Insertable)]
#[table_name = "foo"]
struct CruddedFoo {
    #[primary_key]
    #[serde(rename = "foo")]
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
