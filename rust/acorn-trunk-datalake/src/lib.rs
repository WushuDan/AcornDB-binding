#![allow(dead_code)]

use acorn_core::{AcornError, AcornResult};

#[derive(Debug, Default)]
pub struct DataLakeTrunk;

impl DataLakeTrunk {
    pub fn new() -> Self {
        DataLakeTrunk
    }

    pub fn connect(&self) -> AcornResult<()> {
        Err(AcornError::NotImplemented)
    }
}
