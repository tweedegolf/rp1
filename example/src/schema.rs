table! {
    comments (id) {
        id -> Int4,
        content -> Text,
        approved -> Bool,
        post_id -> Int4,
        user_id -> Nullable<Int4>,
        anonymous_user -> Nullable<Varchar>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

table! {
    posts (id) {
        id -> Int4,
        title -> Varchar,
        subtitle -> Nullable<Varchar>,
        content -> Text,
        publish_date -> Date,
        publish_time -> Time,
        user_id -> Int4,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

table! {
    users (id) {
        id -> Int4,
        username -> Varchar,
        role -> Varchar,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

joinable!(comments -> posts (post_id));
joinable!(comments -> users (user_id));
joinable!(posts -> users (user_id));

allow_tables_to_appear_in_same_query!(
    comments,
    posts,
    users,
);
