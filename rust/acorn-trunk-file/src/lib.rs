#![allow(dead_code)]

use std::fs;
use std::path::PathBuf;
use std::time::SystemTime;

use acorn_core::{
    AcornError, AcornResult, BranchId, CapabilityAdvertiser, Nut, Trunk, TrunkCapability, Ttl, TtlProvider,
};

#[derive(Debug, Clone)]
pub struct FileTrunk {
    root: PathBuf,
    ttl_enabled: bool,
}

impl FileTrunk {
    pub fn new<P: Into<PathBuf>>(root: P) -> Self {
        FileTrunk {
            root: root.into(),
            ttl_enabled: false,
        }
    }

    pub fn with_ttl<P: Into<PathBuf>>(root: P) -> Self {
        FileTrunk {
            root: root.into(),
            ttl_enabled: true,
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
        fs::write(path, nut.value).map_err(|e| AcornError::Trunk(e.to_string()))
    }

    fn delete(&self, branch: &BranchId, key: &str) -> AcornResult<()> {
        let path = self.branch_dir(branch).join(key);
        let ttl_path = self.branch_dir(branch).join(format!("{}.ttl", key));
        let _ = fs::remove_file(&ttl_path);
        fs::remove_file(path).map_err(|e| AcornError::Trunk(e.to_string()))
    }
}

impl CapabilityAdvertiser for FileTrunk {
    fn capabilities(&self) -> &'static [TrunkCapability] {
        if self.ttl_enabled {
            &[TrunkCapability::Ttl]
        } else {
            &[]
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

        fs::write(&path, nut.value).map_err(|e| AcornError::Trunk(e.to_string()))?;
        let expires_at = ttl
            .expires_at
            .duration_since(SystemTime::UNIX_EPOCH)
            .map_err(|e| AcornError::Trunk(e.to_string()))?;
        fs::write(
            dir.join(format!("{}.ttl", key)),
            expires_at.as_millis().to_string(),
        )
        .map_err(|e| AcornError::Trunk(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use acorn_test_harness::TrunkContract;
    use std::fs;

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

    #[test]
    fn contract_round_trip_and_capabilities() {
        let tmp_dir = tempfile::tempdir().unwrap();
        let trunk = FileTrunk::new(tmp_dir.path());
        TrunkContract::round_trip_bytes(&trunk).unwrap();
        TrunkContract::assert_capabilities(&trunk, &[]);
    }

    #[test]
    fn contract_ttl_expiry() {
        let tmp_dir = tempfile::tempdir().unwrap();
        let trunk = FileTrunk::with_ttl(tmp_dir.path());
        TrunkContract::ttl_expiry(&trunk).unwrap();
    }
}
