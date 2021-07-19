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

const MODEL: &str = "
[request_definition]
r = sub, obj, act

[policy_definition]
p = sub, obj, act

[role_definition]
g = _, _

[policy_effect]
e = some(where (p.eft == allow))

[matchers]
m = g(r.sub, p.sub) && r.obj == p.obj && r.act == p.act
";

const POLICY: &str = "
p, bob, /comments, GET
p, admin, /users, GET

g, bob, admin
";

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

    let casbin_fairing = match rocket_crud::access_control::PermissionsFairing::new(m, a).await {
        Ok(f) => f,
        Err(e) => panic!("{:?}", e),
    };

    rocket::build()
        .mount("/users", User::get_routes())
        // .mount("/posts", Post::get_routes())
        // .mount("/comments", Comment::get_routes())
        .attach(RoleHeaderFairing)
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

    let role = Header::new("X-Plain-Text-Auth", "bob");

    let response = client
        .post("/users/")
        .body(r#"{ "username" : "foobar" }"#)
        .header(ContentType::JSON)
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

    let role = Header::new("X-Plain-Text-Auth", "alice");

    let response = client
        .post("/users/")
        .body(r#"{ "username" : "foobar" }"#)
        .header(ContentType::JSON)
        .header(role)
        .dispatch()
        .await;

    assert_eq!(response.status(), Status::Forbidden);
}

pub struct RoleHeaderFairing;

#[rocket::async_trait]
impl Fairing for RoleHeaderFairing {
    fn info(&self) -> Info {
        Info {
            name: "AlwaysAdminFairing",
            kind: Kind::Request | Kind::Response,
        }
    }

    async fn on_request(&self, request: &mut Request<'_>, _data: &mut Data<'_>) {
        let role = request.headers().get_one("X-Plain-Text-Auth").unwrap();

        request.local_cache(|| EnforcedBy::Subject(role.into()));
    }
}
