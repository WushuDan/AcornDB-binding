#![allow(dead_code)]

use std::fs;
use std::path::PathBuf;
use std::time::SystemTime;

use acorn_core::{
    AcornError, AcornResult, BranchId, CapabilityAdvertiser, HistoryEvent, HistoryProvider, Nut, Trunk,
    TrunkCapability, Ttl, TtlProvider,
};

#[derive(Debug, Clone)]
pub struct FileTrunk {
    root: PathBuf,
    ttl_enabled: bool,
    history_enabled: bool,
}

impl FileTrunk {
    pub fn new<P: Into<PathBuf>>(root: P) -> Self {
        FileTrunk {
            root: root.into(),
            ttl_enabled: false,
            history_enabled: false,
        }
    }

    pub fn with_ttl<P: Into<PathBuf>>(root: P) -> Self {
        FileTrunk {
            root: root.into(),
            ttl_enabled: true,
            history_enabled: false,
        }
    }

    pub fn with_history<P: Into<PathBuf>>(root: P) -> Self {
        FileTrunk {
            root: root.into(),
            ttl_enabled: false,
            history_enabled: true,
        }
    }

    pub fn with_history_and_ttl<P: Into<PathBuf>>(root: P) -> Self {
        FileTrunk {
            root: root.into(),
            ttl_enabled: true,
            history_enabled: true,
        }
    }

    pub fn init_filesystem(&self) -> AcornResult<()> {
        fs::create_dir_all(&self.root).map_err(|e| AcornError::Trunk(e.to_string()))
    }

    fn branch_dir(&self, branch: &BranchId) -> PathBuf {
        self.root.join(branch.to_string())
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
        fs::remove_file(&path).map_err(|e| AcornError::Trunk(e.to_string()))?;
        if self.history_enabled {
            self.append_history(branch, HistoryEvent::Delete { key: key.to_string() })?;
        }
        Ok(())
    }
}

impl CapabilityAdvertiser for FileTrunk {
    fn capabilities(&self) -> &'static [TrunkCapability] {
        match (self.ttl_enabled, self.history_enabled) {
            (true, true) => &[TrunkCapability::Ttl, TrunkCapability::History],
            (true, false) => &[TrunkCapability::Ttl],
            (false, true) => &[TrunkCapability::History],
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
