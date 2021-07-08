# Rocket Crud

# Access Control with Casbin

See the `casbin-middleware` branch. This is ready to merge.

We integrate with the [casbin](https://github.com/casbin/casbin-rs) library for access control. 

It needs two pieces of configuration, and a way to determine the role from an incomming request.

TODO model, policy, EnforcedBy, Fairing

# Server-Side validation

In general, you cannot trust incomming requests, and sometimes want to do your own validation. For instance, is the the email field actually a valid email address? We integrate with the [validator](https://github.com/Keats/validator) crate. 

Make sure the `validate` feature flag is turned on to use `rocket-crud` together with `validator`. 

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

