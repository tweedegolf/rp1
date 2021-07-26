use rocket::http::ContentType;
use rocket::Build;
use rocket::Rocket;

use rocket_sync_db_pools::database;

#[database("diesel")]
struct Db(diesel::PgConnection);

#[rocket_crud::crud(database = "Db", table_name = "users")]
#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    serde::Deserialize,
    serde::Serialize,
    diesel::Queryable,
    validator::Validate,
)]
struct User {
    #[primary_key]
    pub id: i32,
    #[validate(email)]
    pub username: String,
    #[generated]
    pub created_at: chrono::NaiveDateTime,
    #[generated]
    pub updated_at: chrono::NaiveDateTime,
}

fn init_rocket() -> Rocket<Build> {
    rocket::build()
        .mount("/users", User::get_routes())
        // .mount("/posts", Post::get_routes())
        // .mount("/comments", Comment::get_routes())
        .attach(Db::fairing())
}

use rocket::http::Status;
use rocket::local::blocking::Client;

#[test]
fn create_user_pass() {
    let client = Client::tracked(init_rocket()).expect("valid rocket instance");
    let response = client
        .post("/users/")
        .body(r#"{ "username" : "foo@bar.com" }"#)
        .header(ContentType::JSON)
        .dispatch();

    assert_eq!(response.status(), Status::Ok);

    let new_user = response.into_json::<User>().unwrap();

    assert_eq!(new_user.username, "foo@bar.com");
}

#[test]
fn create_user_fail() {
    let client = Client::tracked(init_rocket()).expect("valid rocket instance");
    let response = client
        .post("/users/")
        .body(r#"{ "username" : "foobar" }"#)
        .header(ContentType::JSON)
        .dispatch();

    assert_eq!(response.status(), Status::BadRequest);
}
