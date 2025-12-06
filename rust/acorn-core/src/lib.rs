#![allow(dead_code)]

use std::marker::PhantomData;

pub type AcornResult<T> = Result<T, AcornError>;

#[derive(Debug, thiserror::Error)]
pub enum AcornError {
    #[error("not implemented yet")]
    NotImplemented,
}

#[derive(Debug, Clone)]
pub struct Nut<T> {
    _marker: PhantomData<T>,
}

#[derive(Debug, Clone)]
pub struct Tree<T> {
    _marker: PhantomData<T>,
}

impl<T> Tree<T> {
    pub fn not_implemented(&self) -> AcornResult<()> {
        Err(AcornError::NotImplemented)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BranchId(String);

impl BranchId {
    pub fn new<T: Into<String>>(value: T) -> Self {
        BranchId(value.into())
    }
}
