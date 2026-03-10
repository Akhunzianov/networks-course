use axum::Json;
use serde::Serialize;
use tracing::info;

#[derive(Serialize)]
pub struct HealthResponse {
    status: &'static str,
}

pub async fn health() -> Json<HealthResponse> {
    info!("health check");
    Json(HealthResponse { status: "ok" })
}
