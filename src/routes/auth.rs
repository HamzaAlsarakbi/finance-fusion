use std::sync::Arc;

use axum::{
    extract::State,
    http::{header::SET_COOKIE, StatusCode},
    middleware,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use utoipa::{OpenApi, ToSchema};

use crate::{
    database::{connection::DbPool, models::users::User},
    errors::AppError,
};

/// This struct represents the user login request body
#[derive(Debug, Serialize, Deserialize, OpenApi, ToSchema)]
#[openapi(paths(login))]
pub struct LoginInfo {
    /// The username of the user
    username: String,
    /// The password of the user
    password: String,
}

pub fn create_route(pool: Arc<DbPool>) -> Router<Arc<DbPool>> {
    Router::new().route("/auth/login", post(login)).route(
        "/auth/logout",
        get(logout).layer(middleware::from_fn_with_state(
            pool.clone(),
            crate::middleware::auth::jwt_auth,
        )),
    )
}

/// This endpoint logs a user in
///
/// ## Responses
/// `200` : A successful response. Returns a a "Login successful" message.
/// `default` : An unexpected error occurred. Returns an `AppError`.
#[utoipa::path(
    post,
    path = "/auth/login",
    responses((status = 200, description = "Login successful"))
)]
async fn login(
    State(pool): State<Arc<DbPool>>,
    Json(info): Json<LoginInfo>,
) -> Result<impl IntoResponse, AppError> {
    let mut conn = pool.get()?;

    let mut user = User::from_username(&mut conn, &info.username)?;

    let session = user.authenticate(&mut conn, &info.password)?;

    let token = session.token()?;

    let cookie = format!("token={token}; HttpOnly; Secure; SameSite=Strict; Path=/");
    let response = (
        StatusCode::OK,
        [(SET_COOKIE, cookie)],
        "Login successful".to_string(),
    );
    Ok(response)
}

/// This endpoint logs a user out
///
/// ## Responses
/// `200` : A successful response. Returns a string indicating the user was logged out.
/// `default` : An unexpected error occurred. Returns an `AppError`.
#[utoipa::path(
    get,
    path = "/auth/logout",
    responses((status = 200, description = "User logged out"))
)]
async fn logout() -> Result<String, AppError> {
    // Here you would normally handle the logout process
    // For simplicity, we'll just return a success message
    Ok("Logged out".to_string())
}

/// This endpoint refreshes a user's session (TODO: Implement)
///
/// ## Responses
/// `200` : A successful response. Returns a a "Token refresh successful" message.
/// `default` : An unexpected error occurred. Returns an `AppError`.
#[utoipa::path(
    post,
    path = "/auth/refresh",
    responses((status = 200, description = "Token refresh successful"))
)]
async fn refresh(State(pool): State<Arc<DbPool>>) -> Result<impl IntoResponse, AppError> {
    // let mut conn = pool.get()?;

    // let mut user = User::from_username(&mut conn, &info.username)?;

    // let session = user.authenticate(&mut conn, &info.password)?;

    // let token = session.token()?;

    // let cookie = format!("token={}; HttpOnly; Secure; SameSite=Strict", token);
    // let response = (
    //     StatusCode::OK,
    //     [(SET_COOKIE, cookie)],
    //     "Login successful".to_string(),
    // );
    // Ok(response)

    Ok("TODO: Token refresh successful".to_string())
}
