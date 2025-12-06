#![allow(dead_code)]

use acorn_core::{AcornError, AcornResult};

#[derive(Debug, Default)]
pub struct AzureTrunk;

impl AzureTrunk {
    pub fn new() -> Self {
        AzureTrunk
    }

    pub fn connect(&self) -> AcornResult<()> {
        Err(AcornError::NotImplemented)
    }
}
