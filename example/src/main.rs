#[macro_use]
extern crate diesel;

extern crate rocket;

mod schema;

use rocket_sync_db_pools::database;

#[database("diesel")]
struct Db(diesel::PgConnection);

#[rocket_crud::crud(database = "Db", table_name = "users", ignore_casbin = true)]
#[derive(serde::Serialize, diesel::Queryable, validator::Validate)]
struct User {
    #[primary_key]
    id: i32,
    #[validate(email)]
    username: String,
    #[generated]
    created_at: chrono::NaiveDateTime,
    #[generated]
    updated_at: chrono::NaiveDateTime,
}

#[rocket_crud::crud(database = "Db", table_name = "posts", ignore_casbin = true)]
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

#[rocket_crud::crud(database = "Db", table_name = "comments", ignore_casbin = true)]
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

#[rocket::launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/users", User::get_routes())
        .mount("/posts", Post::get_routes())
        .mount("/comments", Comment::get_routes())
        .attach(Db::fairing())
}

enum AuthUser {
    LoggedIn(User),
    Anonymous,
}

use rocket::request::{FromRequest, Outcome, Request};

#[rocket::async_trait]
impl<'r> FromRequest<'r> for AuthUser {
    type Error = std::convert::Infallible;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        use diesel::prelude::*;

        match req.headers().get_one("X-UNSAFE-USER-ID") {
            Some(user_id_str) => {
                let db = <Db as FromRequest>::from_request(req).await.unwrap();

                let user_id: i32 = user_id_str.parse().unwrap();

                let user: User = db
                    .run(move |conn| {
                        schema::users::table
                            .find(user_id)
                            .first::<User>(conn)
                            .unwrap()
                    })
                    .await;

                Outcome::Success(AuthUser::LoggedIn(user))
            }
            None => Outcome::Success(AuthUser::Anonymous),
        }
    }
}
