use axum::{extract::State, Json};
use serde_json::{json, Value};

use crate::server::AppState;

pub async fn info(State(_state): State<AppState>) -> Json<Value> {
    Json(json!({
        "name": "cage-bro",
        "version": env!("CARGO_PKG_VERSION"),
        "runtime": "process",
        "status": "running",
    }))
}
