use axum::{
    routing::{get, post},
    Json, Router,
};
use std::net::SocketAddr;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use acorn_sync::{SyncApplyRequest, SyncApplyResponse, SyncErrorResponse, SyncPullResponse};
use std::net::SocketAddr;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .with(tracing_subscriber::fmt::layer())
        .init();

    let app = Router::new()
        .route("/health", get(health))
        .route("/sync/apply", post(apply_batch))
        .route("/sync/pull", get(pull_batch));

    let addr: SocketAddr = "0.0.0.0:8080".parse().unwrap();
    tracing::info!("acorn-sync-server listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn health() -> &'static str {
    "ok"
}

async fn apply_batch(Json(payload): Json<SyncApplyRequest>) -> Json<SyncApplyResponse> {
    let applied = payload.batch.operations.len();
    Json(SyncApplyResponse {
        applied,
        conflicts: 0,
    })
}

async fn pull_batch() -> Json<SyncPullResponse> {
    Json(SyncPullResponse {
        batch: acorn_sync::SyncBatch {
            branch: acorn_core::BranchId::new("default"),
            operations: Vec::new(),
        },
    })
}
