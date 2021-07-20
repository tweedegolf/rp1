use rocket::data::Data;
use rocket::fairing::{Fairing, Info, Kind};
use rocket::http::ContentType;
use rocket::http::Header;
use rocket::request::Request;
use rocket::Build;
use rocket::Rocket;
use rocket_crud::access_control::EnforcedBy;

use rocket_sync_db_pools::database;

#[database("diesel")]
struct Db(diesel::PgConnection);

#[rocket_crud::crud(database = "Db", table_name = "users")]
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


#[derive(std::hash::Hash, serde::Serialize, Debug)]
struct AuthUser {
    id: i32,
    role: String,
}

const MODEL: &str = "
[request_definition]
r = sub, obj, act

[policy_definition]
p = sub_rule, obj, act

[policy_effect]
e = some(where (p.eft == allow))

[matchers]
m = eval(p.sub_rule) && r.obj == p.obj && r.act == p.act
";

const POLICY: &str = r#"
p, r.sub.role == "admin", /users, POST
p, r.sub.role == "poster" && r.sub.id == r.obj.user_id, /posts, POST
p, r.sub.role == "commenter", /comments, POST
"#;

async fn init_rocket() -> Rocket<Build> {
    use casbin::{DefaultModel, FileAdapter};
    use tempfile::NamedTempFile;

    let m = match DefaultModel::from_str(MODEL).await {
        Ok(m) => m,
        Err(e) => panic!("{:?}", e),
    };

    let mut file = NamedTempFile::new().unwrap();
    use std::io::Write;
    write!(file, "{}", POLICY).unwrap();

    let path = file.path().to_owned();

    let a = FileAdapter::new(path);

    let casbin_fairing = match rocket_crud::access_control::PermissionsFairing::<AuthUser>::new(m, a).await {
        Ok(f) => f,
        Err(e) => panic!("{:?}", e),
    };

    rocket::build()
        .mount("/users", User::get_routes())
        .mount("/posts", Post::get_routes())
        .mount("/comments", Comment::get_routes())
        .attach(UserIdFairing)
        .attach(casbin_fairing)
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

    // create a user, s.t. we can use its ID
    // let role_admin = Header::new("Authorization", "0");
    // let _response = client
    //     .post("/users")
    //     .body(r#"{ "username" : "foobar@example.com" }"#)
    //     .header(ContentType::JSON)
    //     .header(role_admin)
    //     .dispatch()
    //     .await;

    let id = Header::new("X-Auth-Id", "82");
    let role = Header::new("X-Auth-Role", "poster");
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

    // create a user, s.t. we can use its ID
    // let role_admin = Header::new("Authorization", "0");
    // let _response = client
    //     .post("/users")
    //     .body(r#"{ "username" : "foobar@example.com" }"#)
    //     .header(ContentType::JSON)
    //     .header(role_admin)
    //     .dispatch()
    //     .await;

    let id = Header::new("X-Auth-Id", "81");
    let role = Header::new("X-Auth-Role", "poster");
    let response = client
        .post("/posts")
        .body(r#"{ "title": "Bla", "content" : "Blablabla", "user_id": 81 }"#) // TODO make use of newly created user
        .header(ContentType::JSON)
        .header(id)
        .header(role)
        .dispatch()
        .await;

    assert_eq!(response.status(), Status::Ok);
}

pub struct UserIdFairing;

#[rocket::async_trait]
impl Fairing for UserIdFairing {
    fn info(&self) -> Info {
        Info {
            name: "UserIdFairing",
            kind: Kind::Request | Kind::Response,
        }
    }

    async fn on_request(&self, request: &mut Request<'_>, _data: &mut Data<'_>) {
        let id = request.headers().get_one("X-Auth-Id").unwrap().parse::<i32>().unwrap();
        let role = request.headers().get_one("X-Auth-Role").unwrap();

        request.local_cache(|| EnforcedBy::<AuthUser>::Subject(AuthUser { id, role: role.to_owned() }));
    }
}
