use diesel::pg::TransactionBuilder;
use diesel::prelude::Insertable;

struct Foo {
    key: u64,

    name: String,
}

#[derive(Insertable)]
#[table_name = "foo"]
struct NewFoo {
    name: String,
}

impl Foo {
    async fn create<T>(connection: TransactionBuilder, value: NewFoo) -> Foo {
        connection
            .run(move |c| c.insert_into(foo).values(&value))
            .await
            .unwrap()
            .unwrap()
    }
}

pub fn main() {
    println!("hello world");
}
