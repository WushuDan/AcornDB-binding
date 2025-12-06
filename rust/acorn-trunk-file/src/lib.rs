#![allow(dead_code)]

use std::fs;
use std::path::PathBuf;

use acorn_core::{AcornError, AcornResult, BranchId, Nut, Trunk};

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

impl ToString for BranchId {
    fn to_string(&self) -> String {
        self.0.clone()
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
