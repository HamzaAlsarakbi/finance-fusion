use std::sync::Arc;

use axum::{routing::get, Json, Router};
use bson::doc;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{database::db::DbPool, errors::AppError};

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct Vitals {
  pub status: String,
}
pub fn create_route() -> Router<Arc<DbPool>> {
  Router::new()
    .route("/vitals", get(get_vitals))
    .route("/hello", get(hello))
}

/// This endpoint responds with the vitals of the server.
///
/// ## Responses
///
/// `200` : A successful response. Returns a JSON object containing the server's vitals.
///
/// `default` : An unexpected error occurred. Returns an `AppError`.
#[utoipa::path(
  get,
  path = "/vitals",
  responses((status = 200, description = "Successful response"))
)]
pub async fn get_vitals() -> Result<Json<Vitals>, AppError> {
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
  path = "/hello",
  responses((status = 200, description = "Successful response"))
)]
async fn hello() -> Result<String, AppError> {
  Ok("Hello, world!".to_owned())
}
