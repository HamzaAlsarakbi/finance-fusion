use anyhow::Result;
use clap::Parser;
use tracing::{error, info};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod config;
mod errors;

mod api;
mod database;
mod routes;

use config::config::{run, Args, VERSION};

#[tokio::main]
async fn main() -> Result<()> {
  // Set up tracing, which is used for logging.
  tracing_subscriber::registry()
    .with(
      tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        // axum logs rejections from built-in extractors with the `axum::rejection`
        // target, at `TRACE` level. `axum::rejection=trace` enables showing those events
        "finance_fusion=info,tower_http=info,axum::rejection=trace".into()
      }),
    )
    .with(tracing_subscriber::fmt::layer())
    .init();

  // Parse command line arguments
  let args = Args::parse();

  info!("Starting Finance Fusion Server v{VERSION}");

  match run(args).await {
    Ok(()) => info!("Exiting Finance Fusion Server"),
    Err(e) => error!("Server encountered an error: {e}"),
  }

  Ok(())
}
