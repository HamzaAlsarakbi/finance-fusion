use std::sync::Arc;

use axum::{
    extract::{Path, State},
    routing::{delete, get, post, put},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use utoipa::{OpenApi, ToSchema};

use crate::{
    database::{db::DbPool, users::{User, UserPublic}},
    errors::AppError,
};

/// Create a new user request body
#[derive(Debug, Serialize, Deserialize, OpenApi, ToSchema)]
#[openapi(paths(create_user))]
pub struct CreateUser {
    /// The username of the user
    name: String,
    /// The password of the user
    password: String,
}

/// Update user request body
#[derive(Debug, Serialize, Deserialize, OpenApi, ToSchema)]
#[openapi(paths(update_user))]
pub struct UpdateUser {
    /// The username of the user
    name: String,
    /// The password of the user
    password: String,
}

pub fn create_route() -> Router<Arc<DbPool>> {
    Router::new()
        .route("/users", post(create_user))
        .route("/users/username/:username", get(get_user))
        .route("/users/:id", put(update_user))
        .route("/users/:id", delete(delete_user))
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
async fn create_user(
    State(pool): State<Arc<DbPool>>,
    Json(payload): Json<CreateUser>,
) -> Result<String, AppError> {
    let mut conn = pool.get()?;
    User::new(&mut conn, &payload.name, &payload.password)?;
    Ok(format!("Creating user: {}", payload.name))
}

/// Retreives a specific user.
///
/// ## Responses
///
/// `200` : A successful response. Returns a string indicating the user details.
///
/// `default` : An unexpected error occurred. Returns an `AppError`.
#[utoipa::path(
  get,
  path = "/users/username/{username}",
  params(
    ("username" = String, Path, description = "Username of the user to retrieve")
  ),
  responses((status = 200, description = "User retrieved"))
)]
async fn get_user(
    State(pool): State<Arc<DbPool>>,
    Path(username): Path<String>,
) -> Result<Json<UserPublic>, AppError> {
    let mut conn = pool.get()?;
    let user = User::from_username(&mut conn, &username)?.to_public();
    Ok(Json(user))
}

/// Updates a specific user.
///
/// ## Responses
///
/// `201` : A successful response. Returns a string indicateing the user was updated.
///
/// `default` : An unexpected error occurred. Returns an `AppError`.
#[utoipa::path(
  put,
  path = "/users/{id}",
  params(
    ("id" = String, Path, description = "ID of the user to update")
  ),
  responses(
    (status = 200, description = "Updated user {id} successfully"),
    (status = 404, description = "User {id} not found")
)
)]
async fn update_user(
    State(pool): State<Arc<DbPool>>,
    Path(id): Path<u64>,
    Json(payload): Json<UpdateUser>,
) -> Result<String, AppError> {
    // return if can't get pool connection
    let mut conn = pool.get()?;

    User::update(&mut conn, id as i32, &payload.name, &payload.password)
        .map(|_| format!("Updated user {id} successfully"))
}

/// Deletes a specific user.
///
/// ## Responses
///
/// `200` : A successful response. Returns a string indicating the user details.
///
/// `default` : An unexpected error occurred. Returns an `AppError`.
#[utoipa::path(
  delete,
  path = "/users/{id}",
  params(
    ("id" = String, Path, description = "ID of the user to update")
  ),
  responses((status = 200, description = "Deleted user {id} successfully"))
)]
async fn delete_user(
    State(pool): State<Arc<DbPool>>,
    Path(id): Path<u64>,
) -> Result<String, AppError> {
    // return if can't get pool connection
    let mut conn = pool.get()?;

    User::delete(&mut conn, id as i32).map(|_| format!("Deleted user {id} successfully"))
}
