#![allow(dead_code)]

use acorn_core::{AcornError, AcornResult, BranchId, Tree, Trunk};
use tracing::instrument;

#[derive(Debug, Clone)]
pub struct SyncEndpoint {
    pub url: String,
    pub branch: BranchId,
}

#[derive(Debug, Default)]
pub struct SyncClient;

impl SyncClient {
    #[instrument(skip(self, _tree))]
    pub async fn synchronize<T, S>(&self, _tree: &Tree<T, S>, _endpoint: &SyncEndpoint) -> AcornResult<()>
    where
        T: Clone + Send + Sync + 'static,
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
}
