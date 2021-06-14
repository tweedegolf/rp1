#[macro_use]
extern crate diesel;

extern crate rocket;

mod schema;

use rocket_sync_db_pools::database;

#[database("diesel")]
struct Db(diesel::PgConnection);

#[rocket_crud::crud(database = "Db", delete = false, table_name = "users")]
#[derive(serde::Serialize, diesel::Queryable)]
struct User {
    #[primary_key]
    id: i32,
    username: String,
    #[generated]
    created_at: chrono::NaiveDateTime,
    #[generated]
    updated_at: chrono::NaiveDateTime,
}

#[rocket_crud::crud(database = "Db", delete = false, table_name = "posts")]
#[derive(serde::Serialize, diesel::Queryable)]
struct Post {
    #[primary_key]
    id: i32,
    title: String,
    subtitle: Option<String>,
    content: String,
    user_id: i32,
    #[generated]
    created_at: chrono::NaiveDateTime,
    #[generated]
    updated_at: chrono::NaiveDateTime,
}

#[rocket_crud::crud(database = "Db", delete = false, table_name = "comments")]
#[derive(serde::Serialize, diesel::Queryable)]
struct Comment {
    #[primary_key]
    id: i32,
    content: String,
    #[serde(default)]
    approved: bool,
    post_id: i32,
    user_id: Option<i32>,
    anonymous_user: Option<String>,
    #[generated]
    created_at: chrono::NaiveDateTime,
    #[generated]
    updated_at: chrono::NaiveDateTime,
}

#[rocket::launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/users", User::get_routes())
        .mount("/posts", Post::get_routes())
        .mount("/comments", Comment::get_routes())
        .attach(Db::fairing())
}
