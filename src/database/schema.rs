// @generated automatically by Diesel CLI.

diesel::table! {
    sessions (id) {
        id -> Int4,
        session_id -> Uuid,
        user_id -> Int4,
        expires_at -> Timestamp,
        created_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    users (id) {
        id -> Int4,
        #[max_length = 64]
        username -> Varchar,
        pw_hash -> Text,
        two_fa_secret -> Nullable<Text>,
        created_at -> Timestamp,
        invalid_login_attempts -> Int4,
        lock_duration_s -> Int4,
        lock_duration_factor -> Int4,
        lock_duration_cap -> Int4,
        locked_until -> Nullable<Timestamp>,
    }
}

diesel::joinable!(sessions -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(sessions, users,);
