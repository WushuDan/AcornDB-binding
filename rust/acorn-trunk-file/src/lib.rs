#![allow(dead_code)]

use acorn_core::{AcornError, AcornResult};

#[derive(Debug, Default)]
pub struct FileTrunk;

impl FileTrunk {
    pub fn new() -> Self {
        FileTrunk
    }

    pub fn init_filesystem(&self) -> AcornResult<()> {
        Err(AcornError::NotImplemented)
    }
}
