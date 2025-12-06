#![allow(dead_code)]

use acorn_core::{AcornError, AcornResult};

#[derive(Debug, Default)]
pub struct GitTrunk;

impl GitTrunk {
    pub fn new() -> Self {
        GitTrunk
    }

    pub fn connect(&self) -> AcornResult<()> {
        Err(AcornError::NotImplemented)
    }
}
