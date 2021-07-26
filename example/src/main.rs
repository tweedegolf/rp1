#[macro_use]
extern crate diesel;

extern crate rocket;

mod schema;

use rocket_crud::access_control::CheckPermissions;
use rocket_sync_db_pools::database;

fn baz(u: User) -> <User as CheckPermissions>::AuthUser {
    AUser::Anonymous
}

impl CheckPermissions for self::user::User {
    type AuthUser = AUser;
}

impl CheckPermissions for Post {
    type AuthUser = AUser;
}

impl CheckPermissions for Comment {
    type AuthUser = AUser;
}

#[database("diesel")]
struct Db(diesel::PgConnection);

#[rocket_crud::crud(database = "Db", table_name = "users")]
#[derive(serde::Serialize, diesel::Queryable, validator::Validate)]
struct User {
    #[primary_key]
    id: i32,
    #[validate(email)]
    username: String,
    role: String,
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

#[rocket::launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/users", User::get_routes())
        .mount("/posts", Post::get_routes())
        .mount("/comments", Comment::get_routes())
        .attach(Db::fairing())
}

#[::rocket::get("/foo/<id>")]
async fn read_fn(
    db: Db,
    // auth_user: <#ident as ::rocket_crud::access_control::CheckPermissions>::AuthUser,
    id: i32,
) -> ::rocket_crud::RocketCrudResponse<User> {
    use ::diesel::prelude::*;

    use ::rocket_crud::access_control::CheckPermissions;
    use ::rocket_crud::helper::{db_error_to_response, ok_to_response};

    let auth_user = AUser::Anonymous;

    let db_result = db
        .run(move |conn| schema::users::table.find(id).first::<User>(conn))
        .await;

    match db_result {
        Err(e) => db_error_to_response(e),
        Ok(user) => {
            if <User as CheckPermissions>::allow_read(&user, &auth_user) {
                ok_to_response(user)
            } else {
                panic!()
            }
        }
    }
}

pub enum AUser {
    LoggedIn(User),
    Anonymous,
}

use rocket::{
    http::uri::Authority,
    request::{FromRequest, Outcome, Request},
};

#[rocket::async_trait]
impl<'r> FromRequest<'r> for AUser {
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

                Outcome::Success(AUser::LoggedIn(user))
            }
            None => Outcome::Success(AUser::Anonymous),
        }
    }
}
