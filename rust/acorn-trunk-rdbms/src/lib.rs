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
pub struct RdbmsTrunk {
    inner: Arc<RwLock<Inner>>,
}

#[derive(Debug, Default)]
struct Inner {
    data: HashMap<(BranchId, String), Vec<u8>>,
    history: HashMap<BranchId, Vec<HistoryEvent<Vec<u8>>>>,
    ttl: HashMap<(BranchId, String), SystemTime>,
    versions: HashMap<(BranchId, String), u64>,
}

impl RdbmsTrunk {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(Inner::default())),
        }
    }

    pub fn connect(&self) -> AcornResult<()> {
        Ok(())
    }

    pub fn current_version(&self, branch: &BranchId, key: &str) -> Option<u64> {
        let guard = self.inner.read();
        guard.versions.get(&(branch.clone(), key.to_string())).copied()
    }
}

impl Trunk<Vec<u8>> for RdbmsTrunk {
    fn get(&self, branch: &BranchId, key: &str) -> AcornResult<Option<Nut<Vec<u8>>>> {
        let mut guard = self.inner.write();
        if let Some(expires_at) = guard.ttl.get(&(branch.clone(), key.to_string())) {
            if SystemTime::now() >= *expires_at {
                guard.ttl.remove(&(branch.clone(), key.to_string()));
                guard.data.remove(&(branch.clone(), key.to_string()));
                guard.versions.remove(&(branch.clone(), key.to_string()));
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
        let next_version = guard
            .versions
            .get(&(branch.clone(), key.to_string()))
            .copied()
            .unwrap_or(0)
            .saturating_add(1);
        guard
            .versions
            .insert((branch.clone(), key.to_string()), next_version);
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
                guard.versions.remove(&(branch.clone(), key.to_string()));
                guard
                    .history
                    .entry(branch.clone())
                    .or_default()
                    .push(HistoryEvent::Delete { key: key.to_string() });
            })
            .ok_or_else(|| AcornError::MissingKey(key.to_string()))?;
        Ok(())
    }

    fn version(&self, branch: &BranchId, key: &str) -> Option<u64> {
        self.current_version(branch, key)
    }

    fn put_if_version(
        &self,
        branch: &BranchId,
        key: &str,
        expected: Option<u64>,
        nut: Nut<Vec<u8>>,
    ) -> AcornResult<()> {
        let mut guard = self.inner.write();
        let current = guard.versions.get(&(branch.clone(), key.to_string())).copied();
        if let Some(expected) = expected {
            if current != Some(expected) {
                return Err(AcornError::VersionConflict {
                    expected: Some(expected),
                    actual: current,
                });
            }
        }
        let next_version = current.unwrap_or(0).saturating_add(1);
        guard
            .versions
            .insert((branch.clone(), key.to_string()), next_version);
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

    fn delete_if_version(&self, branch: &BranchId, key: &str, expected: Option<u64>) -> AcornResult<()> {
        let mut guard = self.inner.write();
        let current = guard.versions.get(&(branch.clone(), key.to_string())).copied();
        if let Some(expected) = expected {
            if current != Some(expected) {
                return Err(AcornError::VersionConflict {
                    expected: Some(expected),
                    actual: current,
                });
            }
        }
        guard
            .data
            .remove(&(branch.clone(), key.to_string()))
            .map(|_| {
                guard.versions.remove(&(branch.clone(), key.to_string()));
                guard
                    .history
                    .entry(branch.clone())
                    .or_default()
                    .push(HistoryEvent::Delete { key: key.to_string() });
            })
            .ok_or_else(|| AcornError::MissingKey(key.to_string()))?;
        Ok(())
    }
}

impl CapabilityAdvertiser for RdbmsTrunk {
    fn capabilities(&self) -> &'static [TrunkCapability] {
        &[TrunkCapability::History, TrunkCapability::Ttl, TrunkCapability::Versions]
    }
}

impl TtlProvider<Vec<u8>> for RdbmsTrunk {
    fn put_with_ttl(&self, branch: &BranchId, key: &str, nut: Nut<Vec<u8>>, ttl: Ttl) -> AcornResult<()> {
        let mut guard = self.inner.write();
        let next_version = guard
            .versions
            .get(&(branch.clone(), key.to_string()))
            .copied()
            .unwrap_or(0)
            .saturating_add(1);
        guard
            .versions
            .insert((branch.clone(), key.to_string()), next_version);
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

impl HistoryProvider<Vec<u8>> for RdbmsTrunk {
    fn history(&self, branch: &BranchId) -> AcornResult<Vec<HistoryEvent<Vec<u8>>>> {
        let guard = self.inner.read();
        Ok(guard.history.get(branch).cloned().unwrap_or_else(Vec::new))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use acorn_core::{EncodedTree, JsonCodec};
    use acorn_core::CapabilityAdvertiser;
    use serde::{Deserialize, Serialize};

    #[cfg(feature = "contract-tests")]
    use acorn_test_harness::TrunkContract;

    #[cfg(feature = "contract-tests")]
    #[test]
    fn contract_round_trip_history_ttl() {
        let trunk = RdbmsTrunk::new();
        TrunkContract::round_trip_bytes(&trunk).unwrap();
        TrunkContract::assert_capabilities(
            &trunk,
            &[TrunkCapability::History, TrunkCapability::Ttl, TrunkCapability::Versions],
        );
        TrunkContract::ttl_expiry(&trunk).unwrap();
        TrunkContract::history_put_delete(&trunk).unwrap();
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    struct Demo {
        v: String,
    }

    #[test]
    fn encoded_tree_round_trip() {
        let trunk = RdbmsTrunk::new();
        let tree = EncodedTree::new(BranchId::new("enc"), trunk, JsonCodec);
        let val = Demo { v: "rdbms".into() };
        tree.put("k", Nut { value: val.clone() }).unwrap();
        let fetched = tree.get("k").unwrap().unwrap();
        assert_eq!(fetched.value, val);
    }

    #[test]
    fn tracks_versions() {
        let trunk = RdbmsTrunk::new();
        let branch = BranchId::new("versions");

        trunk
            .put(
                &branch,
                "key",
                Nut {
                    value: b"v1".to_vec(),
                },
            )
            .unwrap();
        assert_eq!(trunk.current_version(&branch, "key"), Some(1));

        trunk
            .put(
                &branch,
                "key",
                Nut {
                    value: b"v2".to_vec(),
                },
            )
            .unwrap();
        assert_eq!(trunk.current_version(&branch, "key"), Some(2));

        trunk.delete(&branch, "key").unwrap();
        assert_eq!(trunk.current_version(&branch, "key"), None);
    }

    #[test]
    fn advertises_versions_capability() {
        let trunk = RdbmsTrunk::new();
        let caps = CapabilityAdvertiser::capabilities(&trunk);
        assert!(caps.contains(&TrunkCapability::Versions));
    }
}
