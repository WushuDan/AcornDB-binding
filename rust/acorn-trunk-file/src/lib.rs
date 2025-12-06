#![allow(dead_code)]

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::SystemTime;

use acorn_core::{
    AcornError, AcornResult, BranchId, CapabilityAdvertiser, HistoryEvent, HistoryProvider, KeyedTrunk, Nut,
    TombstoneProvider, Trunk, TrunkCapability, Ttl, TtlCleaner, TtlProvider,
};
use parking_lot::RwLock;

#[derive(Debug, Clone)]
pub struct FileTrunk {
    root: PathBuf,
    ttl_enabled: bool,
    history_enabled: bool,
    versions_enabled: bool,
    tombstones: Arc<RwLock<HashMap<(BranchId, String), Option<u64>>>>,
}

impl FileTrunk {
    pub fn new<P: Into<PathBuf>>(root: P) -> Self {
        FileTrunk {
            root: root.into(),
            ttl_enabled: false,
            history_enabled: false,
            versions_enabled: false,
            tombstones: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn with_ttl<P: Into<PathBuf>>(root: P) -> Self {
        FileTrunk {
            root: root.into(),
            ttl_enabled: true,
            history_enabled: false,
            versions_enabled: false,
            tombstones: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn with_history<P: Into<PathBuf>>(root: P) -> Self {
        FileTrunk {
            root: root.into(),
            ttl_enabled: false,
            history_enabled: true,
            versions_enabled: true,
            tombstones: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn with_history_and_ttl<P: Into<PathBuf>>(root: P) -> Self {
        FileTrunk {
            root: root.into(),
            ttl_enabled: true,
            history_enabled: true,
            versions_enabled: true,
            tombstones: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn init_filesystem(&self) -> AcornResult<()> {
        fs::create_dir_all(&self.root).map_err(|e| AcornError::Trunk(e.to_string()))
    }

    fn branch_dir(&self, branch: &BranchId) -> PathBuf {
        self.root.join(branch.to_string())
    }

    fn versions_dir(&self, branch: &BranchId) -> PathBuf {
        self.branch_dir(branch).join(".versions")
    }

    fn version_path(&self, branch: &BranchId, key: &str) -> PathBuf {
        self.versions_dir(branch).join(key)
    }

    fn bump_version(&self, branch: &BranchId, key: &str) -> AcornResult<u64> {
        if !self.versions_enabled {
            return Ok(0);
        }

        fs::create_dir_all(self.versions_dir(branch)).map_err(|e| AcornError::Trunk(e.to_string()))?;
        let path = self.version_path(branch, key);
        let next = fs::read_to_string(&path)
            .ok()
            .and_then(|raw| raw.parse::<u64>().ok())
            .unwrap_or(0)
            .saturating_add(1);
        fs::write(&path, next.to_string()).map_err(|e| AcornError::Trunk(e.to_string()))?;
        Ok(next)
    }

    fn clear_version(&self, branch: &BranchId, key: &str) {
        if self.versions_enabled {
            let _ = fs::remove_file(self.version_path(branch, key));
        }
    }

    fn record_tombstone(&self, branch: &BranchId, key: &str) {
        let version = if self.versions_enabled {
            self.current_version(branch, key)
        } else {
            None
        };
        self.tombstones
            .write()
            .insert((branch.clone(), key.to_string()), version);
    }

    pub fn current_version(&self, branch: &BranchId, key: &str) -> Option<u64> {
        if !self.versions_enabled {
            return None;
        }
        fs::read_to_string(self.version_path(branch, key))
            .ok()
            .and_then(|raw| raw.parse::<u64>().ok())
    }
}

impl Trunk<Vec<u8>> for FileTrunk {
    fn get(&self, branch: &BranchId, key: &str) -> AcornResult<Option<Nut<Vec<u8>>>> {
        let path = self.branch_dir(branch).join(key);

        if self.ttl_enabled {
            let ttl_path = self.branch_dir(branch).join(format!("{}.ttl", key));
            if let Ok(raw) = fs::read_to_string(&ttl_path) {
                if let Ok(ms) = raw.parse::<u128>() {
                    let expires_at = SystemTime::UNIX_EPOCH + std::time::Duration::from_millis(ms as u64);
                    if SystemTime::now() >= expires_at {
                        let _ = fs::remove_file(&path);
                        let _ = fs::remove_file(&ttl_path);
                        self.clear_version(branch, key);
                        return Ok(None);
                    }
                }
            }
        }

        match fs::read(&path) {
            Ok(bytes) => Ok(Some(Nut { value: bytes })),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(AcornError::Trunk(e.to_string())),
        }
    }

    fn put(&self, branch: &BranchId, key: &str, nut: Nut<Vec<u8>>) -> AcornResult<()> {
        let dir = self.branch_dir(branch);
        fs::create_dir_all(&dir).map_err(|e| AcornError::Trunk(e.to_string()))?;
        let path = dir.join(key);
        fs::write(&path, nut.value.clone()).map_err(|e| AcornError::Trunk(e.to_string()))?;
        let _ = self.bump_version(branch, key)?;
        self.tombstones.write().remove(&(branch.clone(), key.to_string()));
        if self.history_enabled {
            self.append_history(
                branch,
                HistoryEvent::Put {
                    key: key.to_string(),
                    nut,
                },
            )?;
        }
        Ok(())
    }

    fn delete(&self, branch: &BranchId, key: &str) -> AcornResult<()> {
        let path = self.branch_dir(branch).join(key);
        let ttl_path = self.branch_dir(branch).join(format!("{}.ttl", key));
        let _ = fs::remove_file(&ttl_path);
        fs::remove_file(&path).map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                AcornError::MissingKey(key.to_string())
            } else {
                AcornError::Trunk(e.to_string())
            }
        })?;
        let version = self.current_version(branch, key);
        self.clear_version(branch, key);
        self.tombstones
            .write()
            .insert((branch.clone(), key.to_string()), version);
        if self.history_enabled {
            self.append_history(branch, HistoryEvent::Delete { key: key.to_string() })?;
        }
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
        if let Some(expected) = expected {
            let current = self.current_version(branch, key);
            if current != Some(expected) {
                return Err(AcornError::VersionConflict {
                    expected: Some(expected),
                    actual: current,
                });
            }
        }
        self.put(branch, key, nut)
    }

    fn delete_if_version(&self, branch: &BranchId, key: &str, expected: Option<u64>) -> AcornResult<()> {
        let current = self.current_version(branch, key);
        if let Some(expected) = expected {
            if current != Some(expected) {
                return Err(AcornError::VersionConflict {
                    expected: Some(expected),
                    actual: current,
                });
            }
        }
        self.delete(branch, key)
    }
}

impl KeyedTrunk<Vec<u8>> for FileTrunk {
    fn keys(&self, branch: &BranchId) -> Vec<String> {
        FileTrunk::keys(self, branch)
    }
}

impl TtlCleaner<Vec<u8>> for FileTrunk {
    fn purge_expired(&self, branch: &BranchId) -> usize {
        let mut removed = 0usize;
        let dir = self.branch_dir(branch);
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    if name.ends_with(".ttl") {
                        if let Ok(raw) = fs::read_to_string(&path) {
                            if let Ok(ms) = raw.parse::<u128>() {
                                let expires_at =
                                    SystemTime::UNIX_EPOCH + std::time::Duration::from_millis(ms as u64);
                                if SystemTime::now() >= expires_at {
                                    let key = name.trim_end_matches(".ttl");
                                    let data_path = self.branch_dir(branch).join(key);
                                    let version = self.current_version(branch, key);
                                    let _ = fs::remove_file(&path);
                                    let _ = fs::remove_file(&data_path);
                                    self.clear_version(branch, key);
                                    self.tombstones
                                        .write()
                                        .insert((branch.clone(), key.to_string()), version);
                                    if self.history_enabled {
                                        let _ = self.append_history(
                                            branch,
                                            HistoryEvent::Delete { key: key.to_string() },
                                        );
                                    }
                                    removed += 1;
                                }
                            }
                        }
                    }
                }
            }
        }
        removed
    }
}

impl TombstoneProvider<Vec<u8>> for FileTrunk {
    fn tombstones(&self, branch: &BranchId) -> Vec<(String, Option<u64>)> {
        self.tombstones
            .read()
            .iter()
            .filter(|((b, _), _)| b == branch)
            .map(|((_, k), v)| (k.clone(), *v))
            .collect()
    }
}

impl CapabilityAdvertiser for FileTrunk {
    fn capabilities(&self) -> &'static [TrunkCapability] {
        match (self.ttl_enabled, self.history_enabled) {
            (true, true) if self.versions_enabled => &[
                TrunkCapability::Ttl,
                TrunkCapability::History,
                TrunkCapability::Versions,
            ],
            (true, true) => &[TrunkCapability::Ttl, TrunkCapability::History],
            (true, false) if self.versions_enabled => &[TrunkCapability::Ttl, TrunkCapability::Versions],
            (true, false) => &[TrunkCapability::Ttl],
            (false, true) if self.versions_enabled => &[TrunkCapability::History, TrunkCapability::Versions],
            (false, true) => &[TrunkCapability::History],
            (false, false) if self.versions_enabled => &[TrunkCapability::Versions],
            (false, false) => &[],
        }
    }
}

impl TtlProvider<Vec<u8>> for FileTrunk {
    fn put_with_ttl(&self, branch: &BranchId, key: &str, nut: Nut<Vec<u8>>, ttl: Ttl) -> AcornResult<()> {
        if !self.ttl_enabled {
            return Err(AcornError::Trunk("TTL not enabled for this trunk".into()));
        }

        let dir = self.branch_dir(branch);
        fs::create_dir_all(&dir).map_err(|e| AcornError::Trunk(e.to_string()))?;
        let path = dir.join(key);

        fs::write(&path, nut.value.clone()).map_err(|e| AcornError::Trunk(e.to_string()))?;
        let expires_at = ttl
            .expires_at
            .duration_since(SystemTime::UNIX_EPOCH)
            .map_err(|e| AcornError::Trunk(e.to_string()))?;
        fs::write(
            dir.join(format!("{}.ttl", key)),
            expires_at.as_millis().to_string(),
        )
        .map_err(|e| AcornError::Trunk(e.to_string()))?;

        let _ = self.bump_version(branch, key)?;
        self.tombstones.write().remove(&(branch.clone(), key.to_string()));
        if self.history_enabled {
            self.append_history(
                branch,
                HistoryEvent::Put {
                    key: key.to_string(),
                    nut,
                },
            )?;
        }
        Ok(())
    }
}

impl FileTrunk {
    fn history_dir(&self, branch: &BranchId) -> PathBuf {
        self.branch_dir(branch).join(".history")
    }

    fn append_history(&self, branch: &BranchId, event: HistoryEvent<Vec<u8>>) -> AcornResult<()> {
        let dir = self.history_dir(branch);
        fs::create_dir_all(&dir).map_err(|e| AcornError::Trunk(e.to_string()))?;
        let path = dir.join("events.log");
        let mut file = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
            .map_err(|e| AcornError::Trunk(e.to_string()))?;
        let line = serde_json::to_string(&event).map_err(|e| AcornError::Trunk(e.to_string()))?;
        use std::io::Write;
        writeln!(file, "{}", line).map_err(|e| AcornError::Trunk(e.to_string()))
    }

    pub fn keys(&self, branch: &BranchId) -> Vec<String> {
        let dir = self.branch_dir(branch);
        match fs::read_dir(dir) {
            Ok(entries) => entries
                .flatten()
                .filter_map(|entry| {
                    let path = entry.path();
                    if path.is_file() {
                        let name = path.file_name()?.to_string_lossy().to_string();
                        if name.ends_with(".ttl") {
                            None
                        } else {
                            Some(name)
                        }
                    } else {
                        None
                    }
                })
                .collect(),
            Err(_) => Vec::new(),
        }
    }
}

impl HistoryProvider<Vec<u8>> for FileTrunk {
    fn history(&self, branch: &BranchId) -> AcornResult<Vec<HistoryEvent<Vec<u8>>>> {
        if !self.history_enabled {
            return Ok(Vec::new());
        }

        let path = self.history_dir(branch).join("events.log");
        if !path.exists() {
            return Ok(Vec::new());
        }

        let data = fs::read_to_string(&path).map_err(|e| AcornError::Trunk(e.to_string()))?;
        let mut events = Vec::new();
        for line in data.lines() {
            if line.trim().is_empty() {
                continue;
            }
            let event: HistoryEvent<Vec<u8>> =
                serde_json::from_str(line).map_err(|e| AcornError::Trunk(e.to_string()))?;
            events.push(event);
        }
        Ok(events)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use acorn_core::CapabilityAdvertiser;
    #[cfg(feature = "contract-tests")]
    use acorn_test_harness::TrunkContract;
    use std::fs;
    #[cfg(feature = "contract-tests")]
    use std::io::Read;

    #[test]
    fn put_get_delete_round_trip() {
        let tmp_dir = tempfile::tempdir().unwrap();
        let trunk = FileTrunk::new(tmp_dir.path());
        let branch = BranchId::new("main");

        trunk.init_filesystem().unwrap();
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
        assert!(!fs::metadata(tmp_dir.path().join("main").join("key")).is_ok());
    }

    #[test]
    fn respects_ttl_when_enabled() {
        let tmp_dir = tempfile::tempdir().unwrap();
        let trunk = FileTrunk::with_ttl(tmp_dir.path());
        let branch = BranchId::new("main");
        trunk.init_filesystem().unwrap();

        let ttl = Ttl {
            expires_at: SystemTime::now() + std::time::Duration::from_millis(10),
        };

        trunk
            .put_with_ttl(
                &branch,
                "key",
                Nut {
                    value: b"hello".to_vec(),
                },
                ttl,
            )
            .unwrap();
        assert!(trunk.get(&branch, "key").unwrap().is_some());

        std::thread::sleep(std::time::Duration::from_millis(20));
        assert!(trunk.get(&branch, "key").unwrap().is_none());
    }

    #[cfg(feature = "contract-tests")]
    #[test]
    fn contract_round_trip_and_capabilities() {
        let tmp_dir = tempfile::tempdir().unwrap();
        let trunk = FileTrunk::new(tmp_dir.path());
        TrunkContract::round_trip_bytes(&trunk).unwrap();
        TrunkContract::assert_capabilities(&trunk, &[]);
    }

    #[cfg(feature = "contract-tests")]
    #[test]
    fn contract_ttl_expiry() {
        let tmp_dir = tempfile::tempdir().unwrap();
        let trunk = FileTrunk::with_ttl(tmp_dir.path());
        TrunkContract::ttl_expiry(&trunk).unwrap();
    }

    #[test]
    fn advertises_versions_capability_when_enabled() {
        let tmp_dir = tempfile::tempdir().unwrap();
        let trunk = FileTrunk::with_history(tmp_dir.path());
        let caps = CapabilityAdvertiser::capabilities(&trunk);
        assert!(caps.contains(&TrunkCapability::History));
        assert!(caps.contains(&TrunkCapability::Versions));
    }

    #[test]
    fn tracks_versions_when_enabled() {
        let tmp_dir = tempfile::tempdir().unwrap();
        let trunk = FileTrunk::with_history(tmp_dir.path());
        let branch = BranchId::new("versions");

        trunk
            .put(
                &branch,
                "key",
                Nut {
                    value: b"one".to_vec(),
                },
            )
            .unwrap();
        assert_eq!(trunk.current_version(&branch, "key"), Some(1));

        trunk
            .put(
                &branch,
                "key",
                Nut {
                    value: b"two".to_vec(),
                },
            )
            .unwrap();
        assert_eq!(trunk.current_version(&branch, "key"), Some(2));

        trunk.delete(&branch, "key").unwrap();
        assert_eq!(trunk.current_version(&branch, "key"), None);
    }

    #[test]
    fn cas_put_and_conflict() {
        let tmp_dir = tempfile::tempdir().unwrap();
        let trunk = FileTrunk::with_history(tmp_dir.path());
        let branch = BranchId::new("cas");

        // First write without expectation
        trunk
            .put_if_version(
                &branch,
                "key",
                None,
                Nut {
                    value: b"v1".to_vec(),
                },
            )
            .unwrap();
        assert_eq!(trunk.current_version(&branch, "key"), Some(1));

        // Next write with matching expectation succeeds
        trunk
            .put_if_version(
                &branch,
                "key",
                Some(1),
                Nut {
                    value: b"v2".to_vec(),
                },
            )
            .unwrap();
        assert_eq!(trunk.current_version(&branch, "key"), Some(2));

        // Conflicting expectation rejected
        let conflict = trunk.put_if_version(
            &branch,
            "key",
            Some(1),
            Nut {
                value: b"v3".to_vec(),
            },
        );
        assert!(matches!(
            conflict,
            Err(AcornError::VersionConflict {
                expected: Some(1),
                actual: Some(2)
            })
        ));
    }

    #[test]
    fn cas_delete_and_conflict() {
        let tmp_dir = tempfile::tempdir().unwrap();
        let trunk = FileTrunk::with_history(tmp_dir.path());
        let branch = BranchId::new("cas-del");

        trunk
            .put_if_version(
                &branch,
                "key",
                None,
                Nut {
                    value: b"v1".to_vec(),
                },
            )
            .unwrap();
        assert_eq!(trunk.current_version(&branch, "key"), Some(1));

        // delete with correct expected version succeeds
        assert!(trunk.delete_if_version(&branch, "key", Some(1)).is_ok());
        assert_eq!(trunk.current_version(&branch, "key"), None);

        // deleting again should report conflict since version no longer matches
        let res = trunk.delete_if_version(&branch, "key", Some(1));
        assert!(matches!(
            res,
            Err(AcornError::VersionConflict {
                expected: Some(1),
                actual: None
            })
        ));
    }

    #[cfg(feature = "contract-tests")]
    #[test]
    fn history_put_delete_logged() {
        let tmp_dir = tempfile::tempdir().unwrap();
        let trunk = FileTrunk::with_history(tmp_dir.path());
        TrunkContract::history_put_delete(&trunk).unwrap();

        // ensure history file exists and has content
        let hist_path = tmp_dir.path().join("history-contract/.history/events.log");
        let mut contents = String::new();
        let mut file = std::fs::File::open(hist_path).unwrap();
        file.read_to_string(&mut contents).unwrap();
        assert!(contents.contains("history-key"));
    }
}
