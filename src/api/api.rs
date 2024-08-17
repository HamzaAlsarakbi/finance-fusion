use std::sync::Arc;

use axum::{
  // routing::get,
  Router,
};

use tokio::sync::oneshot::Receiver;

use tracing::info;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::database::db::DbPool;
use crate::routes::users::{CreateUser, UpdateUser};
use crate::routes::vitals::Vitals;
use crate::{errors::AppError, routes};
#[derive(OpenApi)]
#[openapi(
  components(schemas(Vitals, CreateUser, UpdateUser)),
  paths(crate::routes::vitals::get_vitals, crate::routes::vitals::hello,
    crate::routes::users::get_user, crate::routes::users::create_user, crate::routes::users::update_user),
  tags(
    (name="vitals", description="Endpoints for retrieving system vitals"),
    (name="users", description="Endpoints for managing users"),
  )
)]
struct ApiDoc;

/// Creates a new instance of the REST application.
///
/// # Returns
///
/// * `Router` - The router with the REST API endpoints.
pub fn app(pool: Arc<DbPool>) -> Router {
  Router::new()
    .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
    .merge(routes::vitals::create_route())
    .merge(routes::users::create_route())
    .with_state(pool)
}

/// Starts the REST server.
///
/// # Arguments
///
/// * `rest_port` - The port number on which the REST server will listen.
/// * `rx` - A Receiver from a one-shot channel for shutdown signal communication.
///
/// # Returns
///
/// * `Result<()>` - Returns `Ok(())` if the server started successfully. Returns `Err(e)` if an error occurred.
///
/// # Behavior
///
/// This function first formats the bind address using the provided port number.
/// It then creates a TCP listener bound to this address.
///
/// The function creates a new instance of the REST application using the `rest_app` function.
///
/// It then creates a new server using the `axum::serve` function, passing in the listener and the application.
/// The server is configured to shut down gracefully when a message is received over the one-shot channel.
///
/// The function then starts the server and waits for it to complete.
/// If the server encounters an error, it is converted to an `anyhow::Error` and returned.
pub async fn start_rest_server(
  rest_port: u16,
  rx: Receiver<()>,
  pool: Arc<DbPool>,
) -> Result<(), AppError> {
  let bind_address = format!("0.0.0.0:{}", rest_port);
  info!("Listening on http://localhost:{rest_port}");
  let listener = tokio::net::TcpListener::bind(bind_address).await?;

  let app = app(pool);

  // Start the server
  let server = axum::serve(listener, app.into_make_service()).with_graceful_shutdown(async {
    rx.await.ok();
  });

  if let Err(err) = server.await {
    return Err(AppError::from(err));
  }

  Ok(())
}

// #[cfg(test)]
// mod tests {
//   use super::*;
//   use axum::body::Body;
//   use axum::http::{Request, Uri};
//   use http_body_util::BodyExt;
//   use tower::ServiceExt;

//   #[tokio::test]
//   async fn test_hello() {
//     let app = app(Arc::new(PgPool::new()));

//     let request = Request::builder()
//       .method("GET")
//       .uri(Uri::from_static("/analytic/hello"))
//       .body(Body::empty())
//       .unwrap();

//     let response = app.oneshot(request).await.unwrap();

//     assert_eq!(response.status(), 200, "Should return 200 OK.");

//     let body = response.into_body().collect().await.unwrap().to_bytes();
//     let body_str = std::str::from_utf8(&body).unwrap();

//     assert_eq!(
//       body_str, "Hello, Rust!",
//       "Should return the correct greeting."
//     );
//   }

//   #[tokio::test]
//   async fn test_start_rest_server() {
//     let (tx, rx) = tokio::sync::oneshot::channel::<()>();

//     let rest_port = 8080;
//     let server = start_rest_server(rest_port, rx);

//     tx.send(()).expect("Failed to send shutdown signal");

//     tokio::select! {
//       result = server => {
//         result.expect("Server encountered an error");
//       }
//       () = tokio::time::sleep(std::time::Duration::from_secs(1)) => {
//         // Server shut down successfully
//       }
//     }
//   }
// }
