#![allow(dead_code)]

use std::collections::HashMap;
use std::sync::Arc;

use acorn_core::{AcornError, AcornResult, BranchId, Nut, Trunk};
use parking_lot::RwLock;

#[derive(Debug, Clone, Default)]
pub struct MemoryTrunk {
    inner: Arc<RwLock<HashMap<(BranchId, String), Vec<u8>>>>,
}

impl MemoryTrunk {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn health_check(&self) -> AcornResult<()> {
        Ok(())
    }
}

impl Trunk<Vec<u8>> for MemoryTrunk {
    fn get(&self, branch: &BranchId, key: &str) -> AcornResult<Option<Nut<Vec<u8>>>> {
        let guard = self.inner.read();
        Ok(guard
            .get(&(branch.clone(), key.to_string()))
            .cloned()
            .map(|value| Nut { value }))
    }

    fn put(&self, branch: &BranchId, key: &str, nut: Nut<Vec<u8>>) -> AcornResult<()> {
        let mut guard = self.inner.write();
        guard.insert((branch.clone(), key.to_string()), nut.value);
        Ok(())
    }

    fn delete(&self, branch: &BranchId, key: &str) -> AcornResult<()> {
        let mut guard = self.inner.write();
        guard
            .remove(&(branch.clone(), key.to_string()))
            .map(|_| ())
            .ok_or_else(|| AcornError::Trunk("missing key".into()))
    }
}
