use std::sync::Arc;

use clap::Parser;
use tokio::signal::unix::{signal, SignalKind};
use tokio::sync::oneshot;

use crate::api::api;
use crate::database::connection::DbPool;
use crate::errors::AppError;
/// Compile-time version string. Defaults to 0.0.0-a.0-0-g0 if git is not available
pub const VERSION: &str =
    git_version::git_version!(args = ["--always", "--long"], fallback = "0.0.0-a.0-0-g0");

/// A server that listens to Finance Fusion's output and generates analytics of various types.
#[derive(Parser, Debug)]
#[command(version = VERSION)]
#[command(author, about, long_about = None)]
pub struct Args {
    /// Directory in which the configuration file is stored
    #[arg(short, long, default_value = "/etc/finance-fusion")]
    pub config_dir: String,

    /// The rest port to listen on
    #[arg(short, long, default_value = "5000")]
    pub rest_port: u16,
}

/// Asynchronously runs the server with the provided arguments.
///
/// # Arguments
///
/// * `args` - The arguments for the server, including the REST port to listen on.
///
/// # Returns
///
/// * `Result<()>` - Returns `Ok(())` if the server ran successfully. Returns `Err(e)` if an error occurred.
///
/// # Behavior
///
/// This function first creates a one-shot channel for shutdown signal communication.
/// It then spawns a new asynchronous task to start the REST server, listening on the provided port.
///
/// The function also sets up Unix signal listeners for SIGINT (Ctrl+C) and SIGTERM (termination request).
/// If either of these signals is received, a shutdown signal is sent to the server.
///
/// The function then waits for either the server task to complete, or for a shutdown signal to be received.
/// If a shutdown signal is received, it sends a message over the one-shot channel to signal the
/// rest server to shut down.
///
/// If the server task completes (either normally or due to an error), the function returns.
/// If a shutdown signal is received, the function waits for the server task to shut down before returning.
///
/// # Errors
/// If an error occurs while starting the REST server, it is converted to an `anyhow::Error` and
/// returned.
pub async fn run(args: Args, pool: Arc<DbPool>) -> Result<(), AppError> {
    // Create a one-shot channel for shutdown signal communication
    let (tx, rx) = oneshot::channel();

    // Spawn a new asynchronous task to start the REST server
    let rest_server_task =
        tokio::spawn(async move { api::start_rest_server(args.rest_port, rx, pool).await });

    let mut sigint = signal(SignalKind::interrupt())?;
    let mut sigterm = signal(SignalKind::terminate())?;

    // Wait for either the REST server task to complete, or for a shutdown signal to be received
    tokio::select! {
      _ = rest_server_task => {},
      _ = sigint.recv() => {
        println!("Received SIGINT");
        let _ = tx.send(());
      },
      _ = sigterm.recv() => {
        println!("Received SIGTERM");
        let _ = tx.send(());
      },
    }

    Ok(())
}
