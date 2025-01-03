use std::env;

use diesel::prelude::*;
use dotenv::dotenv;
use jsonwebtoken::{encode, DecodingKey, EncodingKey, Header, Validation};

use super::claims::Claims;
use crate::database::connection::DbConn;
use crate::database::schema::sessions;
use crate::errors::AppError;

/// Session model
#[derive(Debug, Queryable, Clone, Insertable)]
#[diesel(table_name = sessions)]
pub struct Session {
    /// The session ID
    id: i32,
    /// The user ID
    user_id: i32,
    /// The session expiration timestamp
    expires_at: chrono::NaiveDateTime,
    /// The timestamp when the session was created
    created_at: chrono::NaiveDateTime,
}

/// The default secret for JWT encoding
const DEFAULT_SECRET: &[u8] = b"default-secret-for-dev"; // Fallback for dev/test

/// username and password hash.
#[derive(Insertable)]
#[table_name = "sessions"]
struct NewSession {
    /// The user ID
    user_id: i32,
    /// The session token
    expires_at: chrono::NaiveDateTime,
}

impl Session {
    /// Creates a new session
    ///
    /// # Arguments
    ///
    /// * `conn` - Connection to the database
    /// * `user_id` - User ID
    ///
    /// # Returns
    ///
    /// The newly created session, otherwise an error
    pub fn new(conn: &mut DbConn, user_id: i32) -> Result<Self, AppError> {
        let expires_at = chrono::Utc::now().naive_utc() + chrono::Duration::days(1);

        let new_session = NewSession {
            user_id,
            expires_at,
        };

        diesel::insert_into(sessions::table)
            .values(&new_session)
            .execute(conn)
            .map_err(|e| {
                tracing::error!("Failed to create session: {e:?}");
                AppError::Diesel(e)
            })?;

        Session::from_user_id(conn, user_id)
    }

    /// Deletes a session
    ///
    /// # Arguments
    ///
    /// * `conn` - Connection to the database
    ///
    /// # Returns
    ///
    /// An empty result if successful, otherwise an error
    pub fn delete(&self, conn: &mut DbConn) -> Result<(), AppError> {
        diesel::delete(sessions::table.filter(sessions::id.eq(self.id)))
            .execute(conn)
            .map_err(|e| {
                tracing::error!("Failed to delete session: {e:?}");
                AppError::Diesel(e)
            })?;

        Ok(())
    }

    /// Gets a session by user ID
    ///
    /// # Arguments
    ///
    /// * `conn` - Connection to the database
    /// * `user_id` - User ID
    ///
    /// # Returns
    ///
    /// The session if it exists, otherwise an error
    pub fn from_user_id(conn: &mut DbConn, user_id: i32) -> Result<Self, AppError> {
        let session = sessions::table
            .filter(sessions::user_id.eq(user_id))
            .first::<Session>(conn)
            .map_err(|e| {
                tracing::error!("Failed to get session: {e:?}");
                AppError::Diesel(e)
            })?;

        // Verify that the session hasn't expired yet
        if chrono::Utc::now().naive_utc() < session.expires_at {
            return Ok(session);
        }
        session.delete(conn)?;

        Err(AppError::Authenticate(
            crate::errors::AuthenticateError::SessionExpired,
        ))
    }

    /// Creates a new session from a token
    ///
    /// # Arguments
    ///
    /// * `conn` - Connection to the database
    /// * `token` - The token to decode
    ///
    /// # Returns
    ///
    /// The session if the token is valid
    pub fn from_token(conn: &mut DbConn, token: &str) -> Result<Self, AppError> {
        let validation = Validation::default();

        let secret = Session::get_secret();
        let claims =
            jsonwebtoken::decode::<Claims>(token, &DecodingKey::from_secret(&secret), &validation)
                .map(|data| data.claims)
                .map_err(|e| {
                    tracing::error!("Failed to decode token: {e:?}");
                    AppError::Authenticate(crate::errors::AuthenticateError::InvalidToken)
                })?;

        // Verify that the session exists in the database
        Session::from_user_id(conn, claims.user_id)
    }

    /// Creates a new token
    ///
    /// # Returns
    ///
    /// The token as a string if successful, otherwise an error
    pub fn token(&self) -> Result<String, AppError> {
        let secret = Session::get_secret();
        let claims = Claims::from(self);
        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(&secret),
        )
        .map_err(|e| {
            tracing::error!("Failed to create token: {e:?}");
            AppError::Authenticate(crate::errors::AuthenticateError::TokenCreation)
        })
    }

    /// Gets the secret key for encoding/decoding JWT tokens
    fn get_secret() -> Vec<u8> {
        dotenv().ok(); // Load .env file
        env::var("JWT_SECRET")
            .map(|s| s.into_bytes())
            .unwrap_or_else(|_| {
                tracing::warn!("JWT_SECRET not set, using default secret.");
                DEFAULT_SECRET.to_vec()
            })
    }

    /// Gets the user ID
    pub fn user_id(&self) -> i32 {
        self.user_id
    }

    /// Gets the expiration timestamp
    pub fn expires_at(&self) -> chrono::NaiveDateTime {
        self.expires_at
    }
}

#[cfg(test)]
mod tests {
    use crate::database::{connection::DbPool, models::users::User};

    use super::*;

    #[test]
    fn test_new_session() {
        let pool = DbPool::new_test();
        let conn = &mut pool.get().unwrap();
        conn.begin_test_transaction().unwrap();

        let user = User::default(conn).unwrap();
        let user_id = user.id();
        let session = Session::new(conn, user_id).unwrap();

        assert_eq!(session.user_id, user_id);

        // Verify that the session is saved correctly in the database
        let found_session = sessions::table
            .filter(sessions::id.eq(session.id))
            .first::<Session>(conn)
            .unwrap();

        assert_eq!(found_session.user_id, user_id);
        assert!(chrono::Utc::now().naive_utc() < found_session.expires_at);
    }

    #[test]
    fn test_from_token() {
        let pool = DbPool::new_test();
        let conn = &mut pool.get().unwrap();
        conn.begin_test_transaction().unwrap();

        let user = User::default(conn).unwrap();
        let user_id = user.id();
        let session = Session::new(conn, user_id).unwrap();

        let token = session.token().unwrap();

        let decoded_session = Session::from_token(conn, &token).unwrap();

        assert_eq!(decoded_session.user_id, user_id);
    }
}
