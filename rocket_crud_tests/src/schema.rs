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
