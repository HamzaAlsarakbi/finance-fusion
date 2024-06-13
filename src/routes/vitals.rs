use axum::{ routing::get, Json, Router };
use bson::doc;
use serde::{ Deserialize, Serialize };
use tracing::debug;

use crate::errors::Error;

pub fn create_route() -> Router {
  Router::new().route("/vitals", get(get_vitals))
}

async fn get_vitals() -> Result<Json<Vitals>, Error> {
  debug!("Returning vitals");
  Ok(
    Json(Vitals {
      status: "ok".to_owned(),
    })
  )
}

#[derive(Serialize, Deserialize, Debug)]
struct Vitals {
  status: String,
}
