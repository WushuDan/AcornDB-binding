#![allow(dead_code)]

use acorn_core::{AcornError, AcornResult};

#[derive(Debug, Default)]
pub struct RdbmsTrunk;

impl RdbmsTrunk {
    pub fn new() -> Self {
        RdbmsTrunk
    }

    pub fn connect(&self) -> AcornResult<()> {
        Err(AcornError::NotImplemented)
    }
}
