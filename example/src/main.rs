#[macro_use]
extern crate diesel;

mod schema;

use diesel::backend::Backend;
use rocket_sync_db_pools::database;
use rp1::{
    access_control::{CheckPermissions, PermissionFilter},
    CrudStruct,
};

impl CheckPermissions for User {
    type AuthUser = AUser;
}

impl CheckPermissions for Post {
    type AuthUser = AUser;

    fn filter_list<DB>(u: &Self::AuthUser) -> PermissionFilter<<Self as CrudStruct>::TableType, DB>
    where
        DB: Backend,
    {
        use crate::schema::posts::dsl::*;
        use diesel::prelude::*;
        match u {
            AUser::Anonymous => PermissionFilter::KeepNone,
            AUser::LoggedIn(u) => PermissionFilter::Filter(Box::new(user_id.eq(u.id))),
        }
    }
}

impl CheckPermissions for Comment {
    type AuthUser = AUser;
}

#[database("diesel")]
struct Db(diesel::PgConnection);

#[rp1::crud(database = "Db", table_name = "users")]
#[derive(Debug, serde::Serialize, diesel::Queryable, validator::Validate)]
struct User {
    #[primary_key]
    pub id: i32,
    #[validate(email)]
    username: String,
    role: String,
    #[generated]
    created_at: chrono::NaiveDateTime,
    #[generated]
    updated_at: chrono::NaiveDateTime,
}

#[rp1::crud(database = "Db", table_name = "posts")]
#[derive(Debug, serde::Serialize, diesel::Queryable)]
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

#[rp1::crud(database = "Db", table_name = "comments")]
#[derive(Debug, serde::Serialize, diesel::Queryable)]
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

pub enum AUser {
    LoggedIn(User),
    Anonymous,
}

use rocket::request::{FromRequest, Outcome, Request};

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
