#![allow(dead_code)]

use acorn_core::{AcornError, AcornResult, BranchId, KeyedTrunk, Tree, Trunk};
use serde::{de::DeserializeOwned, Serialize};
#[cfg(feature = "http-client")]
use serde_json;
#[cfg(feature = "http-client")]
use std::collections::{HashMap, HashSet};
use tracing::instrument;

/// HTTP/WebSocket sync endpoint target.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SyncEndpoint {
    pub url: String,
    pub branch: BranchId,
}

#[derive(Debug, Clone)]
pub enum SyncError {
    Network(String),
    Protocol(String),
    Conflict(String),
    Storage(String),
    VersionConflict(String),
    MissingKey(String),
    Unknown(String),
}

/// Transport abstraction for issuing sync requests.
pub trait SyncTransport {
    fn apply(&self, request: &SyncApplyRequest) -> Result<SyncApplyResponse, SyncError>;
    fn pull(&self, branch: &BranchId) -> Result<SyncPullResponse, SyncError>;
}

#[cfg(feature = "http-client")]
#[derive(Clone)]
pub struct HttpTransport {
    client: reqwest::Client,
    base_url: String,
}

#[cfg(feature = "http-client")]
impl HttpTransport {
    pub fn new<T: Into<String>>(base_url: T) -> Self {
        HttpTransport {
            client: reqwest::Client::new(),
            base_url: base_url.into(),
        }
    }

    fn block_on<F, T>(&self, fut: F) -> Result<T, SyncError>
    where
        F: std::future::Future<Output = Result<T, reqwest::Error>>,
    {
        if let Ok(handle) = tokio::runtime::Handle::try_current() {
            handle
                .block_on(fut)
                .map_err(|e| SyncError::Network(e.to_string()))
        } else {
            tokio::runtime::Runtime::new()
                .map_err(|e| SyncError::Network(e.to_string()))?
                .block_on(fut)
                .map_err(|e| SyncError::Network(e.to_string()))
        }
    }
}

#[cfg(feature = "http-client")]
impl SyncTransport for HttpTransport {
    fn apply(&self, request: &SyncApplyRequest) -> Result<SyncApplyResponse, SyncError> {
        let url = format!("{}/sync/apply", self.base_url);
        self.block_on(self.client.post(url).json(request).send())?
            .error_for_status()
            .map_err(|e| SyncError::Protocol(e.to_string()))?
            .json::<SyncApplyResponse>()
            .map_err(|e| SyncError::Protocol(e.to_string()))
    }

    fn pull(&self, branch: &BranchId) -> Result<SyncPullResponse, SyncError> {
        let url = format!("{}/sync/pull", self.base_url);
        self.block_on(self.client.get(url).query(&[("branch", branch.as_str())]).send())?
            .error_for_status()
            .map_err(|e| SyncError::Protocol(e.to_string()))?
            .json::<SyncPullResponse>()
            .map_err(|e| SyncError::Protocol(e.to_string()))
    }
}

/// Stub sync client facade; will orchestrate pull/push/sync with retries.
#[derive(Debug, Default)]
pub struct SyncClient;

impl SyncClient {
    #[instrument(skip(self, tree))]
    pub async fn synchronize<T, S>(
        &self,
        tree: &Tree<T, S>,
        endpoint: &SyncEndpoint,
    ) -> AcornResult<SyncResult>
    where
        T: Clone + Send + Sync + 'static + std::fmt::Debug + Serialize,
        S: Trunk<T> + KeyedTrunk<T> + Clone + Send + Sync,
    {
        #[cfg(feature = "http-client")]
        {
            let transport = HttpTransport::new(endpoint.url.clone());
            let pull = self.pull(tree, endpoint, &transport).await?;
            let push = self.push(tree, endpoint, &transport).await?;
            Ok(SyncResult {
                applied: pull.applied + push.applied,
                conflicts: pull.conflicts + push.conflicts,
                conflict_keys: {
                    let mut keys = pull.conflict_keys;
                    keys.extend(push.conflict_keys);
                    keys
                },
            })
        }
        #[cfg(not(feature = "http-client"))]
        {
            let _ = (tree, endpoint);
            Err(AcornError::NotImplemented)
        }
    }

    #[instrument(skip(self, tree))]
    pub async fn pull<T, S>(&self, tree: &Tree<T, S>, endpoint: &SyncEndpoint) -> AcornResult<SyncResult>
    where
        T: Clone + Send + Sync + 'static + std::fmt::Debug + Serialize + DeserializeOwned,
        S: Trunk<T> + Send + Sync,
    {
        #[cfg(feature = "http-client")]
        {
            let transport = HttpTransport::new(endpoint.url.clone());
            self.pull_with_transport(&transport, &endpoint.branch)
                .and_then(|resp| {
                    for key in resp.deleted {
                        let _ = tree.delete(&key)?;
                    }

                    let mut applied = 0usize;
                    for op in resp.batch.operations {
                        match op {
                            SyncMutation::Put { key, value, .. } => {
                                let decoded: T = serde_json::from_slice(&value)
                                    .map_err(|e| AcornError::Serialization(e.to_string()))?;
                                tree.put(&key, Nut { value: decoded })?;
                                applied += 1;
                            }
                            SyncMutation::Delete { key, .. } => {
                                let _ = tree.delete(&key)?;
                            }
                        }
                    }

                    Ok(SyncResult {
                        applied,
                        conflicts: 0,
                        conflict_keys: Vec::new(),
                    })
                })
        }
        #[cfg(not(feature = "http-client"))]
        {
            let _ = (tree, endpoint);
            Err(AcornError::NotImplemented)
        }
    }

    #[instrument(skip(self, tree))]
    pub async fn push<T, S>(&self, tree: &Tree<T, S>, endpoint: &SyncEndpoint) -> AcornResult<SyncResult>
    where
        T: Clone + Send + Sync + 'static + std::fmt::Debug + Serialize,
        S: Trunk<T> + KeyedTrunk<T> + Clone + Send + Sync,
    {
        #[cfg(feature = "http-client")]
        {
            let transport = HttpTransport::new(endpoint.url.clone());
            // Fetch remote snapshot for delta computation
            let remote = self.pull_with_transport(&transport, &endpoint.branch)?;
            let remote_versions: HashMap<_, _> = remote.versions.into_iter().collect();
            let remote_keys: HashSet<_> = remote
                .batch
                .operations
                .iter()
                .map(|op| match op {
                    SyncMutation::Put { key, .. } => key.clone(),
                    SyncMutation::Delete { key, .. } => key.clone(),
                })
                .collect();
            let remote_deleted: HashSet<_> = remote.deleted.into_iter().collect();
            let remote_deleted_versions: HashMap<_, _> = remote.deleted_versions.into_iter().collect();

            let mut ops = Vec::new();

            // push local puts where version differs or missing remotely
            for key in tree.trunk().keys(&endpoint.branch) {
                let local_version = tree.trunk().version(&endpoint.branch, &key);
                let remote_version = remote_versions.get(&key).copied();
                if remote_version == local_version && !remote_deleted.contains(&key) {
                    continue;
                }
                if let Some(remote_tomb) = remote_deleted_versions.get(&key) {
                    if *remote_tomb == local_version {
                        continue;
                    }
                }
                if remote_deleted.contains(&key) && remote_version.is_none() && local_version.is_none() {
                    continue;
                }
                if let Some(nut) = tree.get(&key)? {
                    let bytes = serde_json::to_vec(&nut.value)
                        .map_err(|e| AcornError::Serialization(e.to_string()))?;
                    ops.push(SyncMutation::Put {
                        key: key.clone(),
                        value: bytes,
                        version: local_version,
                    });
                }
            }

            // send deletes for remote keys missing locally
            let local_keys: HashSet<_> = tree.trunk().keys(&endpoint.branch).into_iter().collect();
            for key in remote_keys {
                if !local_keys.contains(&key) {
                    let version = remote_versions
                        .get(&key)
                        .copied()
                        .or_else(|| remote_deleted_versions.get(&key).copied().flatten());
                    ops.push(SyncMutation::Delete { key, version });
                }
            }

            let request = SyncApplyRequest {
                batch: SyncBatch {
                    branch: endpoint.branch.clone(),
                    operations: ops,
                },
            };

            let result = self.apply_with_transport(&transport, &request)?;
            Ok(SyncResult {
                applied: result.applied,
                conflicts: result.conflicts.len(),
                conflict_keys: result.conflicts.iter().map(|c| c.key.clone()).collect(),
            })
        }
        #[cfg(not(feature = "http-client"))]
        {
            let _ = (tree, endpoint);
            Err(AcornError::NotImplemented)
        }
    }

    /// Apply a batch using the provided transport, surfacing conflicts with kinds.
    pub fn apply_with_transport<T: SyncTransport>(
        &self,
        transport: &T,
        request: &SyncApplyRequest,
    ) -> AcornResult<SyncApplyResult> {
        let response = transport
            .apply(request)
            .map_err(|e| AcornError::Trunk(format!("sync apply failed: {:?}", e)))?;
        Ok(SyncApplyResult {
            applied: response.applied,
            conflicts: response.conflicts,
        })
    }

    /// Push local keys/values using a provided transport (useful for tests or alternate transports).
    pub fn push_with_transport<T, S, X>(
        &self,
        transport: &X,
        tree: &Tree<T, S>,
        branch: &BranchId,
    ) -> AcornResult<SyncResult>
    where
        T: Clone + Send + Sync + 'static + std::fmt::Debug + Serialize,
        S: Trunk<T> + KeyedTrunk<T> + Clone,
        X: SyncTransport,
    {
        let mut ops = Vec::new();
        for key in tree.trunk().keys(branch) {
            if let Some(nut) = tree.get(&key)? {
                let bytes =
                    serde_json::to_vec(&nut.value).map_err(|e| AcornError::Serialization(e.to_string()))?;
                ops.push(SyncMutation::Put {
                    key: key.clone(),
                    value: bytes,
                    version: tree.trunk().version(branch, &key),
                });
            }
        }
        let request = SyncApplyRequest {
            batch: SyncBatch {
                branch: branch.clone(),
                operations: ops,
            },
        };
        let result = self.apply_with_transport(transport, &request)?;
        Ok(SyncResult {
            applied: result.applied,
            conflicts: result.conflicts.len(),
            conflict_keys: result.conflicts.iter().map(|c| c.key.clone()).collect(),
        })
    }

    /// Pull mutations using the provided transport.
    pub fn pull_with_transport<T: SyncTransport>(
        &self,
        transport: &T,
        branch: &BranchId,
    ) -> AcornResult<SyncPullResponse> {
        transport
            .pull(branch)
            .map_err(|e| AcornError::Trunk(format!("sync pull failed: {:?}", e)))
    }
}

/// Subscription placeholder; will be backed by streaming updates.
#[derive(Debug, Default)]
pub struct Subscription;

impl Subscription {
    pub async fn subscribe<T, S>(_tree: &Tree<T, S>) -> AcornResult<Self>
    where
        T: Clone + Send + Sync + 'static,
        S: Trunk<T> + Send + Sync,
    {
        Ok(Subscription)
    }

    pub async fn next_event(&self) -> AcornResult<Option<SyncEvent>> {
        Err(AcornError::NotImplemented)
    }
}

/// Streaming events emitted during sync.
#[derive(Debug, Clone)]
pub enum SyncEvent {
    Applied { key: String },
    Conflict { key: String },
    Heartbeat,
}

/// Mutations applied during sync; optional version enables optimistic concurrency.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum SyncMutation {
    Put {
        key: String,
        value: Vec<u8>,
        version: Option<u64>,
    },
    Delete {
        key: String,
        version: Option<u64>,
    },
}

/// Batch of operations scoped to a branch.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SyncBatch {
    pub branch: BranchId,
    pub operations: Vec<SyncMutation>,
}

/// Payload for apply requests.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SyncApplyRequest {
    pub batch: SyncBatch,
}

/// Result of applying a batch on the server.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SyncApplyResponse {
    pub applied: usize,
    pub conflicts: Vec<SyncConflict>,
}

/// Response payload for pull requests (ops plus version snapshot).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SyncPullResponse {
    pub batch: SyncBatch,
    pub versions: Vec<(String, u64)>,
    pub deleted: Vec<String>,
    pub deleted_versions: Vec<(String, Option<u64>)>,
}

/// Conflict surface returned by sync operations.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SyncConflict {
    pub key: String,
    pub remote_value: Option<Vec<u8>>,
    pub local_value: Option<Vec<u8>>,
    pub remote_version: Option<u64>,
    pub local_version: Option<u64>,
    pub kind: SyncConflictKind,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum SyncConflictKind {
    VersionMismatch,
    MissingKey,
    Unknown,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SyncApplyResult {
    pub applied: usize,
    pub conflicts: Vec<SyncConflict>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SyncErrorResponse {
    pub error: String,
}

#[derive(Debug, Clone)]
pub struct SyncResult {
    pub applied: usize,
    pub conflicts: usize,
    pub conflict_keys: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use acorn_core::{Nut, Tree};
    use acorn_trunk_mem::MemoryTrunk;
    #[cfg(feature = "http-client")]
    use std::sync::Arc;

    #[derive(Debug, Default)]
    struct MockTransport {
        apply_response: Option<SyncApplyResponse>,
        apply_error: Option<SyncError>,
    }

    impl SyncTransport for MockTransport {
        fn apply(&self, _request: &SyncApplyRequest) -> Result<SyncApplyResponse, SyncError> {
            if let Some(err) = &self.apply_error {
                return Err(err.clone());
            }
            self.apply_response
                .clone()
                .ok_or_else(|| SyncError::Unknown("missing mock apply response".into()))
        }

        fn pull(&self, _branch: &BranchId) -> Result<SyncPullResponse, SyncError> {
            Err(SyncError::Unknown("pull not mocked".into()))
        }
    }

    #[test]
    fn surfaces_conflict_kinds_from_transport() {
        let conflicts = vec![SyncConflict {
            key: "k1".into(),
            remote_value: Some(vec![1]),
            local_value: Some(vec![2]),
            remote_version: Some(2),
            local_version: Some(1),
            kind: SyncConflictKind::VersionMismatch,
        }];
        let transport = MockTransport {
            apply_response: Some(SyncApplyResponse {
                applied: 0,
                conflicts: conflicts.clone(),
            }),
            apply_error: None,
        };
        let client = SyncClient;
        let req = SyncApplyRequest {
            batch: SyncBatch {
                branch: BranchId::new("b"),
                operations: vec![],
            },
        };

        let result = client.apply_with_transport(&transport, &req).unwrap();
        assert_eq!(result.applied, 0);
        assert_eq!(result.conflicts.len(), 1);
        assert!(matches!(
            result.conflicts[0].kind,
            SyncConflictKind::VersionMismatch
        ));
    }

    #[test]
    fn maps_transport_error_to_acorn_error() {
        let transport = MockTransport {
            apply_response: None,
            apply_error: Some(SyncError::Network("down".into())),
        };
        let client = SyncClient;
        let req = SyncApplyRequest {
            batch: SyncBatch {
                branch: BranchId::new("b"),
                operations: vec![],
            },
        };

        let result = client.apply_with_transport(&transport, &req);
        assert!(matches!(result, Err(AcornError::Trunk(_))));
    }

    #[test]
    fn loopback_transport_enforces_versions() {
        #[derive(Clone)]
        struct LoopbackTransport {
            trunk: MemoryTrunk,
        }

        impl SyncTransport for LoopbackTransport {
            fn apply(&self, request: &SyncApplyRequest) -> Result<SyncApplyResponse, SyncError> {
                let mut applied = 0usize;
                let mut conflicts = Vec::new();
                for op in &request.batch.operations {
                    match op {
                        SyncMutation::Put { key, value, version } => {
                            let current = self.trunk.version(&request.batch.branch, key);
                            if let Some(expected) = version {
                                if current != Some(*expected) {
                                    conflicts.push(SyncConflict {
                                        key: key.clone(),
                                        remote_value: self
                                            .trunk
                                            .get(&request.batch.branch, key)
                                            .unwrap()
                                            .map(|n| n.value),
                                        local_value: Some(value.clone()),
                                        remote_version: current,
                                        local_version: Some(*expected),
                                        kind: SyncConflictKind::VersionMismatch,
                                    });
                                    continue;
                                }
                            }
                            let _ = self
                                .trunk
                                .put(&request.batch.branch, key, Nut { value: value.clone() });
                            applied += 1;
                        }
                        SyncMutation::Delete { key, version } => {
                            let current = self.trunk.version(&request.batch.branch, key);
                            if let Some(expected) = version {
                                if current != Some(*expected) {
                                    conflicts.push(SyncConflict {
                                        key: key.clone(),
                                        remote_value: self
                                            .trunk
                                            .get(&request.batch.branch, key)
                                            .unwrap()
                                            .map(|n| n.value),
                                        local_value: None,
                                        remote_version: current,
                                        local_version: Some(*expected),
                                        kind: SyncConflictKind::VersionMismatch,
                                    });
                                    continue;
                                }
                            }
                            if self.trunk.get(&request.batch.branch, key).unwrap().is_none() {
                                conflicts.push(SyncConflict {
                                    key: key.clone(),
                                    remote_value: None,
                                    local_value: None,
                                    remote_version: current,
                                    local_version: *version,
                                    kind: SyncConflictKind::MissingKey,
                                });
                                continue;
                            }
                            let _ = self.trunk.delete(&request.batch.branch, key);
                            applied += 1;
                        }
                    }
                }
                Ok(SyncApplyResponse { applied, conflicts })
            }

            fn pull(&self, branch: &BranchId) -> Result<SyncPullResponse, SyncError> {
                let mut ops = Vec::new();
                let mut versions = Vec::new();
                let mut deleted = Vec::new();
                for key in self.trunk.keys(branch) {
                    if let Some(nut) = self.trunk.get(branch, &key).unwrap() {
                        ops.push(SyncMutation::Put {
                            key: key.clone(),
                            value: nut.value,
                            version: self.trunk.version(branch, &key),
                        });
                        if let Some(v) = self.trunk.version(branch, &key) {
                            versions.push((key.clone(), v));
                        }
                    } else {
                        deleted.push(key);
                    }
                }
                Ok(SyncPullResponse {
                    batch: SyncBatch {
                        branch: branch.clone(),
                        operations: ops,
                    },
                    versions,
                    deleted,
                    deleted_versions: Vec::new(),
                })
            }
        }

        let transport = LoopbackTransport {
            trunk: MemoryTrunk::new(),
        };
        let client = SyncClient;
        let branch = BranchId::new("loop");
        let tree = Tree::new(branch.clone(), transport.trunk.clone());

        // seed v1
        tree.put(
            "key",
            Nut {
                value: b"v1".to_vec(),
            },
        )
        .unwrap();

        // attempt apply with stale version triggers conflict
        let apply = SyncApplyRequest {
            batch: SyncBatch {
                branch: branch.clone(),
                operations: vec![SyncMutation::Put {
                    key: "key".into(),
                    value: b"v2".to_vec(),
                    version: Some(0),
                }],
            },
        };
        let result = client.apply_with_transport(&transport, &apply).unwrap();
        assert_eq!(result.applied, 0);
        assert_eq!(result.conflicts.len(), 1);
        assert!(matches!(
            result.conflicts[0].kind,
            SyncConflictKind::VersionMismatch
        ));

        // pull returns latest
        let pull = client.pull_with_transport(&transport, &branch).unwrap();
        assert_eq!(pull.batch.operations.len(), 1);
    }

    #[test]
    fn push_with_transport_surfaces_conflict() {
        #[derive(Clone)]
        struct RemoteTransport {
            remote: MemoryTrunk,
        }

        impl SyncTransport for RemoteTransport {
            fn apply(&self, request: &SyncApplyRequest) -> Result<SyncApplyResponse, SyncError> {
                let mut applied = 0usize;
                let mut conflicts = Vec::new();
                for op in &request.batch.operations {
                    match op {
                        SyncMutation::Put { key, value, version } => {
                            let current = self.remote.version(&request.batch.branch, key);
                            if let Some(expected) = version {
                                if current != Some(*expected) {
                                    conflicts.push(SyncConflict {
                                        key: key.clone(),
                                        remote_value: self
                                            .remote
                                            .get(&request.batch.branch, key)
                                            .unwrap()
                                            .map(|n| n.value),
                                        local_value: Some(value.clone()),
                                        remote_version: current,
                                        local_version: Some(*expected),
                                        kind: SyncConflictKind::VersionMismatch,
                                    });
                                    continue;
                                }
                            }
                            let _ = self
                                .remote
                                .put(&request.batch.branch, key, Nut { value: value.clone() });
                            applied += 1;
                        }
                        SyncMutation::Delete { key, version } => {
                            let current = self.remote.version(&request.batch.branch, key);
                            if let Some(expected) = version {
                                if current != Some(*expected) {
                                    conflicts.push(SyncConflict {
                                        key: key.clone(),
                                        remote_value: self
                                            .remote
                                            .get(&request.batch.branch, key)
                                            .unwrap()
                                            .map(|n| n.value),
                                        local_value: None,
                                        remote_version: current,
                                        local_version: Some(*expected),
                                        kind: SyncConflictKind::VersionMismatch,
                                    });
                                    continue;
                                }
                            }
                            if self.remote.get(&request.batch.branch, key).unwrap().is_none() {
                                conflicts.push(SyncConflict {
                                    key: key.clone(),
                                    remote_value: None,
                                    local_value: None,
                                    remote_version: current,
                                    local_version: *version,
                                    kind: SyncConflictKind::MissingKey,
                                });
                                continue;
                            }
                            let _ = self.remote.delete(&request.batch.branch, key);
                            applied += 1;
                        }
                    }
                }
                Ok(SyncApplyResponse { applied, conflicts })
            }

            fn pull(&self, branch: &BranchId) -> Result<SyncPullResponse, SyncError> {
                let mut ops = Vec::new();
                for key in self.remote.keys(branch) {
                    if let Some(nut) = self.remote.get(branch, &key).unwrap() {
                        ops.push(SyncMutation::Put {
                            key: key.clone(),
                            value: nut.value,
                            version: self.remote.version(branch, &key),
                        });
                    }
                }
                Ok(SyncPullResponse {
                    batch: SyncBatch {
                        branch: branch.clone(),
                        operations: ops,
                    },
                    versions: vec![],
                    deleted: Vec::new(),
                    deleted_versions: Vec::new(),
                })
            }
        }

        let client = SyncClient;
        let branch = BranchId::new("push");

        // local trunk at version 1
        let local_trunk = MemoryTrunk::new();
        let tree = Tree::new(branch.clone(), local_trunk.clone());
        tree.put(
            "key",
            Nut {
                value: b"local".to_vec(),
            },
        )
        .unwrap(); // v1

        // remote trunk at version 2
        let remote_trunk = MemoryTrunk::new();
        remote_trunk
            .put(
                &branch,
                "key",
                Nut {
                    value: b"remote1".to_vec(),
                },
            )
            .unwrap();
        remote_trunk
            .put(
                &branch,
                "key",
                Nut {
                    value: b"remote2".to_vec(),
                },
            )
            .unwrap();

        let transport = RemoteTransport { remote: remote_trunk };
        let result = client.push_with_transport(&transport, &tree, &branch).unwrap();
        assert_eq!(result.applied, 0);
        assert_eq!(result.conflicts, 1);
        assert_eq!(result.conflict_keys, vec!["key".to_string()]);
    }

    #[cfg(feature = "http-client")]
    #[tokio::test]
    async fn http_transport_detects_version_conflict() {
        use axum::{
            extract::State,
            routing::{get, post},
            Json, Router,
        };
        use tokio::net::TcpListener;
        use tokio::task::JoinHandle;

        #[derive(Clone)]
        struct HttpState {
            trunk: Arc<MemoryTrunk>,
        }

        async fn apply_handler(
            State(state): State<HttpState>,
            Json(payload): Json<SyncApplyRequest>,
        ) -> Json<SyncApplyResponse> {
            let mut applied = 0usize;
            let mut conflicts = Vec::new();

            for op in &payload.batch.operations {
                match op {
                    SyncMutation::Put { key, value, version } => {
                        let current = state.trunk.version(&payload.batch.branch, key);
                        if let Some(expected) = version {
                            if current != Some(*expected) {
                                conflicts.push(SyncConflict {
                                    key: key.clone(),
                                    remote_value: state
                                        .trunk
                                        .get(&payload.batch.branch, key)
                                        .unwrap()
                                        .map(|n| n.value),
                                    local_value: Some(value.clone()),
                                    remote_version: current,
                                    local_version: Some(*expected),
                                    kind: SyncConflictKind::VersionMismatch,
                                });
                                continue;
                            }
                        }
                        let _ = state
                            .trunk
                            .put(&payload.batch.branch, key, Nut { value: value.clone() });
                        applied += 1;
                    }
                    SyncMutation::Delete { key, version } => {
                        let current = state.trunk.version(&payload.batch.branch, key);
                        if let Some(expected) = version {
                            if current != Some(*expected) {
                                conflicts.push(SyncConflict {
                                    key: key.clone(),
                                    remote_value: state
                                        .trunk
                                        .get(&payload.batch.branch, key)
                                        .unwrap()
                                        .map(|n| n.value),
                                    local_value: None,
                                    remote_version: current,
                                    local_version: Some(*expected),
                                    kind: SyncConflictKind::VersionMismatch,
                                });
                                continue;
                            }
                        }
                        if state.trunk.get(&payload.batch.branch, key).unwrap().is_none() {
                            conflicts.push(SyncConflict {
                                key: key.clone(),
                                remote_value: None,
                                local_value: None,
                                remote_version: current,
                                local_version: *version,
                                kind: SyncConflictKind::MissingKey,
                            });
                            continue;
                        }
                        let _ = state.trunk.delete(&payload.batch.branch, key);
                        applied += 1;
                    }
                }
            }

            Json(SyncApplyResponse { applied, conflicts })
        }

        async fn pull_handler(State(state): State<HttpState>) -> Json<SyncPullResponse> {
            let branch = BranchId::new("main");
            let mut ops = Vec::new();
            let mut versions = Vec::new();
            let mut deleted = Vec::new();
            let mut deleted_versions = Vec::new();
            for key in state.trunk.keys(&branch) {
                if let Some(nut) = state.trunk.get(&branch, &key).unwrap() {
                    ops.push(SyncMutation::Put {
                        key: key.clone(),
                        value: nut.value,
                        version: state.trunk.version(&branch, &key),
                    });
                    if let Some(v) = state.trunk.version(&branch, &key) {
                        versions.push((key.clone(), v));
                    }
                } else {
                    deleted_versions.push((key.clone(), state.trunk.version(&branch, &key)));
                    deleted.push(key);
                }
            }
            Json(SyncPullResponse {
                batch: SyncBatch {
                    branch,
                    operations: ops,
                },
                versions,
                deleted,
                deleted_versions,
            })
        }

        async fn serve(state: HttpState) -> (SocketAddr, JoinHandle<()>) {
            use std::net::SocketAddr;
            let app = Router::new()
                .route("/sync/apply", post(apply_handler))
                .route("/sync/pull", get(pull_handler))
                .with_state(state);

            let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
            let addr = listener.local_addr().unwrap();
            let handle = tokio::spawn(async move {
                axum::serve(listener, app.into_make_service()).await.unwrap();
            });
            (addr, handle)
        }

        let trunk = Arc::new(MemoryTrunk::new());
        let state = HttpState { trunk: trunk.clone() };

        // seed initial version (v1)
        trunk
            .put(
                &BranchId::new("main"),
                "key",
                Nut {
                    value: b"v1".to_vec(),
                },
            )
            .unwrap();

        let (addr, handle) = serve(state).await;
        let transport = HttpTransport::new(format!("http://{}", addr));
        let client = SyncClient;
        let branch = BranchId::new("main");

        // apply with stale expectation (0 instead of 1) should conflict
        let apply = SyncApplyRequest {
            batch: SyncBatch {
                branch: branch.clone(),
                operations: vec![SyncMutation::Put {
                    key: "key".into(),
                    value: b"v2".to_vec(),
                    version: Some(0),
                }],
            },
        };
        let result = client.apply_with_transport(&transport, &apply).unwrap();
        assert_eq!(result.applied, 0);
        assert_eq!(result.conflicts.len(), 1);
        assert!(matches!(
            result.conflicts[0].kind,
            SyncConflictKind::VersionMismatch
        ));

        // delete a missing key to trigger missing-key conflict
        let delete_missing = SyncApplyRequest {
            batch: SyncBatch {
                branch: branch.clone(),
                operations: vec![SyncMutation::Delete {
                    key: "absent".into(),
                    version: None,
                }],
            },
        };
        let delete_result = client.apply_with_transport(&transport, &delete_missing).unwrap();
        assert_eq!(delete_result.applied, 0);
        assert_eq!(delete_result.conflicts.len(), 1);
        assert!(matches!(
            delete_result.conflicts[0].kind,
            SyncConflictKind::MissingKey
        ));

        // pull should return current value and version
        let pull = client.pull_with_transport(&transport, &branch).unwrap();
        assert_eq!(pull.batch.operations.len(), 1);

        // shutdown server
        handle.abort();
    }
}
