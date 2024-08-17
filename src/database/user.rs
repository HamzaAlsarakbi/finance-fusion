use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct User {
  id: i32,
  name: String,
  pw_hash: String,
  invalid_attempts: u16,
}
 