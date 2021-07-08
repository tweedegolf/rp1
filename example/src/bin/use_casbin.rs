#[macro_use]
extern crate diesel;

extern crate rocket;

#[path = "../schema.rs"]
mod schema;

use rocket::data::Data;
use rocket::fairing::{Fairing, Info, Kind};
use rocket::request::Request;
use rocket_crud::access_control::EnforcedBy;
use rocket_sync_db_pools::database;

#[database("diesel")]
struct Db(diesel::PgConnection);

#[rocket_crud::crud(database = "Db", table_name = "users")]
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

#[rocket_crud::crud(database = "Db", table_name = "posts")]
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

#[rocket_crud::crud(database = "Db", table_name = "comments")]
#[derive(serde::Serialize, diesel::Queryable)]
struct Comment {
    #[primary_key]
    id: i32,
    content: String,
    #[serde(default)]
    approved: bool,
    post_id: i32,
    #[not_sortable]
    user_id: Option<i32>,
    anonymous_user: Option<String>,
    #[generated]
    created_at: chrono::NaiveDateTime,
    #[generated]
    updated_at: chrono::NaiveDateTime,
}

pub struct AlwaysAdminFairing;

#[rocket::async_trait]
impl Fairing for AlwaysAdminFairing {
    fn info(&self) -> Info {
        Info {
            name: "AlwaysAdminFairing",
            kind: Kind::Request | Kind::Response,
        }
    }

    async fn on_request(&self, request: &mut Request<'_>, _data: &mut Data<'_>) {
        request.local_cache(|| EnforcedBy::Subject("admin".into()));
    }
}

#[rocket::launch]
async fn rocket() -> _ {
    use casbin::{DefaultModel, FileAdapter};

    let m = match DefaultModel::from_file("src/bin/rbac_with_pattern_model.conf").await {
        Ok(m) => m,
        Err(e) => panic!("{:?}", e),
    };

    let a = FileAdapter::new("src/bin/rbac_with_pattern_model.csv");

    let casbin_fairing = match rocket_crud::access_control::PermissionsFairing::new(m, a).await {
        Ok(f) => f,
        Err(e) => panic!("{:?}", e),
    };

    rocket::build()
        .attach(AlwaysAdminFairing)
        .attach(casbin_fairing)
        .mount("/users", User::get_routes())
        .mount("/posts", Post::get_routes())
        .mount("/comments", Comment::get_routes())
        .attach(Db::fairing())
}
