use acorn_core::BranchId;
use acorn_sync::{SyncApplyRequest, SyncApplyResponse, SyncErrorResponse, SyncMutation, SyncPullResponse};
use acorn_trunk_mem::MemoryTrunk;
use axum::{
    extract::{ws::WebSocketUpgrade, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use parking_lot::Mutex;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::{select, sync::broadcast};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .with(tracing_subscriber::fmt::layer())
        .init();

    let state = AppState::new();

    let app = Router::new()
        .route("/health", get(health))
        .route("/sync/apply", post(apply_batch))
        .route("/sync/pull", get(pull_batch))
        .route("/sync/stream", get(stream_updates))
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

#[derive(Clone)]
struct AppState {
    trunk: Arc<Mutex<MemoryTrunk>>,
    notifier: broadcast::Sender<String>,
}

impl AppState {
    fn new() -> Self {
        let (tx, _rx) = broadcast::channel(64);
        Self {
            trunk: Arc::new(MemoryTrunk::new()),
            notifier: tx,
        }
    }
}

async fn apply_batch(
    State(state): State<AppState>,
    Json(payload): Json<SyncApplyRequest>,
) -> Result<Json<SyncApplyResponse>, (StatusCode, Json<SyncErrorResponse>)> {
    let trunk = state.trunk.lock();
    let mut applied = 0usize;

    for op in &payload.batch.operations {
        match op {
            SyncMutation::Put { key, value } => {
                let _ = trunk.put(
                    &payload.batch.branch,
                    key,
                    acorn_core::Nut { value: value.clone() },
                );
                applied += 1;
            }
            SyncMutation::Delete { key } => {
                let _ = trunk.delete(&payload.batch.branch, key);
                applied += 1;
            }
        }
    }

    let _ = state.notifier.send(format!(
        "branch:{} applied:{}",
        payload.batch.branch.as_str(),
        applied
    ));

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
    let trunk = state.trunk.lock();
    let mut ops = Vec::new();

    for key in trunk.keys(&branch) {
        if let Ok(Some(nut)) = trunk.get(&branch, &key) {
            ops.push(SyncMutation::Put {
                key: key.clone(),
                value: nut.value,
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

async fn stream_updates(State(state): State<AppState>, ws: WebSocketUpgrade) -> impl IntoResponse {
    let receiver = state.notifier.subscribe();
    ws.on_upgrade(move |socket| handle_ws(socket, receiver))
}

async fn handle_ws(mut socket: axum::extract::ws::WebSocket, mut receiver: broadcast::Receiver<String>) {
    use axum::extract::ws::Message;
    loop {
        select! {
            maybe_msg = receiver.recv() => {
                match maybe_msg {
                    Ok(text) => {
                        if socket.send(Message::Text(text)).await.is_err() {
                            break;
                        }
                    }
                    Err(broadcast::error::RecvError::Closed) => break,
                    Err(broadcast::error::RecvError::Lagged(_)) => continue,
                }
            }
            incoming = socket.recv() => {
                match incoming {
                    Some(Ok(Message::Close(_))) => break,
                    Some(Ok(_)) => continue,
                    _ => break,
                }
            }
        }
    }
}
