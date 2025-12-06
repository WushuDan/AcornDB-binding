#![allow(dead_code)]

use acorn_core::{AcornError, AcornResult, BranchId, Tree, Trunk};
use tracing::instrument;

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
    Unknown(String),
}

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

#[derive(Debug, Clone)]
pub enum SyncEvent {
    Applied { key: String },
    Conflict { key: String },
    Heartbeat,
}

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

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SyncBatch {
    pub branch: BranchId,
    pub operations: Vec<SyncMutation>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SyncApplyRequest {
    pub batch: SyncBatch,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SyncApplyResponse {
    pub applied: usize,
    pub conflicts: usize,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SyncPullResponse {
    pub batch: SyncBatch,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SyncConflict {
    pub key: String,
    pub remote_value: Option<Vec<u8>>,
    pub local_value: Option<Vec<u8>>,
    pub remote_version: Option<u64>,
    pub local_version: Option<u64>,
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
