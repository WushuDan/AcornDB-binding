#![allow(dead_code)]

use std::fmt::{self, Debug};
use std::marker::PhantomData;
use std::time::SystemTime;

use serde::{de::DeserializeOwned, Serialize};

pub type AcornResult<T> = Result<T, AcornError>;

#[derive(Debug, thiserror::Error)]
pub enum AcornError {
    #[error("not implemented yet")]
    NotImplemented,
    #[error("trunk operation failed: {0}")]
    Trunk(String),
    #[error("serialization failed: {0}")]
    Serialization(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BranchId(String);

impl BranchId {
    pub fn new<T: Into<String>>(value: T) -> Self {
        BranchId(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for BranchId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
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

#[derive(Debug, Clone, Copy)]
pub struct Ttl {
    pub expires_at: SystemTime,
}

/// Core storage abstraction shared by trunks.
pub trait Trunk<T>: Send + Sync + Debug {
    fn get(&self, branch: &BranchId, key: &str) -> AcornResult<Option<Nut<T>>>;
    fn put(&self, branch: &BranchId, key: &str, nut: Nut<T>) -> AcornResult<()>;
    fn delete(&self, branch: &BranchId, key: &str) -> AcornResult<()>;
    fn capabilities(&self) -> &'static [TrunkCapability] {
        &[]
    }
}

/// Capability flags for trunks; extend as behaviors are implemented.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrunkCapability {
    History,
    Transactions,
    Ttl,
    Streaming,
}

pub trait CapabilityAdvertiser {
    fn capabilities(&self) -> &'static [TrunkCapability];
}

#[derive(Debug, Clone)]
pub enum HistoryEvent<T> {
    Put { key: String, nut: Nut<T> },
    Delete { key: String },
}

pub trait HistoryProvider<T>: Trunk<T> {
    fn history(&self, branch: &BranchId) -> AcornResult<Vec<HistoryEvent<T>>>;
}

pub trait TtlProvider<T>: Trunk<T> {
    fn put_with_ttl(&self, branch: &BranchId, key: &str, nut: Nut<T>, ttl: Ttl) -> AcornResult<()>;
}

/// Serialization hooks to ensure deterministic cross-language payloads.
pub trait NutCodec<T>: Send + Sync {
    fn encode(&self, value: &T) -> AcornResult<Vec<u8>>;
    fn decode(&self, bytes: &[u8]) -> AcornResult<T>;
}

#[derive(Debug, Clone)]
pub struct JsonCodec;

impl<T> NutCodec<T> for JsonCodec
where
    T: Serialize + DeserializeOwned,
{
    fn encode(&self, value: &T) -> AcornResult<Vec<u8>> {
        serde_json::to_vec(value).map_err(|e| AcornError::Serialization(e.to_string()))
    }

    fn decode(&self, bytes: &[u8]) -> AcornResult<T> {
        serde_json::from_slice(bytes).map_err(|e| AcornError::Serialization(e.to_string()))
    }
}

/// Tree provides typed access to a trunk.
#[derive(Debug, Clone)]
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

    pub fn branch(&self) -> &BranchId {
        &self.branch
    }

    pub fn trunk(&self) -> &S {
        &self.trunk
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

/// Tree wrapper that encodes/decodes typed payloads to byte-oriented trunks.
#[derive(Debug, Clone)]
pub struct EncodedTree<T, S, C>
where
    S: Trunk<Vec<u8>>,
    C: NutCodec<T>,
{
    tree: Tree<Vec<u8>, S>,
    codec: C,
    _marker: PhantomData<T>,
}

impl<T, S, C> EncodedTree<T, S, C>
where
    T: Clone,
    S: Trunk<Vec<u8>> + Clone,
    C: NutCodec<T> + Clone,
{
    pub fn new(branch: BranchId, trunk: S, codec: C) -> Self {
        EncodedTree {
            tree: Tree::new(branch, trunk),
            codec,
            _marker: PhantomData,
        }
    }

    pub fn get(&self, key: &str) -> AcornResult<Option<Nut<T>>> {
        let codec = &self.codec;
        self.tree
            .get(key)?
            .map(|nut| {
                let value = codec.decode(&nut.value)?;
                Ok(Nut { value })
            })
            .transpose()
    }

    pub fn put(&self, key: &str, nut: Nut<T>) -> AcornResult<()> {
        let bytes = self.codec.encode(&nut.value)?;
        self.tree.put(key, Nut { value: bytes })
    }

    pub fn delete(&self, key: &str) -> AcornResult<()> {
        self.tree.delete(key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    struct Demo {
        value: String,
    }

    #[test]
    fn json_codec_round_trip() {
        let codec = JsonCodec;
        let original = Demo {
            value: "hello".into(),
        };

        let bytes = codec.encode(&original).unwrap();
        let decoded: Demo = codec.decode(&bytes).unwrap();
        assert_eq!(decoded, original);
    }
}
