#![allow(dead_code)]

use acorn_core::AcornResult;

#[derive(Debug, Default)]
pub struct MemoryTrunk;

impl MemoryTrunk {
    pub fn new() -> Self {
        MemoryTrunk
    }

    pub fn health_check(&self) -> AcornResult<()> {
        Ok(())
    }
}
