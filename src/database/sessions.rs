/// Session model
#[derive(Queryable, Insertable)]
#[table_name = "sessions"]
pub struct Session {
  /// The session ID
  id: i32,
  /// The user ID
  user_id: i32,
  /// The session token
  session_token: String,
  /// The session expiration timestamp
  expires_at: chrono::NaiveDateTime,
  /// The timestamp when the session was created
  created_at: chrono::NaiveDateTime,
}
