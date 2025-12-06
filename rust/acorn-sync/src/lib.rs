#![allow(dead_code)]

use acorn_core::{AcornError, AcornResult, BranchId, Tree, Trunk};
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
        self.block_on(
            self.client
                .get(url)
                .query(&[("branch", branch.as_str())])
                .send(),
        )?
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
    #[instrument(skip(self, _tree))]
    pub async fn synchronize<T, S>(&self, _tree: &Tree<T, S>, _endpoint: &SyncEndpoint) -> AcornResult<()>
    where
        T: Clone + Send + Sync + 'static + std::fmt::Debug,
        S: Trunk<T> + Send + Sync,
    {
        Err(AcornError::NotImplemented)
    }

    #[instrument(skip(self))]
    pub async fn pull<T, S>(&self, _tree: &Tree<T, S>, _endpoint: &SyncEndpoint) -> AcornResult<()>
    where
        T: Clone + Send + Sync + 'static + std::fmt::Debug,
        S: Trunk<T> + Send + Sync,
    {
        Err(AcornError::NotImplemented)
    }

    #[instrument(skip(self))]
    pub async fn push<T, S>(&self, _tree: &Tree<T, S>, _endpoint: &SyncEndpoint) -> AcornResult<()>
    where
        T: Clone + Send + Sync + 'static + std::fmt::Debug,
        S: Trunk<T> + Send + Sync,
    {
        Err(AcornError::NotImplemented)
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use acorn_core::{Nut, Tree};
    use acorn_trunk_mem::MemoryTrunk;

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
        assert!(matches!(result.conflicts[0].kind, SyncConflictKind::VersionMismatch));
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
                                        remote_value: self.trunk.get(&request.batch.branch, key).unwrap().map(|n| n.value),
                                        local_value: Some(value.clone()),
                                        remote_version: current,
                                        local_version: Some(*expected),
                                        kind: SyncConflictKind::VersionMismatch,
                                    });
                                    continue;
                                }
                            }
                            let _ = self.trunk.put(&request.batch.branch, key, Nut { value: value.clone() });
                            applied += 1;
                        }
                        SyncMutation::Delete { key, version } => {
                            let current = self.trunk.version(&request.batch.branch, key);
                            if let Some(expected) = version {
                                if current != Some(*expected) {
                                    conflicts.push(SyncConflict {
                                        key: key.clone(),
                                        remote_value: self.trunk.get(&request.batch.branch, key).unwrap().map(|n| n.value),
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
                for key in self.trunk.keys(branch) {
                    if let Some(nut) = self.trunk.get(branch, &key).unwrap() {
                        ops.push(SyncMutation::Put {
                            key: key.clone(),
                            value: nut.value,
                            version: self.trunk.version(branch, &key),
                        });
                    }
                }
                Ok(SyncPullResponse {
                    batch: SyncBatch {
                        branch: branch.clone(),
                        operations: ops,
                    },
                    versions: vec![],
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
        tree.put("key", Nut { value: b"v1".to_vec() }).unwrap();

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
        assert!(matches!(result.conflicts[0].kind, SyncConflictKind::VersionMismatch));

        // pull returns latest
        let pull = client.pull_with_transport(&transport, &branch).unwrap();
        assert_eq!(pull.batch.operations.len(), 1);
    }
}
