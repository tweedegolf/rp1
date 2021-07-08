use rocket::http::ContentType;
use rocket::Build;
use rocket::Rocket;

use crate::schema;

use rocket_sync_db_pools::database;

fn clear_database() {
    use diesel::connection::Connection;
    use diesel::prelude::RunQueryDsl;

    let url_origin = "postgres://crud@127.0.0.1:5432/crud";
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

#[database("diesel")]
struct Db(diesel::PgConnection);

#[rocket_crud::crud(database = "Db", table_name = "users", casbin = false)]
#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, diesel::Queryable)]
struct User {
    #[primary_key]
    pub id: i32,
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
    let r = init_rocket();
    let client = Client::tracked(r).expect("valid rocket instance");

    clear_database();

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

#[test]
fn retrieve_list_user_sort() {
    let r = init_rocket();
    let client = Client::tracked(r).expect("valid rocket instance");

    clear_database();

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

    // a to z
    let response = client.get("/users?sort=username").dispatch();
    let users = response.into_json::<Vec<User>>().unwrap();
    assert_eq!(
        &users,
        &[
            create_user_1.clone(),
            create_user_3.clone(),
            create_user_2.clone()
        ]
    );

    // a to z, but different
    let response = client.get("/users?sort=+username").dispatch();
    let users = response.into_json::<Vec<User>>().unwrap();
    assert_eq!(
        &users,
        &[
            create_user_1.clone(),
            create_user_3.clone(),
            create_user_2.clone()
        ]
    );

    // z to a
    let response = client.get("/users?sort=-username").dispatch();
    let users = response.into_json::<Vec<User>>().unwrap();
    assert_eq!(&users, &[create_user_2, create_user_3, create_user_1]);
}

#[test]
fn retrieve_list_user_filter() {
    let r = init_rocket();
    let client = Client::tracked(r).expect("valid rocket instance");

    clear_database();

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

    let response = client.get("/users?filter[username]in=alice").dispatch();
    let users = response.into_json::<Vec<User>>().unwrap();
    assert_eq!(&users, &[create_user_1.clone()]);

    let response = client.get("/users?limit=2").dispatch();
    let users = response.into_json::<Vec<User>>().unwrap();
    assert_eq!(&users, &[create_user_1.clone(), create_user_2.clone()]);

    let response = client
        .get(format!("/users?filter[id]gt={}", create_user_1.id))
        .dispatch();
    let users = response.into_json::<Vec<User>>().unwrap();
    assert_eq!(&users, &[create_user_2.clone(), create_user_3.clone(),]);

    let response = client.get("/users?offset=1").dispatch();
    let users = response.into_json::<Vec<User>>().unwrap();
    assert_eq!(&users, &[create_user_2, create_user_3]);
}
