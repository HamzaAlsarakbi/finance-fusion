use std::sync::Arc;

use axum::{
    extract::{Request, State},
    middleware::{self, Next},
    response::Response,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use utoipa::{OpenApi, ToSchema};

use crate::{
    database::{db::DbPool, sessions::Session, users::User},
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
            crate::routes::auth::jwt_auth,
        )),
    )
}

/// This endpoint logs a user in
///
/// ## Responses
/// `200` : A successful response. Returns a session token string.
/// `default` : An unexpected error occurred. Returns an `AppError`.
#[utoipa::path(
    post,
    path = "/auth/login",
    responses((status = 200))
)]
async fn login(
    State(pool): State<Arc<DbPool>>,
    Json(info): Json<LoginInfo>,
) -> Result<String, AppError> {
    let mut conn = pool.get()?;

    let mut user = User::from_username(&mut conn, &info.username)?;

    let session = user.authenticate(&mut conn, &info.password)?;

    session.token()
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

pub async fn jwt_auth(
    State(pool): State<Arc<DbPool>>,
    mut req: Request<axum::body::Body>, // Use concrete `axum::body::Body` type
    next: Next,                         // Use `Next` without generics
) -> Result<Response, AppError> {
    let mut conn = pool.get()?;
    // Extract token from the `Authorization` header
    if let Some(auth_header) = req.headers().get("Authorization") {
        if let Ok(auth_str) = auth_header.to_str() {
            if auth_str.starts_with("Bearer ") {
                let token = auth_str.strip_prefix("Bearer ").unwrap_or("");
                if let Ok(session) = Session::from_token(&mut conn, token) {
                    // Add user ID (claims.sub) to request extensions
                    req.extensions_mut().insert(session.user_id());
                    return Ok(next.run(req).await);
                }
            }
        }
    }

    // Reject if no valid token is found
    Err(AppError::Authenticate(
        crate::errors::AuthenticateError::InvalidToken,
    ))
}
