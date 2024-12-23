use diesel::pg::PgConnection;
use diesel::r2d2::{self, ConnectionManager, Pool, PooledConnection};
use dotenv::dotenv;
use std::env;

use crate::errors::AppError;

/// Type alias for a connection pool
pub struct DbPool {
    /// The connection pool
    connection: r2d2::Pool<ConnectionManager<PgConnection>>,
}
/// A connection from a connection pool `DbPool`
pub type DbConn = PooledConnection<ConnectionManager<PgConnection>>;

impl DbPool {
    /// Create and return a connection pool
    ///
    /// # Returns
    ///
    /// A `DbPool` if successful
    ///
    /// # Notes
    ///
    /// * This function will panic if the `DATABASE_USERNAME` environment variable is not set.
    /// * This function will panic if the `DATABASE_PASSWORD` environment variable is not set.
    /// * This function will panic if the `DATABASE_HOST` environment variable is not set.
    /// * This function will panic if the `DATABASE_PORT` environment variable is not set.
    /// * This function will panic if the `DATABASE_NAME` environment variable is not set.
    /// * This function should only be called once in the application.
    pub fn establish_connection_pool() -> Self {
        tracing::info!("Establishing connection pool.");

        dotenv().ok();

        let database_username =
            env::var("DATABASE_USERNAME").expect("DATABASE_USERNAME must be set");
        let database_password =
            env::var("DATABASE_PASSWORD").expect("DATABASE_PASSWORD must be set");
        let database_host = env::var("DATABASE_HOST").expect("DATABASE_HOST must be set");
        let database_port = env::var("DATABASE_PORT").expect("DATABASE_PORT must be set");
        let database_name = env::var("DATABASE_NAME").expect("DATABASE_NAME must be set");

        let manager = ConnectionManager::<PgConnection>::new(format!(
            "postgres://{}:{}@{}:{}/{}",
            database_username, database_password, database_host, database_port, database_name
        ));
        Self {
            connection: Pool::builder()
                .build(manager)
                .expect("Failed to create pool."),
        }
    }
    /// Function to get a connection from the pool
    ///
    /// # Returns
    ///
    /// A `DbConn` if successful, otherwise an `AppError`
    pub fn get(&self) -> Result<DbConn, AppError> {
        self.connection.get().map_err(|e| {
            tracing::error!("Failed to get connection from the pool ({e}).");
            AppError::DbConnectionError
        })
    }
}
