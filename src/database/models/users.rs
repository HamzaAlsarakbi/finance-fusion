use crate::{database::schema::users, errors::AppError};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

use crate::database::{connection::DbConn, models::sessions::manager::Session};

/// Struct to represent a user
///
/// This struct is used to represent a user in the database. It includes fields for the user's
/// username, password hash, two-factor authentication secret, creation timestamp, invalid login
/// attempts, lockout policy, and a vector of login timestamps.
#[derive(Debug, Serialize, Deserialize, Queryable, AsChangeset)]
#[diesel(table_name = users)]
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
    /// If the user is in developer mode
    is_dev_mode: bool,
    // logins: Vec<UserLogin>,

    // Lockout policy
    /// The number of invalid login attempts
    invalid_login_attempts: i32,
    /// The lockout duration in seconds (default 60 seconds)
    lock_duration_s: i32,
    /// The lockout duration factor (default 2x)
    lock_duration_factor: i32,
    /// The lockout duration cap in seconds (default 3600 seconds = 60 minutes)
    lock_duration_cap_s: i32,
    /// The timestamp when the user was locked out
    locked_until: Option<chrono::NaiveDateTime>,
}

/// Public user struct
///
/// This struct is used to represent a user in the database. It includes fields for the user's
/// username and creation timestamp.
///
/// This struct omits the password hash and other sensitive information.
#[derive(Debug, Serialize, Deserialize, Queryable)]
pub struct UserPublic {
    /// The user ID
    id: i32,
    /// The username of the user
    username: String,
    /// The timestamp when the user was created
    created_at: chrono::NaiveDateTime,
    /// If the user is in developer mode
    is_dev_mode: bool,
}

/// New user struct
#[derive(Insertable)]
#[table_name = "users"]
struct NewUser<'a> {
    /// The username of the new user
    username: &'a str,
    /// The password hash of the new user
    pw_hash: &'a str,
}

impl User {
    /// Creates a new user
    ///
    /// # Arguments
    ///
    /// * `conn` - A mutable reference to a `DbConn`.
    /// * `username` - A string slice that holds the username of the new user.
    /// * `password` - A string slice that holds the password of the new user.
    ///
    /// # Returns
    ///
    /// A boolean indicating if the user was created successfully.
    pub fn new(conn: &mut DbConn, username: &str, password: &str) -> Result<Self, AppError> {
        let new_user = NewUser {
            username,
            pw_hash: password,
        };

        diesel::insert_into(users::table)
            .values(&new_user)
            .get_result::<User>(conn)
            .map_err(|e| {
                tracing::error!("Error inserting user: {}, error: {e}.", new_user.username);
                AppError::Diesel(e)
            })
    }

    /// Creates a new user with default values
    #[cfg(test)]
    pub fn default(conn: &mut DbConn) -> Result<Self, AppError> {
        let username = "test_user";
        let password = "test_password";

        User::new(conn, username, password)
    }

    /// Updates a user's password
    ///
    /// # Arguments
    ///
    /// * `conn` - A mutable reference to a `DbConn`.
    /// * `username` - A string slice that holds the username of the user to update.
    /// * `password` - A string slice that holds the new password for the user.
    ///
    /// # Returns
    ///
    /// A boolean indicating if the update was successful.
    pub fn update(
        conn: &mut DbConn,
        id: i32,
        username: &str,
        password: &str,
    ) -> Result<(), AppError> {
        let user = users::table.filter(users::id.eq(id));
        if user.first::<User>(conn).is_err() {
            return Err(AppError::not_found());
        }
        match diesel::update(user)
            .set(users::pw_hash.eq(password))
            .execute(conn)
        {
            Ok(_) => Ok(()),
            Err(e) => {
                tracing::error!("Error updating user: {username:?}, error: {e}.");
                Err(AppError::Diesel(e))
            }
        }
    }

    /// Deletes a user
    ///
    /// # Arguments
    ///
    /// * `conn` - A mutable reference to a `DbConn`.
    /// * `id` - A string slice that holds the ID of the user to delete.
    ///
    /// # Returns
    ///
    /// A boolean indicating if the user was deleted successfully.
    pub fn delete(conn: &mut DbConn, id: i32) -> Result<(), AppError> {
        diesel::delete(users::table.filter(users::id.eq(id)))
            .execute(conn)
            .map(|_| ())
            .map_err(|e| {
                tracing::error!("Error deleting user: {id}, error: {e}.");
                AppError::Diesel(e)
            })
    }

    pub fn is_locked(&self) -> bool {
        self.locked_until.is_some()
    }

    /// Gets a user by ID
    ///
    /// # Arguments
    ///
    /// * `id` - The ID of the user to retrieve.
    ///
    /// # Returns
    ///
    /// A `User` struct if the user was found.
    pub fn from_id(conn: &mut DbConn, id: i32) -> Result<Self, AppError> {
        users::table
            .filter(users::id.eq(id))
            .first::<User>(conn)
            .map_err(|e| {
                tracing::error!("Error getting user by ID: {e:?}");
                AppError::Diesel(e)
            })
    }
    /// Gets a user by username
    ///
    /// # Arguments
    ///
    /// * `conn` - A mutable reference to a `DbConn`.
    /// * `username` - The username of the user to retrieve.
    ///
    /// # Returns
    ///
    /// A `User` struct if the user was found.
    pub fn from_username(conn: &mut DbConn, username: &str) -> Result<Self, AppError> {
        users::table
            .filter(users::username.eq(username))
            .first::<User>(conn)
            .map_err(|e| {
                tracing::error!("Error getting user by username \"{username}\": {e:?}");
                AppError::Diesel(e)
            })
    }

    pub fn authenticate(&mut self, conn: &mut DbConn, password: &str) -> Result<Session, AppError> {
        // Check if the password is correct
        // bcrypt::verify(password, &user.pw_hash).unwrap()

        // If account is locked and cannot be unlocked.
        if self.is_locked() && self.unlock(conn).is_err() {
            return Err(AppError::Authenticate(
                crate::errors::AuthenticateError::Locked,
            ));
        }

        // If the password is correct, return Ok(())
        if !self.check_password(password) {
            // Increment the invalid login attempts and lock account if necessary
            self.increment_invalid_login_attempts(conn)?;

            return Err(AppError::Authenticate(
                crate::errors::AuthenticateError::WrongCredentials,
            ));
        }
        self.reset_invalid_login_attempts(conn)?;

        let session = Session::new(conn, self.id)?;
        Ok(session)
    }

    /// Will attempt to unlock the user account if it is locked
    ///
    /// # Arguments
    ///
    /// * `conn` - A mutable reference to a `DbConn`.
    ///
    /// # Returns
    ///
    /// A boolean indicating if the account was unlocked.
    pub fn unlock(&mut self, conn: &mut DbConn) -> Result<(), AppError> {
        let locked_until = match self.locked_until {
            None => return Ok(()),
            Some(locked_until) => locked_until,
        };

        // Check if the lock duration has expired
        if locked_until < chrono::Utc::now().naive_utc() {
            return Ok(());
        }

        // Unlock the account

        self.locked_until = None;
        self.invalid_login_attempts = 0;

        self.save_changes(conn)
    }

    fn lock(&mut self, conn: &mut DbConn) -> Result<(), AppError> {
        let lock_duration = self.lock_duration_s * self.lock_duration_factor;
        let lock_duration = lock_duration.min(self.lock_duration_cap_s) as i64;

        self.locked_until =
            Some(chrono::Utc::now().naive_utc() + chrono::Duration::seconds(lock_duration));

        // Update database
        self.save_changes(conn)
    }

    /// Check if the password is correct
    ///
    /// # Arguments
    ///
    /// * `password` - A string slice that holds the password to check.
    ///
    /// # Returns
    ///
    /// A boolean indicating if the password is correct.
    pub fn check_password(&self, password: &str) -> bool {
        // bcrypt::verify(password, &self.pw_hash).unwrap()
        self.pw_hash == password
    }

    /// Reset the number of invalid login attempts
    ///
    /// # Arguments
    ///
    /// * `conn` - A mutable reference to a `DbConn`.
    ///
    /// # Returns
    ///
    /// A result indicating if the invalid login attempts were reset successfully.
    pub fn reset_invalid_login_attempts(&mut self, conn: &mut DbConn) -> Result<(), AppError> {
        self.invalid_login_attempts = 0;
        self.lock_duration_factor = 3600;

        // Update database
        self.save_changes(conn)
    }

    /// Increment the number of invalid login attempts
    ///
    /// # Arguments
    ///
    /// * `conn` - A mutable reference to a `DbConn`.
    ///
    /// # Returns
    ///
    /// A result indicating if the invalid login attempts were incremented successfully.
    pub fn increment_invalid_login_attempts(&mut self, conn: &mut DbConn) -> Result<(), AppError> {
        self.invalid_login_attempts += 1;

        // Lock the account if necessary
        if self.invalid_login_attempts >= 3 {
            self.lock(conn)?
        }
        Ok(())
    }

    /// Save changes to the user
    ///
    /// # Arguments
    ///
    /// * `conn` - A mutable reference to a `DbConn`.
    ///
    /// # Returns
    ///
    /// A result indicating if the changes were saved successfully.
    fn save_changes(&self, conn: &mut DbConn) -> Result<(), AppError> {
        diesel::update(users::table.filter(users::id.eq(self.id)))
            .set(self)
            .execute(conn)
            .map(|_| ())
            .map_err(|e| {
                tracing::error!("Error saving changes to user: {e:?}");
                AppError::Diesel(e)
            })
    }

    /// Convert a `User` struct to a `UserPublic` struct
    ///
    /// # Returns
    ///
    /// A `UserPublic` struct
    pub fn to_public(&self) -> UserPublic {
        UserPublic {
            id: self.id,
            username: self.username.clone(),
            created_at: self.created_at,
            is_dev_mode: self.is_dev_mode,
        }
    }

    /// Get the ID of the user
    #[cfg(test)]
    pub fn id(&self) -> i32 {
        self.id
    }
}

// write tests
#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::connection::DbPool;

    #[test]
    fn test_new_user() {
        let pool = DbPool::new_test();
        let conn = &mut pool.get().unwrap();
        conn.begin_test_transaction().unwrap();

        let username = "test_user";
        let password = "test_password";

        let user = User::new(conn, username, password).unwrap();

        assert_eq!(user.username, username);

        // TODO(hamza): Verify that the password is hashed
        // Verify that the user is saved correctly in the database
        let found_user = users::table
            .filter(users::id.eq(user.id))
            .first::<User>(conn)
            .unwrap();

        assert_eq!(found_user.username, username);
        // assert_eq!(found_user.pw_hash, password);
        assert!(!found_user.is_dev_mode);
        assert_eq!(found_user.invalid_login_attempts, 0);
        assert_eq!(found_user.lock_duration_s, 60);
        assert_eq!(found_user.lock_duration_factor, 2);
        assert_eq!(found_user.lock_duration_cap_s, 3600);
        assert_eq!(found_user.locked_until, None);

        // Cleanup
        diesel::delete(users::table.filter(users::id.eq(user.id)))
            .execute(conn)
            .unwrap();
    }

    #[test]
    fn test_from_id() {
        let pool = DbPool::new_test();
        let conn = &mut pool.get().unwrap();
        conn.begin_test_transaction().unwrap();

        let username = "test_user";
        let password = "test_password";

        let user = User::new(conn, username, password).unwrap();

        let found_user = User::from_id(conn, user.id).unwrap();

        assert_eq!(found_user.username, username);
    }

    #[test]
    fn test_from_username() {
        let pool = DbPool::new_test();
        let conn = &mut pool.get().unwrap();
        conn.begin_test_transaction().unwrap();

        let username = "test_user";
        let password = "test_password";

        let user = User::new(conn, username, password).unwrap();

        assert_eq!(user.username, username);
    }

    // #[test]
    // fn test_update_user() {
    //     let pool = DbPool::establish_connection_pool_for_testing();
    //     let conn = &mut pool.get().unwrap();

    //     let username = "test_user";
    //     let password = "test_password";

    //     let user = User::new(conn, username, password).unwrap();

    //     let new_password = "new_password";
    //     User::update(conn, user.id, username, new_password).unwrap();

    //     let updated_user = users.filter(id.eq(user.id)).first::<User>(conn).unwrap();

    //     assert_eq!(updated_user.pw_hash, new_password);
    // }

    #[test]
    fn test_delete_user() {
        let pool = DbPool::new_test();
        let conn = &mut pool.get().unwrap();
        conn.begin_test_transaction().unwrap();

        let username = "test_user";
        let password = "test_password";

        let user = User::new(conn, username, password).unwrap();

        User::delete(conn, user.id).unwrap();

        let result = User::from_username(conn, username);

        assert!(result.is_err());
    }
}
