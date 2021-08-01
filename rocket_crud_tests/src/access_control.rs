use rocket::http::ContentType;
use rocket::http::Header;
use rocket::request::FromRequest;
use rocket::request::Outcome;
use rocket::request::Request;
use rocket::Build;
use rocket::Rocket;

use rocket_crud::CheckPermissions;
use rocket_sync_db_pools::database;

#[database("diesel")]
struct Db(diesel::PgConnection);

pub enum AUser {
    LoggedIn(User),
    Anonymous,
}

#[rocket_crud::crud(database = "Db", table_name = "users", auth = false)]
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

#[rocket_crud::crud(database = "Db", table_name = "posts", auth = false)]
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

#[rocket_crud::crud(database = "Db", table_name = "comments", auth = false)]
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

impl CheckPermissions for User {
    type AuthUser = AUser;
}

impl CheckPermissions for Post {
    type AuthUser = AUser;
}

impl CheckPermissions for Comment {
    type AuthUser = AUser;
}

#[derive(std::hash::Hash, serde::Serialize, Debug)]
struct AuthUser {
    id: i32,
    role: String,
}

async fn init_rocket() -> Rocket<Build> {
    rocket::build()
        .mount("/users", User::get_routes())
        .mount("/posts", Post::get_routes())
        .mount("/comments", Comment::get_routes())
        .attach(Db::fairing())
}

use rocket::http::Status;
use rocket::local::asynchronous::Client;

#[tokio::test]
async fn create_user_fail() {
    let client = Client::tracked(init_rocket().await)
        .await
        .expect("valid rocket instance");

    let id = Header::new("X-Auth-Id", "1");
    let role = Header::new("X-Auth-Role", "not-admin");
    let response = client
        .post("/users")
        .body(r#"{ "username" : "foobar@example.com" }"#)
        .header(ContentType::JSON)
        .header(id)
        .header(role)
        .dispatch()
        .await;

    assert_eq!(response.status(), Status::Forbidden);
}

#[tokio::test]
async fn create_user_pass() {
    let client = Client::tracked(init_rocket().await)
        .await
        .expect("valid rocket instance");

    let id = Header::new("X-Auth-Id", "1");
    let role = Header::new("X-Auth-Role", "admin");
    let response = client
        .post("/users")
        .body(r#"{ "username" : "foobar@example.com" }"#)
        .header(ContentType::JSON)
        .header(id)
        .header(role)
        .dispatch()
        .await;

    assert_eq!(response.status(), Status::Ok);
}

#[tokio::test]
async fn create_post_fail() {
    let client = Client::tracked(init_rocket().await)
        .await
        .expect("valid rocket instance");

    let id = Header::new("X-Auth-Id", "82");
    let role = Header::new("X-Auth-Role", "not-poster");
    let response = client
        .post("/posts")
        .body(r#"{ "title": "Bla", "content" : "Blablabla", "user_id": 81 }"#)
        .header(ContentType::JSON)
        .header(id)
        .header(role)
        .dispatch()
        .await;

    assert_eq!(response.status(), Status::Forbidden);
}

#[tokio::test]
async fn create_post_pass() {
    let client = Client::tracked(init_rocket().await)
        .await
        .expect("valid rocket instance");

    let id = Header::new("X-Auth-Id", "81");
    let role = Header::new("X-Auth-Role", "poster");
    let response = client
        .post("/posts")
        .body(r#"{ "title": "Bla", "content" : "Blablabla", "user_id": 81 }"#)
        .header(ContentType::JSON)
        .header(id)
        .header(role)
        .dispatch()
        .await;

    assert_eq!(response.status(), Status::Ok);
}


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
                        crate::schema::users::table
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
