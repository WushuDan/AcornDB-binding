#![allow(dead_code)]

use std::collections::HashMap;
use std::sync::Arc;
use std::time::SystemTime;

use acorn_core::{
    AcornError, AcornResult, BranchId, CapabilityAdvertiser, HistoryEvent, HistoryProvider, Nut, Trunk,
    TrunkCapability, Ttl, TtlProvider,
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
    ttl: HashMap<(BranchId, String), SystemTime>,
    versions: HashMap<(BranchId, String), u64>,
}

impl MemoryTrunk {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn health_check(&self) -> AcornResult<()> {
        Ok(())
    }

    pub fn keys(&self, branch: &BranchId) -> Vec<String> {
        let guard = self.inner.read();
        guard
            .data
            .keys()
            .filter(|(b, _)| b == branch)
            .map(|(_, k)| k.clone())
            .collect()
    }

    pub fn current_version(&self, branch: &BranchId, key: &str) -> Option<u64> {
        let guard = self.inner.read();
        guard.versions.get(&(branch.clone(), key.to_string())).copied()
    }
}

impl Trunk<Vec<u8>> for MemoryTrunk {
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

impl CapabilityAdvertiser for MemoryTrunk {
    fn capabilities(&self) -> &'static [TrunkCapability] {
        &[
            TrunkCapability::History,
            TrunkCapability::Ttl,
            TrunkCapability::Versions,
        ]
    }
}

impl TtlProvider<Vec<u8>> for MemoryTrunk {
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

impl HistoryProvider<Vec<u8>> for MemoryTrunk {
    fn history(&self, branch: &BranchId) -> AcornResult<Vec<HistoryEvent<Vec<u8>>>> {
        let guard = self.inner.read();
        Ok(guard.history.get(branch).cloned().unwrap_or_else(Vec::new))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use acorn_core::{CapabilityAdvertiser, EncodedTree, JsonCodec, Tree};
    #[cfg(feature = "contract-tests")]
    use acorn_test_harness::TrunkContract;
    #[cfg(feature = "contract-tests")]
    use acorn_test_harness::TrunkContract;
    use serde::{Deserialize, Serialize};

    #[test]
    fn put_get_delete_round_trip() {
        let trunk = MemoryTrunk::new();
        let branch = BranchId::new("main");

        trunk
            .put(
                &branch,
                "key",
                Nut {
                    value: b"hello".to_vec(),
                },
            )
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
            .put(
                &branch,
                "key",
                Nut {
                    value: b"bytes".to_vec(),
                },
            )
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

    #[test]
    fn expires_entries_with_ttl() {
        let trunk = MemoryTrunk::new();
        let branch = BranchId::new("ttl");
        let ttl = Ttl {
            expires_at: SystemTime::now() + std::time::Duration::from_millis(10),
        };

        trunk
            .put_with_ttl(
                &branch,
                "key",
                Nut {
                    value: b"live".to_vec(),
                },
                ttl,
            )
            .unwrap();
        assert!(trunk.get(&branch, "key").unwrap().is_some());

        std::thread::sleep(std::time::Duration::from_millis(20));
        assert!(trunk.get(&branch, "key").unwrap().is_none());
    }

    #[test]
    fn reports_capabilities() {
        let trunk = MemoryTrunk::new();
        let caps = CapabilityAdvertiser::capabilities(&trunk);
        assert!(caps.contains(&TrunkCapability::History));
        assert!(caps.contains(&TrunkCapability::Ttl));
        assert!(caps.contains(&TrunkCapability::Versions));
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    struct Demo {
        msg: String,
    }

    #[test]
    fn encoded_tree_round_trip() {
        let trunk = MemoryTrunk::new();
        let tree = EncodedTree::new(BranchId::new("enc"), trunk, JsonCodec);

        let value = Demo { msg: "hi".into() };
        tree.put("key", Nut { value: value.clone() }).unwrap();

        let fetched = tree.get("key").unwrap().unwrap();
        assert_eq!(fetched.value, value);
    }

    #[test]
    fn versions_increment_on_put() {
        let trunk = MemoryTrunk::new();
        let branch = BranchId::new("v");
        assert_eq!(trunk.current_version(&branch, "k"), None);

        trunk.put(&branch, "k", Nut { value: b"1".to_vec() }).unwrap();
        assert_eq!(trunk.current_version(&branch, "k"), Some(1));

        trunk.put(&branch, "k", Nut { value: b"2".to_vec() }).unwrap();
        assert_eq!(trunk.current_version(&branch, "k"), Some(2));
    }

    #[test]
    fn tree_put_if_version_detects_conflict() {
        let trunk = MemoryTrunk::new();
        let tree = Tree::new(BranchId::new("branch"), trunk.clone());

        tree.put("key", Nut { value: b"one".to_vec() }).unwrap();
        assert_eq!(trunk.current_version(&BranchId::new("branch"), "key"), Some(1));

        let ok = tree
            .put_if_version("key", Some(1), Nut { value: b"two".to_vec() });
        assert!(ok.is_ok());

        let conflict = tree.put_if_version("key", Some(1), Nut { value: b"three".to_vec() });
        assert!(matches!(
            conflict,
            Err(AcornError::VersionConflict {
                expected: Some(1),
                actual: Some(2)
            })
        ));
    }

    #[test]
    fn tree_delete_if_version_confirms_missing_or_conflict() {
        let trunk = MemoryTrunk::new();
        let tree = Tree::new(BranchId::new("branch-del"), trunk.clone());

        // deleting missing key yields missing error when expectation is None
        let res = tree.delete_if_version("nope", None);
        assert!(matches!(res, Err(AcornError::MissingKey(_))));

        tree.put("key", Nut { value: b"one".to_vec() }).unwrap();
        assert!(tree.delete_if_version("key", Some(1)).is_ok());

        // Once deleted, another delete with expected version conflicts
        let res = tree.delete_if_version("key", Some(1));
        assert!(matches!(res, Err(AcornError::VersionConflict { .. })));
    }

    #[cfg(feature = "contract-tests")]
    #[test]
    fn contract_round_trip_and_ttl() {
        let trunk = MemoryTrunk::new();
        TrunkContract::round_trip_bytes(&trunk).unwrap();
        TrunkContract::assert_capabilities(&trunk, &[TrunkCapability::History, TrunkCapability::Ttl]);
        TrunkContract::ttl_expiry(&trunk).unwrap();
        TrunkContract::history_put_delete(&trunk).unwrap();
    }
}
