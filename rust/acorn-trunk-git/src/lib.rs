#![allow(dead_code)]

use std::collections::HashMap;
use std::sync::Arc;
use std::time::SystemTime;

use acorn_core::{
    AcornError, AcornResult, BranchId, CapabilityAdvertiser, HistoryEvent, HistoryProvider, Nut, Trunk,
    TrunkCapability, Ttl, TtlProvider,
};
use parking_lot::RwLock;

#[derive(Debug, Default, Clone)]
pub struct GitTrunk {
    inner: Arc<RwLock<Inner>>,
}

#[derive(Debug, Default)]
struct Inner {
    data: HashMap<(BranchId, String), Vec<u8>>,
    history: HashMap<BranchId, Vec<HistoryEvent<Vec<u8>>>>,
    ttl: HashMap<(BranchId, String), SystemTime>,
}

impl GitTrunk {
    pub fn new() -> Self {
        GitTrunk {
            inner: Arc::new(RwLock::new(Inner::default())),
        }
    }

    pub fn connect(&self) -> AcornResult<()> {
        Ok(())
    }
}

impl Trunk<Vec<u8>> for GitTrunk {
    fn get(&self, branch: &BranchId, key: &str) -> AcornResult<Option<Nut<Vec<u8>>>> {
        let mut guard = self.inner.write();
        if let Some(expires_at) = guard.ttl.get(&(branch.clone(), key.to_string())) {
            if SystemTime::now() >= *expires_at {
                guard.ttl.remove(&(branch.clone(), key.to_string()));
                guard.data.remove(&(branch.clone(), key.to_string()));
                guard
                    .history
                    .entry(branch.clone())
                    .or_default()
                    .push(HistoryEvent::Delete { key: key.to_string() });
                return Ok(None);
            }
        }

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
                    .push(HistoryEvent::Delete { key: key.to_string() });
            })
            .ok_or_else(|| AcornError::Trunk("missing key".into()))?;
        Ok(())
    }
}

impl CapabilityAdvertiser for GitTrunk {
    fn capabilities(&self) -> &'static [TrunkCapability] {
        &[TrunkCapability::History, TrunkCapability::Ttl]
    }
}

impl TtlProvider<Vec<u8>> for GitTrunk {
    fn put_with_ttl(&self, branch: &BranchId, key: &str, nut: Nut<Vec<u8>>, ttl: Ttl) -> AcornResult<()> {
        let mut guard = self.inner.write();
        guard
            .ttl
            .insert((branch.clone(), key.to_string()), ttl.expires_at);
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
}

impl HistoryProvider<Vec<u8>> for GitTrunk {
    fn history(&self, branch: &BranchId) -> AcornResult<Vec<HistoryEvent<Vec<u8>>>> {
        let guard = self.inner.read();
        Ok(guard.history.get(branch).cloned().unwrap_or_else(Vec::new))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use acorn_core::{CapabilityAdvertiser, EncodedTree, JsonCodec};
    use serde::{Deserialize, Serialize};

    #[cfg(feature = "contract-tests")]
    use acorn_test_harness::TrunkContract;

    #[cfg(feature = "contract-tests")]
    #[test]
    fn contract_round_trip_history_ttl() {
        let trunk = GitTrunk::new();
        TrunkContract::round_trip_bytes(&trunk).unwrap();
        TrunkContract::assert_capabilities(&trunk, &[TrunkCapability::History, TrunkCapability::Ttl]);
        TrunkContract::ttl_expiry(&trunk).unwrap();
        TrunkContract::history_put_delete(&trunk).unwrap();
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    struct Demo {
        v: String,
    }

    #[test]
    fn encoded_tree_round_trip() {
        let trunk = GitTrunk::new();
        let tree = EncodedTree::new(BranchId::new("enc"), trunk, JsonCodec);
        let val = Demo { v: "git".into() };
        tree.put("k", Nut { value: val.clone() }).unwrap();
        let fetched = tree.get("k").unwrap().unwrap();
        assert_eq!(fetched.value, val);
    }
}
