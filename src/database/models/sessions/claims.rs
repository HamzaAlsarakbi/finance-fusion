use serde::{Deserialize, Serialize};

use super::manager::Session;
/// Claims for a JWT token (used for encoding/decoding)
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    /// The user ID
    pub user_id: i32,
    /// The expiration timestamp (UNIX timestamp)
    exp: usize,
    /// The issued at timestamp (UNIX timestamp)
    iat: usize,
    /// The not before timestamp (UNIX timestamp)
    nbf: usize,
}

impl Claims {
    pub fn from(session: &Session) -> Self {
        Self {
            user_id: session.user_id(),
            exp: session.expires_at().and_utc().timestamp() as usize,
            iat: chrono::Utc::now().timestamp() as usize,
            nbf: chrono::Utc::now().timestamp() as usize,
        }
    }
}
