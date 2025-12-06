#![allow(dead_code)]

use acorn_core::{AcornError, AcornResult};

#[derive(Debug, Default)]
pub struct S3Trunk;

impl S3Trunk {
    pub fn new() -> Self {
        S3Trunk
    }

    pub fn connect(&self) -> AcornResult<()> {
        Err(AcornError::NotImplemented)
    }
}
