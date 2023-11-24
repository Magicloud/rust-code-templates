use axum::Json;
use serde_json::{ Value, json };

pub async fn heart_beat() -> String {
    "200 OK".to_string()
}

pub async fn status() -> Json<Value> {
    Json(json!({}))
}
