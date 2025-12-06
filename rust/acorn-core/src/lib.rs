#![allow(dead_code)]

use std::fmt::Debug;

pub type AcornResult<T> = Result<T, AcornError>;

#[derive(Debug, thiserror::Error)]
pub enum AcornError {
    #[error("not implemented yet")]
    NotImplemented,
    #[error("trunk operation failed: {0}")]
    Trunk(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BranchId(String);

impl BranchId {
    pub fn new<T: Into<String>>(value: T) -> Self {
        BranchId(value.into())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct GroveId(String);

impl GroveId {
    pub fn new<T: Into<String>>(value: T) -> Self {
        GroveId(value.into())
    }
}

/// Represents a stored value plus metadata.
#[derive(Debug, Clone)]
pub struct Nut<T> {
    pub value: T,
}

/// Core storage abstraction shared by trunks.
pub trait Trunk<T>: Send + Sync + Debug {
    fn get(&self, branch: &BranchId, key: &str) -> AcornResult<Option<Nut<T>>>;
    fn put(&self, branch: &BranchId, key: &str, nut: Nut<T>) -> AcornResult<()>;
    fn delete(&self, branch: &BranchId, key: &str) -> AcornResult<()>;
}

/// Tree provides typed access to a trunk.
#[derive(Debug)]
pub struct Tree<T, S: Trunk<T>> {
    branch: BranchId,
    trunk: S,
    _marker: std::marker::PhantomData<T>,
}

impl<T, S> Tree<T, S>
where
    T: Clone + Send + Sync + 'static,
    S: Trunk<T> + Clone,
{
    pub fn new(branch: BranchId, trunk: S) -> Self {
        Tree {
            branch,
            trunk,
            _marker: std::marker::PhantomData,
        }
    }

    pub fn get(&self, key: &str) -> AcornResult<Option<Nut<T>>> {
        self.trunk.get(&self.branch, key)
    }

    pub fn put(&self, key: &str, nut: Nut<T>) -> AcornResult<()> {
        self.trunk.put(&self.branch, key, nut)
    }

    pub fn delete(&self, key: &str) -> AcornResult<()> {
        self.trunk.delete(&self.branch, key)
    }
}
