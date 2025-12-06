use acorn_core::BranchId;
use acorn_sync::{SyncApplyRequest, SyncApplyResponse, SyncErrorResponse, SyncMutation, SyncPullResponse};
use acorn_trunk_file::FileTrunk;
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

#[derive(Debug, Clone, serde::Serialize)]
struct StreamEvent {
    branch: String,
    applied: usize,
    conflicts: usize,
}

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
    trunk: Arc<Mutex<BackendTrunk>>,
    versions: Arc<Mutex<HashMap<(BranchId, String), u64>>>,
    notifier: broadcast::Sender<StreamEvent>,
}

impl AppState {
    fn new() -> Self {
        let (tx, _rx) = broadcast::channel(64);
        Self {
            trunk: Arc::new(BackendTrunk::from_env()),
            versions: Arc::new(Mutex::new(HashMap::new())),
            notifier: tx,
        }
    }
}

#[derive(Debug)]
enum BackendTrunk {
    Memory(MemoryTrunk),
    File(FileTrunk),
}

impl BackendTrunk {
    fn from_env() -> Self {
        if let Ok(path) = std::env::var("ACORN_TRUNK_FILE") {
            BackendTrunk::File(FileTrunk::with_history_and_ttl(path))
        } else {
            BackendTrunk::Memory(MemoryTrunk::new())
        }
    }

    fn put(&self, branch: &BranchId, key: &str, value: Vec<u8>) -> Result<(), acorn_core::AcornError> {
        match self {
            BackendTrunk::Memory(t) => t.put(branch, key, acorn_core::Nut { value }),
            BackendTrunk::File(t) => t.put(branch, key, acorn_core::Nut { value }),
        }
    }

    fn delete(&self, branch: &BranchId, key: &str) -> Result<(), acorn_core::AcornError> {
        match self {
            BackendTrunk::Memory(t) => t.delete(branch, key),
            BackendTrunk::File(t) => t.delete(branch, key),
        }
    }

    fn get(&self, branch: &BranchId, key: &str) -> Option<Vec<u8>> {
        match self {
            BackendTrunk::Memory(t) => t.get(branch, key).ok().flatten().map(|n| n.value),
            BackendTrunk::File(t) => t.get(branch, key).ok().flatten().map(|n| n.value),
        }
    }

    fn keys(&self, branch: &BranchId) -> Vec<String> {
        match self {
            BackendTrunk::Memory(t) => t.keys(branch),
            BackendTrunk::File(t) => t.keys(branch),
        }
    }
}

async fn apply_batch(
    State(state): State<AppState>,
    Json(payload): Json<SyncApplyRequest>,
) -> Result<Json<SyncApplyResponse>, (StatusCode, Json<SyncErrorResponse>)> {
    let trunk = state.trunk.lock();
    let mut versions = state.versions.lock();
    let mut applied = 0usize;
    let mut conflicts = 0usize;

    for op in &payload.batch.operations {
        match op {
            SyncMutation::Put { key, value, version } => {
                let current_version = versions
                    .get(&(payload.batch.branch.clone(), key.clone()))
                    .copied();
                if let Some(expected) = version {
                    if let Some(existing) = current_version {
                        if existing != *expected {
                            conflicts += 1;
                            continue;
                        }
                    }
                }
                if let Err(e) = trunk.put(&payload.batch.branch, key, value.clone()) {
                    return Err((
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(SyncErrorResponse { error: e.to_string() }),
                    ));
                }
                let next_version = current_version.unwrap_or(0).saturating_add(1);
                versions.insert((payload.batch.branch.clone(), key.clone()), next_version);
                applied += 1;
            }
            SyncMutation::Delete { key, version } => {
                let current_version = versions
                    .get(&(payload.batch.branch.clone(), key.clone()))
                    .copied();
                if let Some(expected) = version {
                    if let Some(existing) = current_version {
                        if existing != *expected {
                            conflicts += 1;
                            continue;
                        }
                    }
                }
                if trunk.get(&payload.batch.branch, key).is_none() {
                    conflicts += 1;
                    continue;
                }
                if let Err(e) = trunk.delete(&payload.batch.branch, key) {
                    return Err((
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(SyncErrorResponse { error: e.to_string() }),
                    ));
                }
                versions.remove(&(payload.batch.branch.clone(), key.clone()));
                applied += 1;
            }
        }
    }

    let _ = state.notifier.send(StreamEvent {
        branch: payload.batch.branch.as_str().to_string(),
        applied,
        conflicts,
    });

    Ok(Json(SyncApplyResponse { applied, conflicts }))
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
        if let Some(value) = trunk.get(&branch, &key) {
            ops.push(SyncMutation::Put {
                key: key.clone(),
                value,
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

async fn handle_ws(mut socket: axum::extract::ws::WebSocket, mut receiver: broadcast::Receiver<StreamEvent>) {
    use axum::extract::ws::Message;
    loop {
        select! {
            maybe_msg = receiver.recv() => {
                match maybe_msg {
                    Ok(evt) => {
                        let payload = serde_json::to_string(&evt).unwrap_or_else(|_| "{}".into());
                        if socket.send(Message::Text(payload)).await.is_err() {
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
