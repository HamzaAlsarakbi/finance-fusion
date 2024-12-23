use std::env;

use diesel::prelude::*;
use dotenv::dotenv;
use jsonwebtoken::{encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

use crate::database::db::DbConn;
use crate::database::schema::sessions;
use crate::errors::AppError;

/// Session model
#[derive(Debug, Queryable, Insertable, Serialize, Deserialize)]
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

    pub fn delete(&self, conn: &mut DbConn) -> Result<(), AppError> {
        diesel::delete(sessions::table.filter(sessions::id.eq(self.id)))
            .execute(conn)
            .map_err(|e| {
                tracing::error!("Failed to delete session: {e:?}");
                AppError::Diesel(e)
            })?;

        Ok(())
    }

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
    pub fn token(&self) -> Result<String, AppError> {
        let secret = Session::get_secret();
        encode(
            &Header::default(),
            &self,
            &EncodingKey::from_secret(&secret),
        )
        .map_err(|e| {
            tracing::error!("Failed to create token: {e:?}");
            AppError::Authenticate(crate::errors::AuthenticateError::TokenCreation)
        })
    }

    fn get_secret() -> Vec<u8> {
        dotenv().ok(); // Load .env file
        env::var("JWT_SECRET")
            .map(|s| s.into_bytes())
            .unwrap_or_else(|_| {
                tracing::warn!("JWT_SECRET not set, using default secret.");
                DEFAULT_SECRET.to_vec()
            })
    }

    pub fn from_token(conn: &mut DbConn, token: &str) -> Result<Self, AppError> {
        let validation = Validation::new(Algorithm::HS256);

        let secret = Session::get_secret();
        let session =
            jsonwebtoken::decode::<Session>(token, &DecodingKey::from_secret(&secret), &validation)
                .map(|data| data.claims)
                .map_err(|e| {
                    tracing::error!("Failed to decode token: {e:?}");
                    AppError::Authenticate(crate::errors::AuthenticateError::InvalidToken)
                })?;

        // Verify that the session exists in the database
        Session::from_user_id(conn, session.user_id)
    }

    /// Get the user ID
    ///
    /// # Returns
    ///
    /// The user ID
    pub fn user_id(&self) -> i32 {
        self.user_id
    }
}
