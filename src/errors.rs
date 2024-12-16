use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use bcrypt::BcryptError;
use serde_json::json;
use tokio::task::JoinError;

use diesel::result::ConnectionError as SQLError;
use diesel::result::Error as DieselError;

#[derive(thiserror::Error, Debug)]
#[error("...")]
pub enum AppError {
    #[error("{0}")]
    Diesel(#[from] DieselError),

    #[error("{0}")]
    SQL(#[from] SQLError),

    #[error("{0}")]
    Signal(#[from] std::io::Error),

    #[error("Error parsing ObjectID {0}")]
    ParseObjectID(String),

    #[error("{0}")]
    SerializeMongoResponse(#[from] bson::de::Error),

    #[error("{0}")]
    Authenticate(#[from] AuthenticateError),

    #[error("{0}")]
    BadRequest(#[from] BadRequest),

    #[error("{0}")]
    NotFound(#[from] NotFound),

    #[error("{0}")]
    RunSyncTask(#[from] JoinError),

    #[error("{0}")]
    HashPassword(#[from] BcryptError),

    #[error("Unknown error")]
    Unknown,
}

impl AppError {
    fn get_codes(&self) -> (StatusCode, u16) {
        match *self {
            // 4XX Errors
            AppError::ParseObjectID(_) => (StatusCode::BAD_REQUEST, 40001),
            AppError::BadRequest(_) => (StatusCode::BAD_REQUEST, 40002),
            AppError::NotFound(_) => (StatusCode::NOT_FOUND, 40003),
            AppError::Authenticate(AuthenticateError::WrongCredentials) => {
                (StatusCode::UNAUTHORIZED, 40004)
            }
            AppError::Authenticate(AuthenticateError::InvalidToken) => {
                (StatusCode::UNAUTHORIZED, 40005)
            }
            AppError::Authenticate(AuthenticateError::Locked) => (StatusCode::LOCKED, 40006),

            // 5XX Errors
            AppError::Signal(_) => (StatusCode::INTERNAL_SERVER_ERROR, 5003),
            AppError::Authenticate(AuthenticateError::TokenCreation) => {
                (StatusCode::INTERNAL_SERVER_ERROR, 5001)
            }
            AppError::Diesel(_) => (StatusCode::INTERNAL_SERVER_ERROR, 5002),
            AppError::SQL(_) => (StatusCode::INTERNAL_SERVER_ERROR, 5003),
            AppError::SerializeMongoResponse(_) => (StatusCode::INTERNAL_SERVER_ERROR, 5004),
            AppError::RunSyncTask(_) => (StatusCode::INTERNAL_SERVER_ERROR, 5005),
            AppError::HashPassword(_) => (StatusCode::INTERNAL_SERVER_ERROR, 5006),
            AppError::Unknown => (StatusCode::INTERNAL_SERVER_ERROR, 5000),
        }
    }

    pub fn bad_request() -> Self {
        AppError::BadRequest(BadRequest {})
    }

    pub fn not_found() -> Self {
        AppError::NotFound(NotFound {})
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status_code, code) = self.get_codes();
        let message = self.to_string();
        let body = Json(json!({ "code": code, "message": message }));

        (status_code, body).into_response()
    }
}

#[derive(thiserror::Error, Debug)]
#[error("...")]
pub enum AuthenticateError {
    #[error("Wrong authentication credentials")]
    WrongCredentials,
    #[error("Failed to create authentication token")]
    TokenCreation,
    #[error("Invalid authentication credentials")]
    InvalidToken,
    #[error("User is locked")]
    Locked,
}

#[derive(thiserror::Error, Debug)]
#[error("Bad Request")]
pub struct BadRequest {}

#[derive(thiserror::Error, Debug)]
#[error("Not found")]
pub struct NotFound {}
