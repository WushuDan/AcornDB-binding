use axum::{
    extract::{Query, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use parking_lot::Mutex;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use acorn_core::BranchId;
use acorn_sync::{SyncApplyRequest, SyncApplyResponse, SyncErrorResponse, SyncMutation, SyncPullResponse};

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .with(tracing_subscriber::fmt::layer())
        .init();

    let state = AppState::default();

    let app = Router::new()
        .route("/health", get(health))
        .route("/sync/apply", post(apply_batch))
        .route("/sync/pull", get(pull_batch))
        .with_state(state);

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

#[derive(Clone, Default)]
struct AppState {
    inner: Arc<Mutex<HashMap<(BranchId, String), Vec<u8>>>>,
}

async fn apply_batch(
    State(state): State<AppState>,
    Json(payload): Json<SyncApplyRequest>,
) -> Result<Json<SyncApplyResponse>, (StatusCode, Json<SyncErrorResponse>)> {
    let mut store = state.inner.lock();
    let mut applied = 0usize;

    for op in &payload.batch.operations {
        match op {
            SyncMutation::Put { key, value } => {
                store.insert((payload.batch.branch.clone(), key.clone()), value.clone());
                applied += 1;
            }
            SyncMutation::Delete { key } => {
                store.remove(&(payload.batch.branch.clone(), key.clone()));
                applied += 1;
            }
        }
    }

    Ok(Json(SyncApplyResponse {
        applied,
        conflicts: 0,
    }))
}

#[derive(Debug, serde::Deserialize)]
struct PullQuery {
    branch: Option<String>,
}

async fn pull_batch(
    State(state): State<AppState>,
    Query(query): Query<PullQuery>,
) -> Result<Json<SyncPullResponse>, (StatusCode, Json<SyncErrorResponse>)> {
    let branch = BranchId::new(query.branch.unwrap_or_else(|| "default".into()));
    let store = state.inner.lock();
    let mut ops = Vec::new();

    for ((b, key), value) in store.iter() {
        if b == &branch {
            ops.push(SyncMutation::Put {
                key: key.clone(),
                value: value.clone(),
            });
        }
    }

    Ok(Json(SyncPullResponse {
        batch: acorn_sync::SyncBatch {
            branch,
            operations: ops,
        },
    }))
}
