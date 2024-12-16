use diesel::pg::PgConnection;
use diesel::r2d2::{self, ConnectionManager, Pool, PooledConnection};
use dotenv::dotenv;
use std::env;

// Define a type alias for the connection pool
pub type DbPool = r2d2::Pool<ConnectionManager<PgConnection>>;

// Function to create and return a connection pool
pub fn establish_connection_pool() -> DbPool {
    tracing::info!("Establishing connection pool.");

    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let manager = ConnectionManager::<PgConnection>::new(database_url);
    Pool::builder()
        .build(manager)
        .expect("Failed to create pool.")
}

/// Function to get a connection from the pool
///
/// # Arguments
///
/// * `pool` - A reference to the connection pool
///
/// # Returns
///
/// An `Option` containing a connection if successful, otherwise `None`
///
/// # Example
///
/// ```
/// let pool = establish_connection_pool();
/// let connection = get_connection(&pool);
/// ```
pub fn get_connection(pool: &DbPool) -> Option<PooledConnection<ConnectionManager<PgConnection>>> {
    if let Ok(connection) = pool.get() {
        Some(connection)
    } else {
        tracing::error!("Failed to get connection from the pool.");
        None
    }
}
