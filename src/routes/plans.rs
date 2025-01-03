use std::sync::Arc;

use axum::{
    extract::{Path, State},
    middleware,
    routing::{delete, get, post},
    Extension, Json, Router,
};

use crate::{
    database::{
        connection::DbPool,
        models::{plans::Plan, sessions::manager::Session},
    },
    errors::AppError,
};

pub fn create_route(pool: Arc<DbPool>) -> Router<Arc<DbPool>> {
    Router::new()
        .route("/plans", get(all_plans))
        .route("/plans/:name", post(create_plan))
        .route("/plans/:name", delete(delete_plan))
        .layer(middleware::from_fn_with_state(
            pool.clone(),
            crate::middleware::auth::jwt_auth,
        ))
}

/// This endpoint returns all plans for the authenticated user
///
/// ## Responses
/// `200` : A successful response. Returns  a vector of plans.
/// `default` : An unexpected error occurred. Returns an `AppError`.
#[utoipa::path(
        get,
        path = "/plans",
        responses((status = 200), (status = 401, description = "User is not authenticated"))
    )]
async fn all_plans(
    Extension(session): Extension<Session>,
    State(pool): State<Arc<DbPool>>,
) -> Result<Json<Vec<Plan>>, AppError> {
    let mut conn = pool.get()?;

    let plans = Plan::get_all(&mut conn, session.user_id())?;
    Ok(Json(plans)) // Wrap the result in Json
}

/// This endpoint creates a new plan
///
/// ## Responses
///
/// `201` : A successful response. Returns a string indicating the plan was created.
/// `default` : An unexpected error occurred. Returns an `AppError`.
#[utoipa::path(
    post,
    path = "/plans/{name}",
    responses((status = 201, description = "Plan created"))
)]
async fn create_plan(
    State(pool): State<Arc<DbPool>>,
    Extension(session): Extension<Session>,
    Path(name): Path<String>,
) -> Result<String, AppError> {
    let mut conn = pool.get()?;

    Plan::new(&mut conn, &name, session.user_id())?;

    Ok("Plan created".to_string())
}

/// This endpoint deletes a plan
///
/// ## Responses
///
/// `200` : A successful response. Returns a string indicating the plan was deleted.
/// `default` : An unexpected error occurred. Returns an `AppError`.
#[utoipa::path(
    delete,
    path = "/plans/{name}",
    responses((status = 200, description = "Plan deleted"))
)]
async fn delete_plan(
    State(pool): State<Arc<DbPool>>,
    Extension(session): Extension<Session>,
    Path(name): Path<String>,
) -> Result<String, AppError> {
    let mut conn = pool.get()?;

    Plan::delete(&mut conn, &name, session.user_id())?;

    Ok("Plan deleted".to_string())
}
