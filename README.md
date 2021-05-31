# Rocket Crud

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

