#![allow(dead_code)]

use std::fs;
use std::path::PathBuf;

use acorn_core::{
    AcornError, AcornResult, BranchId, CapabilityAdvertiser, Nut, Trunk, TrunkCapability,
};

#[derive(Debug, Clone)]
pub struct FileTrunk {
    root: PathBuf,
}

impl FileTrunk {
    pub fn new<P: Into<PathBuf>>(root: P) -> Self {
        FileTrunk { root: root.into() }
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
        fs::remove_file(path).map_err(|e| AcornError::Trunk(e.to_string()))
    }
}

impl CapabilityAdvertiser for FileTrunk {
    fn capabilities(&self) -> &'static [TrunkCapability] {
        &[]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn put_get_delete_round_trip() {
        let tmp_dir = tempfile::tempdir().unwrap();
        let trunk = FileTrunk::new(tmp_dir.path());
        let branch = BranchId::new("main");

        trunk.init_filesystem().unwrap();
        trunk
            .put(&branch, "key", Nut { value: b"hello".to_vec() })
            .unwrap();

        let fetched = trunk.get(&branch, "key").unwrap().unwrap();
        assert_eq!(fetched.value, b"hello".to_vec());

        trunk.delete(&branch, "key").unwrap();
        assert!(!fs::metadata(tmp_dir.path().join("main").join("key")).is_ok());
    }
}
