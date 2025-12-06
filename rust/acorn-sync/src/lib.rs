#![allow(dead_code)]

use acorn_core::{AcornError, AcornResult, Tree};
use tracing::instrument;

#[derive(Debug, Default)]
pub struct SyncClient;

impl SyncClient {
    #[instrument(skip(self, _tree))]
    pub async fn synchronize<T>(&self, _tree: &Tree<T>) -> AcornResult<()> {
        Err(AcornError::NotImplemented)
    }
}
