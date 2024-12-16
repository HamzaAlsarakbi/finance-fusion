use crate::database::schema::users;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

/// Struct to represent a user
///
/// This struct is used to represent a user in the database. It includes fields for the user's
/// username, password hash, two-factor authentication secret, creation timestamp, invalid login
/// attempts, lockout policy, and a vector of login timestamps.
#[derive(Debug, Serialize, Deserialize, Queryable)]
pub struct User {
    /// The user ID
    id: i32,
    /// The username of the user
    username: String,
    /// The password hash of the user
    pw_hash: String,
    /// The two-factor authentication secret of the user
    two_fa_secret: Option<String>,
    /// The timestamp when the user was created
    created_at: chrono::NaiveDateTime,
    // logins: Vec<UserLogin>,

    // Lockout policy
    /// The number of invalid login attempts
    invalid_login_attempts: i32,
    /// The lockout duration in seconds
    lock_duration_s: i32,
    /// The lockout duration factor
    lock_duration_factor: i32,
    /// The lockout duration cap
    lock_duration_cap: i32,
    /// The timestamp when the user was locked out
    locked_until: Option<chrono::NaiveDateTime>,
}

/// Struct to represent a user login timestamp
///
/// This struct is used to represent a user login timestamp in the database. It includes fields for
/// the login ID, user ID, and login timestamp.
#[derive(Debug, Serialize, Deserialize, Queryable)]
struct UserLogin {
    /// The login ID
    id: i32,
    /// The user ID
    user_id: i32,
    /// The login timestamp
    login_time: chrono::NaiveDateTime,
}

/// Struct to represent a new user
///
/// This struct is used to represent a new user in the database. It includes fields for the user's
/// username and password hash.
#[derive(Insertable)]
#[table_name = "users"]
struct NewUser<'a> {
    /// The username of the new user
    username: &'a str,
    /// The password hash of the new user
    pw_hash: &'a str,
}

/// Create a new user
///
/// This function creates a new user in the database.
///
/// # Arguments
///
/// * `conn` - A mutable reference to a `PgConnection`.
/// * `username` - A string slice that holds the username of the new user.
/// * `password` - A string slice that holds the password of the new user.
///
/// # Returns
///
/// A boolean indicating if the user was created successfully.
pub fn db_create_user(conn: &mut PgConnection, username: &str, password: &str) -> bool {
    let new_user = NewUser {
        username,
        pw_hash: password,
    };

    match diesel::insert_into(users::table)
        .values(&new_user)
        .execute(conn)
    {
        Ok(_) => true,
        Err(e) => {
            tracing::error!("Error inserting user: {:?}, error: {e}.", new_user.username);
            false
        }
    }
}

/// Get a user by username
///
/// This function retrieves a user from the database by their username.
///
/// # Arguments
///
/// * `conn` - A mutable reference to a `PgConnection`.
/// * `username` - A string slice that holds the username of the user to retrieve.
///
/// # Returns
///
/// An `Option` that holds a `User` if the user was found, otherwise `None`.
pub fn db_get_user(conn: &mut PgConnection, username: String) -> Option<User> {
    users::table
        .filter(users::username.eq(username))
        .first::<User>(conn)
        .ok()
}

/// Update a user's password
///
/// This function updates a user's password in the database.
///
/// # Arguments
///
/// * `conn` - A mutable reference to a `PgConnection`.
/// * `username` - A string slice that holds the username of the user to update.
/// * `password` - A string slice that holds the new password for the user.
///
/// # Returns
///
/// A boolean indicating if the update was successful.
pub fn db_update_user(conn: &mut PgConnection, username: &str, password: &str) -> bool {
    diesel::update(users::table.filter(users::username.eq(username)))
        .set(users::pw_hash.eq(password))
        .execute(conn)
        .is_ok()
}
