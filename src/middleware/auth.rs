use std::sync::Arc;

use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
};

use crate::{
    database::{connection::DbPool, models::sessions::manager::Session},
    errors::AppError,
};
/// Authorizes protected routes using JWT tokens.
pub async fn jwt_auth(
    State(pool): State<Arc<DbPool>>,
    mut req: Request<axum::body::Body>, // Use concrete `axum::body::Body` type
    next: Next,                         // Use `Next` without generics
) -> Result<Response, AppError> {
    let mut conn = pool.get()?;

    // Extract the `token` cookie
    let token = req
        .headers()
        .get("cookie")
        .and_then(|cookies| cookies.to_str().ok())
        .and_then(|cookies| cookies.split("; ").find(|c| c.starts_with("token=")))
        .and_then(|token_cookie| token_cookie.strip_prefix("token="));

    tracing::info!("token = {token:?}");

    if let Some(token) = token {
        // Validate the token (implement your logic here)
        if let Ok(session) = Session::from_token(&mut conn, token) {
            tracing::info!("Token is valid");
            // Add user ID (claims.sub) to request extensions, so that it can be used in the routes later
            req.extensions_mut().insert(session);
            return Ok(next.run(req).await);
        }
    }

    // Reject if no valid token is found
    Err(AppError::Authenticate(
        crate::errors::AuthenticateError::InvalidToken,
    ))
}
