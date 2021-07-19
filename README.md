# Rocket Crud

# Access Control with Casbin

See the `casbin-middleware` branch. This is ready to merge.

We integrate with the [casbin](https://github.com/casbin/casbin-rs) library for access control.
It needs two pieces of configuration, and a way to determine the role from an incomming request.

The model ?:

```rust
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
```

Policies define whether a user is allowed to access a resource. Policies can be defined statically (e.g. in a file) or loaded dynamically (e.g. from a database, see below). A simple example is:

```rust
const POLICY: &str = "
p, bob, /comments, GET
p, admin, /users, GET

g, bob, admin
";
```

In words

* the user `bob` is allowed to perform a `GET` request on the `/comments` endpoint.
* any user with the `admin` role is allowed to perform a `GET` request on the `/users` endpoint.
* user `bob` has role `admin`

Next, we must be able to determine which user is making a request, to determine whether that user is allowed to make the request. This is done with a rocket `Fairing`: middleware that intercepts requests and can perform validation.

This `RoleHeaderFairing` inspects the request headers, and expects a custom header that provides the role. Proper authentiction should be used in practice. The role is then stored in the request's local storage.

```rust
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
```

There are three `EnforcedBy` options:

```rust
#[derive(Debug)]
pub enum EnforcedBy {
    Subject(String),
    SubjectAndDomain { subject: String, domain: String },
    ForbidAll,
}
```

Finally, we must initialize rocket with the appropriate fairings: our custom fairing that sets the role, and the `PermissionsFairing` that checks the role agains the current casbin policies.

# Access Control: Loading policies from the database

If you're using this crate with access control, then you are already using diesel and casbin. Loading policies while the application is running is enabled by the [diesel-adapter](https://github.com/casbin-rs/diesel-adapter) crate. Its README explains how to set up the policies table and how to initialize the enforcer to use that table.

## Design discussion

```rust
#[make_create("/foo")]
struct Foo {
    #[primary_key]
    key: u64,

    name: String
}

struct NewFoo {
    name : String
}

impl Foo {
    type NewType = NewFoo;

    async fn create<C>(connection: C, value: Foo::NewType) -> Foo
        where
            T::NewType : Diesel::Insertable
    {
        connection.run(move |c| c.insert_into(table_name).values(&value))
            .await
            .unwrap()
            .unwrap()
    }
}


#[post("/foo")]
fn foo(connection: C, value: Json<Foo::NewType>) -> Response {
    create(connecion

# ik krijg

struct T {
    a : U
}

struct T::Partial {
    a : Option<U>
}

struct Age = Age(u64);




Controller::read(connection: C, index: T::Index);

Controller::update(connection: C, index: T::Index, partial_value: T::Partial);

Controller::update_with(connection: C, index: T::Index, FnOnce(&mut T))

Controller::update_row<T>(connection: C, index: T::Index, value: U))

Controller::delete(connection: C, index: T::Index);
```


