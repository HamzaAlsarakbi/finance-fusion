use std::sync::Arc;

use axum::{
  extract::Path,
  routing::{delete, get, post, put},
  Json, Router,
};
use serde::{Deserialize, Serialize};
use utoipa::{OpenApi, ToSchema};

use crate::{database::db::DbPool, errors::AppError};

#[derive(Debug, Serialize, Deserialize, OpenApi, ToSchema)]
#[openapi(paths(create_user))]
pub struct CreateUser {
  name: String,
  password: String,
}

#[derive(Debug, Serialize, Deserialize, OpenApi, ToSchema)]
#[openapi(paths(update_user))]
pub struct UpdateUser {
  name: String,
  password: String,
}

pub fn create_route() -> Router<Arc<DbPool>> {
  Router::new()
    .route("/users", post(create_user))
    .route("/users/:id", put(update_user))
    .route("/users/{id}", get(get_user))
  // .route("/users/{id}", delete(delete_user))
}

/// This endpoint creates a user
///
/// ## Responses
///
/// `201` : A successful response. Returns a string indicating the user was created.
///
/// `default` : An unexpected error occurred. Returns an `AppError`.
#[utoipa::path(
  post,
  path = "/users",
  responses((status = 201, description = "User created"))
)]
async fn create_user(Json(payload): Json<CreateUser>) -> Result<String, AppError> {
  Ok(format!("Creating user: {0}", payload.name))
}

/// This endpoint updates a specific user.
///
/// ## Responses
///
/// `201` : A successful response. Returns a string indicateing the user was updated.
///
/// `default` : An unexpected error occurred. Returns an `AppError`.
#[utoipa::path(
  get,
  path = "/users/:id",
  responses((status = 200, description = "User updated"))
)]
async fn update_user(
  Path(id): Path<u64>,
  Json(payload): Json<UpdateUser>,
) -> Result<String, AppError> {
  Ok(format!("Updating user {id} with name {}", payload.name))
}

/// This endpoint retrieves a specific user.
///
/// ## Responses
///
/// `200` : A successful response. Returns a string indicating the user details.
///
/// `default` : An unexpected error occurred. Returns an `AppError`.
#[utoipa::path(
  get,
  path = "/users/:id",
  responses((status = 200, description = "User retrieved"))
)]
async fn get_user(Path(id): Path<u64>) -> Result<String, AppError> {
  Ok(format!("Retrieving user with ID: {}", id))
}
