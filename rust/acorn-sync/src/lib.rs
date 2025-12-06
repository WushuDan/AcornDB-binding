#![allow(dead_code)]

use acorn_core::{AcornError, AcornResult, BranchId, Tree, Trunk};
use tracing::instrument;

/// HTTP/WebSocket sync endpoint target.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SyncEndpoint {
    pub url: String,
    pub branch: BranchId,
}

#[derive(Debug, Clone)]
pub enum SyncError {
    Network(String),
    Protocol(String),
    Conflict(String),
    Storage(String),
    VersionConflict(String),
    MissingKey(String),
    Unknown(String),
}

/// Stub sync client facade; will orchestrate pull/push/sync with retries.
#[derive(Debug, Default)]
pub struct SyncClient;

impl SyncClient {
    #[instrument(skip(self, _tree))]
    pub async fn synchronize<T, S>(&self, _tree: &Tree<T, S>, _endpoint: &SyncEndpoint) -> AcornResult<()>
    where
        T: Clone + Send + Sync + 'static + std::fmt::Debug,
        S: Trunk<T> + Send + Sync,
    {
        Err(AcornError::NotImplemented)
    }

    #[instrument(skip(self))]
    pub async fn pull<T, S>(&self, _tree: &Tree<T, S>, _endpoint: &SyncEndpoint) -> AcornResult<()>
    where
        T: Clone + Send + Sync + 'static + std::fmt::Debug,
        S: Trunk<T> + Send + Sync,
    {
        Err(AcornError::NotImplemented)
    }

    #[instrument(skip(self))]
    pub async fn push<T, S>(&self, _tree: &Tree<T, S>, _endpoint: &SyncEndpoint) -> AcornResult<()>
    where
        T: Clone + Send + Sync + 'static + std::fmt::Debug,
        S: Trunk<T> + Send + Sync,
    {
        Err(AcornError::NotImplemented)
    }
}

/// Subscription placeholder; will be backed by streaming updates.
#[derive(Debug, Default)]
pub struct Subscription;

impl Subscription {
    pub async fn subscribe<T, S>(_tree: &Tree<T, S>) -> AcornResult<Self>
    where
        T: Clone + Send + Sync + 'static,
        S: Trunk<T> + Send + Sync,
    {
        Ok(Subscription)
    }

    pub async fn next_event(&self) -> AcornResult<Option<SyncEvent>> {
        Err(AcornError::NotImplemented)
    }
}

/// Streaming events emitted during sync.
#[derive(Debug, Clone)]
pub enum SyncEvent {
    Applied { key: String },
    Conflict { key: String },
    Heartbeat,
}

/// Mutations applied during sync; optional version enables optimistic concurrency.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum SyncMutation {
    Put {
        key: String,
        value: Vec<u8>,
        version: Option<u64>,
    },
    Delete {
        key: String,
        version: Option<u64>,
    },
}

/// Batch of operations scoped to a branch.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SyncBatch {
    pub branch: BranchId,
    pub operations: Vec<SyncMutation>,
}

/// Payload for apply requests.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SyncApplyRequest {
    pub batch: SyncBatch,
}

/// Result of applying a batch on the server.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SyncApplyResponse {
    pub applied: usize,
    pub conflicts: Vec<SyncConflict>,
}

/// Response payload for pull requests (ops plus version snapshot).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SyncPullResponse {
    pub batch: SyncBatch,
    pub versions: Vec<(String, u64)>,
}

/// Conflict surface returned by sync operations.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SyncConflict {
    pub key: String,
    pub remote_value: Option<Vec<u8>>,
    pub local_value: Option<Vec<u8>>,
    pub remote_version: Option<u64>,
    pub local_version: Option<u64>,
    pub kind: SyncConflictKind,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum SyncConflictKind {
    VersionMismatch,
    MissingKey,
    Unknown,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SyncApplyResult {
    pub applied: usize,
    pub conflicts: Vec<SyncConflict>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SyncErrorResponse {
    pub error: String,
}

#[derive(Debug, Clone)]
pub struct SyncResult {
    pub applied: usize,
    pub conflicts: usize,
}
