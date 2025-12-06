#![allow(dead_code)]

use std::collections::HashMap;
use std::sync::Arc;

use acorn_core::{
    AcornError, AcornResult, BranchId, CapabilityAdvertiser, HistoryEvent, Nut, Trunk,
    TrunkCapability,
};
use parking_lot::RwLock;

#[derive(Debug, Clone, Default)]
pub struct MemoryTrunk {
    inner: Arc<RwLock<Inner>>,
}

#[derive(Debug, Default)]
struct Inner {
    data: HashMap<(BranchId, String), Vec<u8>>,
    history: HashMap<BranchId, Vec<HistoryEvent<Vec<u8>>>>,
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
            .data
            .get(&(branch.clone(), key.to_string()))
            .cloned()
            .map(|value| Nut { value }))
    }

    fn put(&self, branch: &BranchId, key: &str, nut: Nut<Vec<u8>>) -> AcornResult<()> {
        let mut guard = self.inner.write();
        guard
            .history
            .entry(branch.clone())
            .or_default()
            .push(HistoryEvent::Put {
                key: key.to_string(),
                nut: Nut {
                    value: nut.value.clone(),
                },
            });
        guard.data.insert((branch.clone(), key.to_string()), nut.value);
        Ok(())
    }

    fn delete(&self, branch: &BranchId, key: &str) -> AcornResult<()> {
        let mut guard = self.inner.write();
        guard
            .data
            .remove(&(branch.clone(), key.to_string()))
            .map(|_| {
                guard
                    .history
                    .entry(branch.clone())
                    .or_default()
                    .push(HistoryEvent::Delete {
                        key: key.to_string(),
                    });
            })
            .ok_or_else(|| AcornError::Trunk("missing key".into()))?;
        Ok(())
    }
}

impl CapabilityAdvertiser for MemoryTrunk {
    fn capabilities(&self) -> &'static [TrunkCapability] {
        &[TrunkCapability::History]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn put_get_delete_round_trip() {
        let trunk = MemoryTrunk::new();
        let branch = BranchId::new("main");

        trunk
            .put(&branch, "key", Nut { value: b"hello".to_vec() })
            .unwrap();

        let fetched = trunk.get(&branch, "key").unwrap().unwrap();
        assert_eq!(fetched.value, b"hello".to_vec());

        trunk.delete(&branch, "key").unwrap();
        assert!(trunk.get(&branch, "key").unwrap().is_none());
    }

    #[test]
    fn tracks_history_for_put_and_delete() {
        let trunk = MemoryTrunk::new();
        let branch = BranchId::new("history");

        trunk
            .put(&branch, "key", Nut { value: b"bytes".to_vec() })
            .unwrap();
        trunk.delete(&branch, "key").unwrap();

        let inner = trunk.inner.read();
        let history = inner.history.get(&branch).cloned().unwrap();
        assert_eq!(history.len(), 2);
        match &history[0] {
            HistoryEvent::Put { key, nut } => {
                assert_eq!(key, "key");
                assert_eq!(nut.value, b"bytes".to_vec());
            }
            _ => panic!("expected put history"),
        }
        match &history[1] {
            HistoryEvent::Delete { key } => assert_eq!(key, "key"),
            _ => panic!("expected delete history"),
        }
    }
}
