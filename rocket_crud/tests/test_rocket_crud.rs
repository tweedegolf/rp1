#![cfg(test)]
// #[macro_use]
// extern crate pretty_assertions;
#[macro_use]
extern crate diesel;

extern crate rocket;

use rocket::http::ContentType;

// #[macro_use]
// use rocket_crud;

mod schema {

    table! {
        comments (id) {
            id -> Int4,
            content -> Text,
            approved -> Bool,
            post_id -> Int4,
            user_id -> Nullable<Int4>,
            anonymous_user -> Nullable<Varchar>,
            created_at -> Timestamp,
            updated_at -> Timestamp,
        }
    }

    table! {
        foo (id) {
            id -> Int4,
            name -> Text,
        }
    }

    table! {
        posts (id) {
            id -> Int4,
            title -> Varchar,
            subtitle -> Nullable<Varchar>,
            content -> Text,
            user_id -> Int4,
            created_at -> Timestamp,
            updated_at -> Timestamp,
        }
    }

    table! {
        users (id) {
            id -> Int4,
            username -> Varchar,
            created_at -> Timestamp,
            updated_at -> Timestamp,
        }
    }

    joinable!(comments -> posts (post_id));
    joinable!(comments -> users (user_id));
    joinable!(posts -> users (user_id));

    allow_tables_to_appear_in_same_query!(comments, foo, posts, users,);
}

use rocket_sync_db_pools::database;

#[database("diesel")]
struct Db(diesel::PgConnection);

#[rocket_crud::crud(database = "Db", table_name = "users")]
#[derive(Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize, diesel::Queryable)]
struct User {
    #[primary_key]
    pub id: i32,
    pub username: String,
    #[generated]
    pub created_at: chrono::NaiveDateTime,
    #[generated]
    pub updated_at: chrono::NaiveDateTime,
}

#[rocket::launch]
fn init_rocket() -> _ {
    rocket::build()
        .mount("/users", User::get_routes())
        // .mount("/posts", Post::get_routes())
        // .mount("/comments", Comment::get_routes())
        .attach(Db::fairing())
}

use rocket::http::Status;
use rocket::local::blocking::Client;

#[test]
fn test1() {
    let client = Client::tracked(init_rocket()).expect("valid rocket instance");
    let response = client.get("/users").dispatch();
    assert_eq!(response.status(), Status::Ok);

    assert_eq!(
        response.into_json::<Vec<User>>().unwrap(),
        vec![],
        /*(
        User {
            id: 0,
            username: String::new(),
            created_at: chrono::NaiveDateTime::from_timestamp(0, 0),
            updated_at: chrono::NaiveDateTime::from_timestamp(0, 0)
        }
        */
    );
}

#[test]
fn create_user_json() {
    let client = Client::tracked(init_rocket()).expect("valid rocket instance");
    let response = client
        .post("/users/")
        .body(r#"{ "username" : "foobar" }"#)
        .header(ContentType::JSON)
        .dispatch();

    assert_eq!(response.status(), Status::Ok);

    let new_user = response.into_json::<User>().unwrap();

    assert_eq!(new_user.username, "foobar");
}

#[test]
fn create_user_form() {
    let client = Client::tracked(init_rocket()).expect("valid rocket instance");
    let response = client
        .post("/users/form")
        .body("username=foobar")
        .header(ContentType::Form)
        .dispatch();

    assert_eq!(response.status(), Status::Ok);

    let new_user = response.into_json::<User>().unwrap();

    assert_eq!(new_user.username, "foobar");
}

#[test]
fn update_user_json() {
    let client = Client::tracked(init_rocket()).expect("valid rocket instance");
    let response = client
        .post("/users/")
        .body(r#"{ "username" : "foobar" }"#)
        .header(ContentType::JSON)
        .dispatch();

    assert_eq!(response.status(), Status::Ok);

    let new_user = response.into_json::<User>().unwrap();

    assert_eq!(new_user.username, "foobar");

    let response = client
        .patch(format!("/users/{}", new_user.id))
        .body(r#"{ "username" : "baz" }"#)
        .header(ContentType::JSON)
        .dispatch();

    let newer_user = response.into_json::<User>().unwrap();

    assert_eq!(newer_user.username, "baz");
}

#[test]
fn update_user_form() {
    let client = Client::tracked(init_rocket()).expect("valid rocket instance");
    let response = client
        .post("/users/")
        .body(r#"{ "username" : "foobar" }"#)
        .header(ContentType::JSON)
        .dispatch();

    assert_eq!(response.status(), Status::Ok);

    let new_user = response.into_json::<User>().unwrap();

    assert_eq!(new_user.username, "foobar");

    let response = client
        .patch(format!("/users/form/{}", new_user.id))
        .body("username=baz")
        .header(ContentType::Form)
        .dispatch();

    let newer_user = response.into_json::<User>().unwrap();

    assert_eq!(newer_user.username, "baz");
}

#[test]
fn retrieve_user() {
    let client = Client::tracked(init_rocket()).expect("valid rocket instance");
    let response = client
        .post("/users/")
        .body(r#"{ "username" : "foobar" }"#)
        .header(ContentType::JSON)
        .dispatch();

    assert_eq!(response.status(), Status::Ok);

    let create_user = response.into_json::<User>().unwrap();

    let response = client.get(format!("/users/{}", create_user.id)).dispatch();

    let retrieve_user = response.into_json::<User>().unwrap();

    assert_eq!(create_user, retrieve_user);
}

#[test]
fn delete_user() {
    let client = Client::tracked(init_rocket()).expect("valid rocket instance");
    let response = client
        .post("/users/")
        .body(r#"{ "username" : "foobar" }"#)
        .header(ContentType::JSON)
        .dispatch();
    assert_eq!(response.status(), Status::Ok);

    let create_user = response.into_json::<User>().unwrap();

    let response = client
        .delete(format!("/users/{}", create_user.id))
        .dispatch();
    assert_eq!(response.status(), Status::Ok);

    let response = client.get(format!("/users/{}", create_user.id)).dispatch();
    assert_eq!(response.status(), Status::NotFound);
}

#[test]
fn retrieve_list_user() {
    use diesel::prelude::RunQueryDsl;

    let r = init_rocket();
    let client = Client::tracked(r).expect("valid rocket instance");

    let url_origin = "postgres://crud@127.0.0.1:5432/crud";
    // let db_name = "crud";

    {
        use diesel::connection::Connection;

        let connection = diesel::PgConnection::establish(url_origin).unwrap();

        diesel::delete(schema::comments::table)
            .execute(&connection)
            .unwrap();

        diesel::delete(schema::posts::table)
            .execute(&connection)
            .unwrap();

        diesel::delete(schema::users::table)
            .execute(&connection)
            .unwrap();
    }

    let create_user_1 = client
        .post("/users/")
        .body(r#"{ "username" : "alice" }"#)
        .header(ContentType::JSON)
        .dispatch()
        .into_json::<User>()
        .unwrap();

    let create_user_2 = client
        .post("/users/")
        .body(r#"{ "username" : "eve" }"#)
        .header(ContentType::JSON)
        .dispatch()
        .into_json::<User>()
        .unwrap();

    let create_user_3 = client
        .post("/users/")
        .body(r#"{ "username" : "bob" }"#)
        .header(ContentType::JSON)
        .dispatch()
        .into_json::<User>()
        .unwrap();

    let response = client.get("/users").dispatch();

    let users = response.into_json::<Vec<User>>().unwrap();

    assert_eq!(&users, &[create_user_1, create_user_2, create_user_3]);
}
