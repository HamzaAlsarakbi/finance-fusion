use axum::{routing::get, Json, Router};
use bson::doc;
use serde::{Deserialize, Serialize};
use tracing::debug;

use crate::errors::Error;

pub fn create_route() -> Router {
  Router::new()
    .route("/vitals", get(get_vitals))
    .route("/hello", get(get_vitals))
}

/// This endpoint responds with a simple greeting message.
///
/// ## Responses
///
/// `200` : A successful response. Returns a greeting message as a string.
///
/// `default` : An unexpected error occurred. Returns an `AppError`.
#[utoipa::path(
  get,
  path = "/vitals",
  responses((status = 200, description = "Successful response"))
)]
async fn get_vitals() -> Result<Json<Vitals>, Error> {
  debug!("Returning vitals");
  Ok(Json(Vitals {
    status: "ok".to_owned(),
  }))
}

/// This endpoint responds with a simple greeting message.
///
/// ## Responses
///
/// `200` : A successful response. Returns a greeting message as a string.
///
/// `default` : An unexpected error occurred. Returns an `AppError`.
#[utoipa::path(
  get,
  path = "/api/hello",
  responses((status = 200, description = "Successful response"))
)]
async fn hello() -> Result<String, Error> {
  Ok("Hello, Rust!".to_string())
}

#[derive(Serialize, Deserialize, Debug)]
struct Vitals {
  status: String,
}
