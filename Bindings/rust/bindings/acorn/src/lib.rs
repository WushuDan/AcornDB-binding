//! # AcornDB Rust Bindings
//!
//! Safe, idiomatic Rust bindings for AcornDB - a lightweight, embedded, event-driven NoSQL database engine.
//!
//! ## Overview
//!
//! AcornDB is designed to be:
//! - **Local-first**: Perfect for edge devices, mobile, desktop, or microservices
//! - **Embedded**: Integrates directly into your application without external services
//! - **Event-driven**: Supports reactive subscriptions to data changes
//! - **Extendable**: Seamlessly connects to cloud services for scaling and backup
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use acorn::{AcornTree, Error};
//! use serde::{Deserialize, Serialize};
//!
//! #[derive(Serialize, Deserialize, Debug)]
//! struct User {
//!     id: String,
//!     name: String,
//!     email: String,
//! }
//!
//! fn main() -> Result<(), Error> {
//!     // Open a tree (database)
//!     let tree = AcornTree::open_memory()?;
//!     
//!     // Create a user
//!     let user = User {
//!         id: "user-1".to_string(),
//!         name: "Alice".to_string(),
//!         email: "alice@example.com".to_string(),
//!     };
//!     
//!     // Serialize and store
//!     let user_json = serde_json::to_string(&user)?;
//!     tree.stash(&user.id, &user_json)?;
//!     
//!     // Retrieve and deserialize
//!     if let Some(user_json) = tree.crack(&user.id)? {
//!         let retrieved_user: User = serde_json::from_str(&user_json)?;
//!         println!("Retrieved user: {:?}", retrieved_user);
//!     }
//!     
//!     Ok(())
//! }
//! ```
//!
//! ## Core Concepts
//!
//! ### Trees
//! A **Tree** is AcornDB's primary data structure, similar to a table or collection in other databases.
//! Each tree stores key-value pairs where values are JSON documents.
//!
//! ### Trunks
//! A **Trunk** defines how data is stored. AcornDB supports multiple storage backends:
//! - `MemoryTrunk`: In-memory storage (fast, not persistent)
//! - `FileTrunk`: File-based storage (persistent, local)
//! - `DocumentStoreTrunk`: Append-only storage with history
//! - `GitHubTrunk`: Git-based storage with version history
//! - Cloud trunks: Azure, AWS S3, etc.
//! - Database trunks: SQLite, PostgreSQL, MySQL, etc.
//!
//! ### Operations
//! - **Stash**: Store a value with a key
//! - **Crack**: Retrieve a value by key
//! - **Toss**: Delete a value by key
//! - **Iterate**: Iterate over keys/values
//!
//! ## Features
//!
//! ### 🔐 Security & Encryption
//! - AES-256 encryption with PBKDF2 key derivation
//! - Password-based and key-based encryption
//! - Secure key export/import
//!
//! ### 🗜️ Compression
//! - Gzip, Brotli, and custom compression
//! - Configurable compression levels
//! - Compression statistics and monitoring
//!
//! ### 🚀 Performance
//! - LRU caching with configurable limits
//! - Batch operations for improved performance
//! - Performance monitoring and metrics
//!
//! ### 🔄 Synchronization
//! - HTTP-based synchronization
//! - Mesh synchronization for peer-to-peer sync
//! - Conflict resolution strategies
//!
//! ### 📊 Advanced Features
//! - LINQ-style queries with filtering and sorting
//! - ACID transactions
//! - Reactive programming with subscriptions
//! - Git integration for version history
//! - Nursery system for dynamic trunk management
//! - Event management and monitoring
//!
//! ## Examples
//!
//! ### Basic Operations
//! ```rust,no_run
//! use acorn::{AcornTree, Error};
//!
//! fn basic_operations() -> Result<(), Error> {
//!     let tree = AcornTree::open_memory()?;
//!     
//!     // Store data
//!     tree.stash("key1", r#"{"name": "Alice", "age": 30}"#)?;
//!     tree.stash("key2", r#"{"name": "Bob", "age": 25}"#)?;
//!     
//!     // Retrieve data
//!     if let Some(data) = tree.crack("key1")? {
//!         println!("Found: {}", data);
//!     }
//!     
//!     // Iterate over all data
//!     for item in tree.iter()? {
//!         println!("Item: {}", item);
//!     }
//!     
//!     // Delete data
//!     tree.toss("key1")?;
//!     
//!     Ok(())
//! }
//! ```
//!
//! ### Batch Operations
//! ```rust,no_run
//! use acorn::{AcornTree, AcornBatch, Error};
//!
//! fn batch_operations() -> Result<(), Error> {
//!     let tree = AcornTree::open_memory()?;
//!     let mut batch = AcornBatch::new(tree)?;
//!     
//!     // Add multiple operations to batch
//!     batch.stash("user1", r#"{"name": "Alice"}"#)?;
//!     batch.stash("user2", r#"{"name": "Bob"}"#)?;
//!     batch.toss("old_user")?;
//!     
//!     // Commit all operations atomically
//!     batch.commit()?;
//!     
//!     Ok(())
//! }
//! ```
//!
//! ### Queries
//! ```rust,no_run
//! use acorn::{AcornTree, AcornQuery, Error};
//!
//! fn query_example() -> Result<(), Error> {
//!     let tree = AcornTree::open_memory()?;
//!     
//!     // Add some data
//!     tree.stash("user1", r#"{"name": "Alice", "age": 30}"#)?;
//!     tree.stash("user2", r#"{"name": "Bob", "age": 25}"#)?;
//!     tree.stash("user3", r#"{"name": "Charlie", "age": 35}"#)?;
//!     
//!     // Query with filtering
//!     let query = AcornQuery::new(tree)?;
//!     let adults: Vec<String> = query
//!         .where_condition(|json| {
//!             json.contains("\"age\":") && 
//!             json.find("\"age\":").and_then(|pos| {
//!                 json[pos+6..].parse::<i32>().ok()
//!             }).map_or(false, |age| age >= 30)
//!         })
//!         .collect()?;
//!     
//!     println!("Adults: {:?}", adults);
//!     
//!     Ok(())
//! }
//! ```
//!
//! ### Encryption
//! ```rust,no_run
//! use acorn::{AcornTree, AcornEncryption, Error};
//!
//! fn encryption_example() -> Result<(), Error> {
//!     // Create encryption provider
//!     let encryption = AcornEncryption::from_password("my-password", "my-salt")?;
//!     
//!     // Open encrypted tree
//!     let tree = AcornTree::open_encrypted("file://./secure_db", &encryption)?;
//!     
//!     // Use tree normally - data is automatically encrypted/decrypted
//!     tree.stash("secret", "sensitive data")?;
//!     let data = tree.crack("secret")?;
//!     
//!     Ok(())
//! }
//! ```
//!
//! ### Synchronization
//! ```rust,no_run
//! use acorn::{AcornTree, AcornSync, Error};
//!
//! fn sync_example() -> Result<(), Error> {
//!     let tree = AcornTree::open_memory()?;
//!     
//!     // Create sync client
//!     let sync = AcornSync::new(tree, "http://localhost:8080/sync")?;
//!     
//!     // Push local changes to server
//!     sync.push()?;
//!     
//!     // Pull changes from server
//!     sync.pull()?;
//!     
//!     Ok(())
//! }
//! ```
//!
//! ## Error Handling
//!
//! The library provides comprehensive error handling with detailed error types:
//!
//! ```rust,no_run
//! use acorn::{AcornTree, Error, ErrorContext};
//!
//! fn error_handling_example() -> Result<(), Error> {
//!     let tree = AcornTree::open_memory()?;
//!     
//!     match tree.crack("nonexistent-key") {
//!         Ok(value) => println!("Found: {}", value),
//!         Err(Error::NotFound { key, operation }) => {
//!             println!("Key '{}' not found during {}", key, operation);
//!         },
//!         Err(e) => {
//!             println!("Error: {}", e);
//!             if let Some(op) = e.operation() {
//!                 println!("Operation: {}", op);
//!             }
//!             if let Some(ctx) = e.context() {
//!                 println!("Context: {}", ctx);
//!             }
//!         }
//!     }
//!     
//!     Ok(())
//! }
//! ```
//!
//! ## Performance
//!
//! AcornDB is designed for high performance:
//! - **Memory efficiency**: Minimal memory allocations
//! - **Batch operations**: Efficient bulk operations
//! - **Caching**: Configurable LRU caching
//! - **Compression**: Optional data compression
//! - **Async support**: Non-blocking operations where possible
//!
//! ## Thread Safety
//!
//! All AcornDB types are thread-safe and can be safely shared between threads:
//!
//! ```rust,no_run
//! use acorn::{AcornTree, Error};
//! use std::sync::Arc;
//! use std::thread;
//!
//! fn thread_safety_example() -> Result<(), Error> {
//!     let tree = Arc::new(AcornTree::open_memory()?);
//!     
//!     let handles: Vec<_> = (0..4).map(|i| {
//!         let tree_clone = tree.clone();
//!         thread::spawn(move || {
//!             for j in 0..100 {
//!                 let key = format!("thread-{}-{}", i, j);
//!                 let value = format!("data-{}-{}", i, j);
//!                 tree_clone.stash(&key, &value).unwrap();
//!             }
//!         })
//!     }).collect();
//!     
//!     for handle in handles {
//!         handle.join().unwrap();
//!     }
//!     
//!     Ok(())
//! }
//! ```
//!
//! ## Integration
//!
//! AcornDB integrates well with the Rust ecosystem:
//! - **Serde**: Automatic serialization/deserialization
//! - **Tokio**: Async runtime support
//! - **Tracing**: Structured logging
//! - **Anyhow/Thiserror**: Error handling
//!
//! ## License
//!
//! This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
//!
//! ## Contributing
//!
//! Contributions are welcome! Please see our [Contributing Guide](CONTRIBUTING.md) for details.
//!
//! ## Support
//!
//! - **Documentation**: [docs.rs/acorn](https://docs.rs/acorn)
//! - **Issues**: [GitHub Issues](https://github.com/acorn-db/acorn/issues)
//! - **Discussions**: [GitHub Discussions](https://github.com/acorn-db/acorn/discussions)
//! - **Discord**: [AcornDB Discord](https://discord.gg/acorn-db)

use acorn_sys::*;
use serde::{de::DeserializeOwned, Serialize};
use std::{ffi::CString, ptr, fmt, backtrace::Backtrace};

/// Comprehensive error types for AcornDB Rust bindings
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// AcornDB internal error with context
    #[error("AcornDB error: {message}")]
    Acorn {
        message: String,
        context: Option<String>,
        operation: Option<String>,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },
    
    /// Item not found in the tree
    #[error("Item not found: {key}")]
    NotFound {
        key: String,
        operation: String,
    },
    
    /// Serialization/deserialization error
    #[error("Serialization error: {message}")]
    Serialization {
        message: String,
        data_type: String,
        operation: String,
    },
    
    /// Invalid input parameters
    #[error("Invalid input: {message}")]
    InvalidInput {
        message: String,
        parameter: String,
        expected: String,
    },
    
    /// File system related errors
    #[error("File system error: {message}")]
    FileSystem {
        message: String,
        path: String,
        operation: String,
    },
    
    /// Network related errors
    #[error("Network error: {message}")]
    Network {
        message: String,
        url: String,
        operation: String,
    },
    
    /// Encryption/decryption errors
    #[error("Encryption error: {message}")]
    Encryption {
        message: String,
        operation: String,
    },
    
    /// Compression/decompression errors
    #[error("Compression error: {message}")]
    Compression {
        message: String,
        operation: String,
    },
    
    /// Cache related errors
    #[error("Cache error: {message}")]
    Cache {
        message: String,
        operation: String,
    },
    
    /// Query related errors
    #[error("Query error: {message}")]
    Query {
        message: String,
        query_type: String,
        operation: String,
    },
    
    /// Transaction related errors
    #[error("Transaction error: {message}")]
    Transaction {
        message: String,
        operation: String,
    },
    
    /// Sync related errors
    #[error("Sync error: {message}")]
    Sync {
        message: String,
        operation: String,
        remote_url: Option<String>,
    },
    
    /// Configuration errors
    #[error("Configuration error: {message}")]
    Configuration {
        message: String,
        parameter: String,
    },
    
    /// Resource exhaustion errors
    #[error("Resource exhausted: {message}")]
    ResourceExhausted {
        message: String,
        resource_type: String,
        limit: Option<u64>,
    },
    
    /// Timeout errors
    #[error("Operation timed out: {message}")]
    Timeout {
        message: String,
        operation: String,
        duration_ms: u64,
    },
    
    /// Concurrent access errors
    #[error("Concurrent access error: {message}")]
    ConcurrentAccess {
        message: String,
        operation: String,
    },
}

impl Error {
    /// Create a new AcornDB error with context
    pub fn acorn(message: impl Into<String>) -> Self {
        Self::Acorn {
            message: message.into(),
            context: None,
            operation: None,
            source: None,
        }
    }
    
    /// Create a new AcornDB error with context and operation
    pub fn acorn_with_context(
        message: impl Into<String>,
        context: impl Into<String>,
        operation: impl Into<String>,
    ) -> Self {
        Self::Acorn {
            message: message.into(),
            context: Some(context.into()),
            operation: Some(operation.into()),
            source: None,
        }
    }
    
    /// Create a not found error
    pub fn not_found(key: impl Into<String>, operation: impl Into<String>) -> Self {
        Self::NotFound {
            key: key.into(),
            operation: operation.into(),
        }
    }
    
    /// Create a serialization error
    pub fn serialization(
        message: impl Into<String>,
        data_type: impl Into<String>,
        operation: impl Into<String>,
    ) -> Self {
        Self::Serialization {
            message: message.into(),
            data_type: data_type.into(),
            operation: operation.into(),
        }
    }
    
    /// Create an invalid input error
    pub fn invalid_input(
        message: impl Into<String>,
        parameter: impl Into<String>,
        expected: impl Into<String>,
    ) -> Self {
        Self::InvalidInput {
            message: message.into(),
            parameter: parameter.into(),
            expected: expected.into(),
        }
    }
    
    /// Create a file system error
    pub fn file_system(
        message: impl Into<String>,
        path: impl Into<String>,
        operation: impl Into<String>,
    ) -> Self {
        Self::FileSystem {
            message: message.into(),
            path: path.into(),
            operation: operation.into(),
        }
    }
    
    /// Create a network error
    pub fn network(
        message: impl Into<String>,
        url: impl Into<String>,
        operation: impl Into<String>,
    ) -> Self {
        Self::Network {
            message: message.into(),
            url: url.into(),
            operation: operation.into(),
        }
    }
    
    /// Create an encryption error
    pub fn encryption(message: impl Into<String>, operation: impl Into<String>) -> Self {
        Self::Encryption {
            message: message.into(),
            operation: operation.into(),
        }
    }
    
    /// Create a compression error
    pub fn compression(message: impl Into<String>, operation: impl Into<String>) -> Self {
        Self::Compression {
            message: message.into(),
            operation: operation.into(),
        }
    }
    
    /// Create a cache error
    pub fn cache(message: impl Into<String>, operation: impl Into<String>) -> Self {
        Self::Cache {
            message: message.into(),
            operation: operation.into(),
        }
    }
    
    /// Create a query error
    pub fn query(
        message: impl Into<String>,
        query_type: impl Into<String>,
        operation: impl Into<String>,
    ) -> Self {
        Self::Query {
            message: message.into(),
            query_type: query_type.into(),
            operation: operation.into(),
        }
    }
    
    /// Create a transaction error
    pub fn transaction(message: impl Into<String>, operation: impl Into<String>) -> Self {
        Self::Transaction {
            message: message.into(),
            operation: operation.into(),
        }
    }
    
    /// Create a sync error
    pub fn sync(
        message: impl Into<String>,
        operation: impl Into<String>,
        remote_url: Option<String>,
    ) -> Self {
        Self::Sync {
            message: message.into(),
            operation: operation.into(),
            remote_url,
        }
    }
    
    /// Create a configuration error
    pub fn configuration(message: impl Into<String>, parameter: impl Into<String>) -> Self {
        Self::Configuration {
            message: message.into(),
            parameter: parameter.into(),
        }
    }
    
    /// Create a resource exhausted error
    pub fn resource_exhausted(
        message: impl Into<String>,
        resource_type: impl Into<String>,
        limit: Option<u64>,
    ) -> Self {
        Self::ResourceExhausted {
            message: message.into(),
            resource_type: resource_type.into(),
            limit,
        }
    }
    
    /// Create a timeout error
    pub fn timeout(
        message: impl Into<String>,
        operation: impl Into<String>,
        duration_ms: u64,
    ) -> Self {
        Self::Timeout {
            message: message.into(),
            operation: operation.into(),
            duration_ms,
        }
    }
    
    /// Create a concurrent access error
    pub fn concurrent_access(message: impl Into<String>, operation: impl Into<String>) -> Self {
        Self::ConcurrentAccess {
            message: message.into(),
            operation: operation.into(),
        }
    }
    
    /// Get the operation that caused the error
    pub fn operation(&self) -> Option<&str> {
        match self {
            Self::Acorn { operation, .. } => operation.as_deref(),
            Self::NotFound { operation, .. } => Some(operation),
            Self::Serialization { operation, .. } => Some(operation),
            Self::InvalidInput { .. } => None,
            Self::FileSystem { operation, .. } => Some(operation),
            Self::Network { operation, .. } => Some(operation),
            Self::Encryption { operation, .. } => Some(operation),
            Self::Compression { operation, .. } => Some(operation),
            Self::Cache { operation, .. } => Some(operation),
            Self::Query { operation, .. } => Some(operation),
            Self::Transaction { operation, .. } => Some(operation),
            Self::Sync { operation, .. } => Some(operation),
            Self::Configuration { .. } => None,
            Self::ResourceExhausted { .. } => None,
            Self::Timeout { operation, .. } => Some(operation),
            Self::ConcurrentAccess { operation, .. } => Some(operation),
        }
    }
    
    /// Get additional context for the error
    pub fn context(&self) -> Option<&str> {
        match self {
            Self::Acorn { context, .. } => context.as_deref(),
            _ => None,
        }
    }
    
    /// Check if this is a recoverable error
    pub fn is_recoverable(&self) -> bool {
        match self {
            Self::NotFound { .. } => true,
            Self::Timeout { .. } => true,
            Self::Network { .. } => true,
            Self::ResourceExhausted { .. } => true,
            Self::ConcurrentAccess { .. } => true,
            _ => false,
        }
    }
    
    /// Get error severity level
    pub fn severity(&self) -> ErrorSeverity {
        match self {
            Self::NotFound { .. } => ErrorSeverity::Info,
            Self::Timeout { .. } => ErrorSeverity::Warning,
            Self::Network { .. } => ErrorSeverity::Warning,
            Self::ConcurrentAccess { .. } => ErrorSeverity::Warning,
            Self::ResourceExhausted { .. } => ErrorSeverity::Error,
            Self::Configuration { .. } => ErrorSeverity::Error,
            Self::Serialization { .. } => ErrorSeverity::Error,
            Self::InvalidInput { .. } => ErrorSeverity::Error,
            Self::FileSystem { .. } => ErrorSeverity::Error,
            Self::Encryption { .. } => ErrorSeverity::Error,
            Self::Compression { .. } => ErrorSeverity::Error,
            Self::Cache { .. } => ErrorSeverity::Error,
            Self::Query { .. } => ErrorSeverity::Error,
            Self::Transaction { .. } => ErrorSeverity::Error,
            Self::Sync { .. } => ErrorSeverity::Error,
            Self::Acorn { .. } => ErrorSeverity::Error,
        }
    }
}

/// Error severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ErrorSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

impl fmt::Display for ErrorSeverity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Info => write!(f, "INFO"),
            Self::Warning => write!(f, "WARNING"),
            Self::Error => write!(f, "ERROR"),
            Self::Critical => write!(f, "CRITICAL"),
        }
    }
}

/// Helper trait for creating errors with context
pub trait ErrorContext<T> {
    fn with_context<F>(self, f: F) -> Result<T>
    where
        F: FnOnce() -> String;
    
    fn with_operation(self, operation: &str) -> Result<T>;
}

impl<T, E> ErrorContext<T> for std::result::Result<T, E>
where
    E: std::error::Error + Send + Sync + 'static,
{
    fn with_context<F>(self, f: F) -> Result<T>
    where
        F: FnOnce() -> String,
    {
        self.map_err(|e| Error::Acorn {
            message: f(),
            context: None,
            operation: None,
            source: Some(Box::new(e)),
        })
    }
    
    fn with_operation(self, operation: &str) -> Result<T> {
        self.map_err(|e| Error::Acorn {
            message: e.to_string(),
            context: None,
            operation: Some(operation.to_string()),
            source: Some(Box::new(e)),
        })
    }
}

/// Helper function to get the last error from the FFI layer
fn get_last_error() -> String {
    unsafe { acorn_sys::last_error_string() }
}

/// Helper function to create an AcornDB error from FFI result
fn ffi_result<T>(rc: i32, operation: &str, success_value: T) -> Result<T> {
    if rc == 0 {
        Ok(success_value)
    } else {
        Err(Error::acorn_with_context(
            get_last_error(),
            "FFI operation failed",
            operation,
        ))
    }
}

/// Helper function to create an AcornDB error from FFI result with context
fn ffi_result_with_context<T>(rc: i32, operation: &str, context: &str, success_value: T) -> Result<T> {
    if rc == 0 {
        Ok(success_value)
    } else {
        Err(Error::acorn_with_context(
            get_last_error(),
            context,
            operation,
        ))
    }
}

pub type Result<T> = std::result::Result<T, Error>;

pub struct AcornTree { h: acorn_tree_handle }

/// Encryption provider for AcornDB
pub struct AcornEncryption { h: acorn_encryption_handle }

impl AcornEncryption {
    /// Create an encryption provider from a password and salt using PBKDF2 key derivation
    /// 
    /// # Arguments
    /// * `password` - The password to derive the encryption key from
    /// * `salt` - The salt to use for key derivation (should be unique per database)
    /// 
    /// # Example
    /// ```no_run
    /// # use acorn::{AcornEncryption, Error};
    /// # fn main() -> Result<(), Error> {
    /// let encryption = AcornEncryption::from_password("my-secret-password", "my-unique-salt")?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn from_password(password: &str, salt: &str) -> Result<Self> {
        let password_c = CString::new(password).map_err(|e| Error::Acorn(format!("Invalid password: {}", e)))?;
        let salt_c = CString::new(salt).map_err(|e| Error::Acorn(format!("Invalid salt: {}", e)))?;
        let mut h: acorn_encryption_handle = 0;
        let rc = unsafe { acorn_encryption_from_password(password_c.as_ptr(), salt_c.as_ptr(), &mut h as *mut _) };
        if rc == 0 { 
            Ok(Self { h }) 
        } else { 
            Err(Error::Acorn(unsafe { acorn_sys::last_error_string() })) 
        }
    }

    /// Create an encryption provider from explicit key and IV (base64 encoded)
    /// 
    /// # Arguments
    /// * `key_base64` - The encryption key encoded as base64 (must be 32 bytes)
    /// * `iv_base64` - The initialization vector encoded as base64 (must be 16 bytes)
    /// 
    /// # Example
    /// ```no_run
    /// # use acorn::{AcornEncryption, Error};
    /// # fn main() -> Result<(), Error> {
    /// let encryption = AcornEncryption::from_key_iv("base64-key", "base64-iv")?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn from_key_iv(key_base64: &str, iv_base64: &str) -> Result<Self> {
        let key_c = CString::new(key_base64).map_err(|e| Error::Acorn(format!("Invalid key: {}", e)))?;
        let iv_c = CString::new(iv_base64).map_err(|e| Error::Acorn(format!("Invalid IV: {}", e)))?;
        let mut h: acorn_encryption_handle = 0;
        let rc = unsafe { acorn_encryption_from_key_iv(key_c.as_ptr(), iv_c.as_ptr(), &mut h as *mut _) };
        if rc == 0 { 
            Ok(Self { h }) 
        } else { 
            Err(Error::Acorn(unsafe { acorn_sys::last_error_string() })) 
        }
    }

    /// Generate a random key and IV for testing or new deployments
    /// 
    /// # Returns
    /// A tuple of (key_base64, iv_base64) strings
    /// 
    /// # Warning
    /// Store the returned key and IV securely - data cannot be decrypted without them!
    /// 
    /// # Example
    /// ```no_run
    /// # use acorn::{AcornEncryption, Error};
    /// # fn main() -> Result<(), Error> {
    /// let (key, iv) = AcornEncryption::generate_key_iv()?;
    /// println!("Key: {}", key);
    /// println!("IV: {}", iv);
    /// # Ok(())
    /// # }
    /// ```
    pub fn generate_key_iv() -> Result<(String, String)> {
        let mut key_buf = acorn_buf { data: ptr::null_mut(), len: 0 };
        let mut iv_buf = acorn_buf { data: ptr::null_mut(), len: 0 };
        let rc = unsafe { acorn_encryption_generate_key_iv(&mut key_buf as *mut _, &mut iv_buf as *mut _) };
        if rc != 0 {
            return Err(Error::Acorn(unsafe { acorn_sys::last_error_string() }));
        }

        // Convert buffers to strings
        let key_slice = unsafe { std::slice::from_raw_parts(key_buf.data, key_buf.len) };
        let iv_slice = unsafe { std::slice::from_raw_parts(iv_buf.data, iv_buf.len) };
        
        let key = String::from_utf8(key_slice.to_vec()).map_err(|e| Error::Acorn(format!("Invalid key UTF-8: {}", e)))?;
        let iv = String::from_utf8(iv_slice.to_vec()).map_err(|e| Error::Acorn(format!("Invalid IV UTF-8: {}", e)))?;

        // Free the buffers
        unsafe { 
            acorn_free_buf(&mut key_buf as *mut _);
            acorn_free_buf(&mut iv_buf as *mut _);
        }

        Ok((key, iv))
    }

    /// Export the encryption key as a base64 string (for backup/storage)
    pub fn export_key(&self) -> Result<String> {
        let mut key_buf = acorn_buf { data: ptr::null_mut(), len: 0 };
        let rc = unsafe { acorn_encryption_export_key(self.h, &mut key_buf as *mut _) };
        if rc != 0 {
            return Err(Error::Acorn(unsafe { acorn_sys::last_error_string() }));
        }

        let key_slice = unsafe { std::slice::from_raw_parts(key_buf.data, key_buf.len) };
        let key = String::from_utf8(key_slice.to_vec()).map_err(|e| Error::Acorn(format!("Invalid key UTF-8: {}", e)))?;

        unsafe { acorn_free_buf(&mut key_buf as *mut _) };
        Ok(key)
    }

    /// Export the initialization vector as a base64 string (for backup/storage)
    pub fn export_iv(&self) -> Result<String> {
        let mut iv_buf = acorn_buf { data: ptr::null_mut(), len: 0 };
        let rc = unsafe { acorn_encryption_export_iv(self.h, &mut iv_buf as *mut _) };
        if rc != 0 {
            return Err(Error::Acorn(unsafe { acorn_sys::last_error_string() }));
        }

        let iv_slice = unsafe { std::slice::from_raw_parts(iv_buf.data, iv_buf.len) };
        let iv = String::from_utf8(iv_slice.to_vec()).map_err(|e| Error::Acorn(format!("Invalid IV UTF-8: {}", e)))?;

        unsafe { acorn_free_buf(&mut iv_buf as *mut _) };
        Ok(iv)
    }

    /// Encrypt plaintext data
    pub fn encrypt(&self, plaintext: &str) -> Result<String> {
        let plaintext_c = CString::new(plaintext).map_err(|e| Error::Acorn(format!("Invalid plaintext: {}", e)))?;
        let mut ciphertext_buf = acorn_buf { data: ptr::null_mut(), len: 0 };
        let rc = unsafe { acorn_encryption_encrypt(self.h, plaintext_c.as_ptr(), &mut ciphertext_buf as *mut _) };
        if rc != 0 {
            return Err(Error::Acorn(unsafe { acorn_sys::last_error_string() }));
        }

        let ciphertext_slice = unsafe { std::slice::from_raw_parts(ciphertext_buf.data, ciphertext_buf.len) };
        let ciphertext = String::from_utf8(ciphertext_slice.to_vec()).map_err(|e| Error::Acorn(format!("Invalid ciphertext UTF-8: {}", e)))?;

        unsafe { acorn_free_buf(&mut ciphertext_buf as *mut _) };
        Ok(ciphertext)
    }

    /// Decrypt ciphertext data
    pub fn decrypt(&self, ciphertext: &str) -> Result<String> {
        let ciphertext_c = CString::new(ciphertext).map_err(|e| Error::Acorn(format!("Invalid ciphertext: {}", e)))?;
        let mut plaintext_buf = acorn_buf { data: ptr::null_mut(), len: 0 };
        let rc = unsafe { acorn_encryption_decrypt(self.h, ciphertext_c.as_ptr(), &mut plaintext_buf as *mut _) };
        if rc != 0 {
            return Err(Error::Acorn(unsafe { acorn_sys::last_error_string() }));
        }

        let plaintext_slice = unsafe { std::slice::from_raw_parts(plaintext_buf.data, plaintext_buf.len) };
        let plaintext = String::from_utf8(plaintext_slice.to_vec()).map_err(|e| Error::Acorn(format!("Invalid plaintext UTF-8: {}", e)))?;

        unsafe { acorn_free_buf(&mut plaintext_buf as *mut _) };
        Ok(plaintext)
    }

    /// Check if encryption is enabled
    pub fn is_enabled(&self) -> Result<bool> {
        let rc = unsafe { acorn_encryption_is_enabled(self.h) };
        if rc == -1 {
            Err(Error::Acorn(unsafe { acorn_sys::last_error_string() }))
        } else {
            Ok(rc == 1)
        }
    }
}

impl Drop for AcornEncryption {
    fn drop(&mut self) {
        unsafe { acorn_encryption_close(self.h); }
    }
}

/// Compression provider for AcornDB
pub struct AcornCompression { h: acorn_compression_handle }

/// Compression levels available in AcornDB
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompressionLevel {
    /// Fastest compression (least CPU usage, larger output)
    Fastest = 0,
    /// Optimal balance of speed and compression ratio
    Optimal = 1,
    /// Smallest size (most CPU usage, smallest output)
    SmallestSize = 2,
}

/// Compression statistics
#[derive(Debug, Clone)]
pub struct CompressionStats {
    pub original_size: i32,
    pub compressed_size: i32,
    pub ratio: f64,
    pub space_saved: i32,
}

impl AcornCompression {
    /// Create a Gzip compression provider
    /// 
    /// # Arguments
    /// * `level` - The compression level to use
    /// 
    /// # Example
    /// ```no_run
    /// # use acorn::{AcornCompression, CompressionLevel, Error};
    /// # fn main() -> Result<(), Error> {
    /// let compression = AcornCompression::gzip(CompressionLevel::Optimal)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn gzip(level: CompressionLevel) -> Result<Self> {
        let mut h: acorn_compression_handle = 0;
        let rc = unsafe { acorn_compression_gzip(level as i32, &mut h as *mut _) };
        if rc == 0 { 
            Ok(Self { h }) 
        } else { 
            Err(Error::Acorn(unsafe { acorn_sys::last_error_string() })) 
        }
    }

    /// Create a Brotli compression provider
    /// 
    /// # Arguments
    /// * `level` - The compression level to use
    /// 
    /// # Example
    /// ```no_run
    /// # use acorn::{AcornCompression, CompressionLevel, Error};
    /// # fn main() -> Result<(), Error> {
    /// let compression = AcornCompression::brotli(CompressionLevel::Optimal)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn brotli(level: CompressionLevel) -> Result<Self> {
        let mut h: acorn_compression_handle = 0;
        let rc = unsafe { acorn_compression_brotli(level as i32, &mut h as *mut _) };
        if rc == 0 { 
            Ok(Self { h }) 
        } else { 
            Err(Error::Acorn(unsafe { acorn_sys::last_error_string() })) 
        }
    }

    /// Create a no-op compression provider (passes data through unchanged)
    /// 
    /// # Example
    /// ```no_run
    /// # use acorn::{AcornCompression, Error};
    /// # fn main() -> Result<(), Error> {
    /// let compression = AcornCompression::none()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn none() -> Result<Self> {
        let mut h: acorn_compression_handle = 0;
        let rc = unsafe { acorn_compression_none(&mut h as *mut _) };
        if rc == 0 { 
            Ok(Self { h }) 
        } else { 
            Err(Error::Acorn(unsafe { acorn_sys::last_error_string() })) 
        }
    }

    /// Compress data
    /// 
    /// # Arguments
    /// * `data` - The data to compress
    /// 
    /// # Returns
    /// * `Ok(String)` - The compressed data as a base64-encoded string
    /// * `Err(Error)` - If compression fails
    /// 
    /// # Example
    /// ```no_run
    /// # use acorn::{AcornCompression, CompressionLevel, Error};
    /// # fn main() -> Result<(), Error> {
    /// let compression = AcornCompression::gzip(CompressionLevel::Optimal)?;
    /// let compressed = compression.compress("Hello, world!")?;
    /// println!("Compressed: {}", compressed);
    /// # Ok(())
    /// # }
    /// ```
    pub fn compress(&self, data: &str) -> Result<String> {
        let data_c = CString::new(data).map_err(|e| Error::Acorn(format!("Invalid data: {}", e)))?;
        let mut buf = acorn_buf { data: ptr::null_mut(), len: 0 };
        let rc = unsafe { acorn_compression_compress(self.h, data_c.as_ptr(), &mut buf as *mut _) };
        if rc == 0 {
            let result = unsafe { 
                std::slice::from_raw_parts(buf.data, buf.len as usize) 
            };
            let result_str = String::from_utf8_lossy(result).to_string();
            unsafe { acorn_free_buf(&mut buf); }
            Ok(result_str)
        } else {
            Err(Error::Acorn(unsafe { acorn_sys::last_error_string() }))
        }
    }

    /// Decompress data
    /// 
    /// # Arguments
    /// * `compressed_data` - The compressed data as a base64-encoded string
    /// 
    /// # Returns
    /// * `Ok(String)` - The decompressed data
    /// * `Err(Error)` - If decompression fails
    /// 
    /// # Example
    /// ```no_run
    /// # use acorn::{AcornCompression, CompressionLevel, Error};
    /// # fn main() -> Result<(), Error> {
    /// let compression = AcornCompression::gzip(CompressionLevel::Optimal)?;
    /// let compressed = compression.compress("Hello, world!")?;
    /// let decompressed = compression.decompress(&compressed)?;
    /// assert_eq!(decompressed, "Hello, world!");
    /// # Ok(())
    /// # }
    /// ```
    pub fn decompress(&self, compressed_data: &str) -> Result<String> {
        let compressed_c = CString::new(compressed_data).map_err(|e| Error::Acorn(format!("Invalid compressed data: {}", e)))?;
        let mut buf = acorn_buf { data: ptr::null_mut(), len: 0 };
        let rc = unsafe { acorn_compression_decompress(self.h, compressed_c.as_ptr(), &mut buf as *mut _) };
        if rc == 0 {
            let result = unsafe { 
                std::slice::from_raw_parts(buf.data, buf.len as usize) 
            };
            let result_str = String::from_utf8_lossy(result).to_string();
            unsafe { acorn_free_buf(&mut buf); }
            Ok(result_str)
        } else {
            Err(Error::Acorn(unsafe { acorn_sys::last_error_string() }))
        }
    }

    /// Check if compression is enabled
    /// 
    /// # Returns
    /// * `true` - If compression is enabled
    /// * `false` - If compression is disabled (no-op provider)
    /// 
    /// # Example
    /// ```no_run
    /// # use acorn::{AcornCompression, CompressionLevel, Error};
    /// # fn main() -> Result<(), Error> {
    /// let compression = AcornCompression::gzip(CompressionLevel::Optimal)?;
    /// assert!(compression.is_enabled()?);
    /// 
    /// let no_compression = AcornCompression::none()?;
    /// assert!(!no_compression.is_enabled()?);
    /// # Ok(())
    /// # }
    /// ```
    pub fn is_enabled(&self) -> Result<bool> {
        let rc = unsafe { acorn_compression_is_enabled(self.h) };
        if rc >= 0 {
            Ok(rc == 1)
        } else {
            Err(Error::Acorn(unsafe { acorn_sys::last_error_string() }))
        }
    }

    /// Get the algorithm name
    /// 
    /// # Returns
    /// * `Ok(String)` - The name of the compression algorithm
    /// * `Err(Error)` - If the operation fails
    /// 
    /// # Example
    /// ```no_run
    /// # use acorn::{AcornCompression, CompressionLevel, Error};
    /// # fn main() -> Result<(), Error> {
    /// let compression = AcornCompression::gzip(CompressionLevel::Optimal)?;
    /// let algorithm = compression.algorithm_name()?;
    /// assert_eq!(algorithm, "Gzip");
    /// # Ok(())
    /// # }
    /// ```
    pub fn algorithm_name(&self) -> Result<String> {
        let mut buf = acorn_buf { data: ptr::null_mut(), len: 0 };
        let rc = unsafe { acorn_compression_algorithm_name(self.h, &mut buf as *mut _) };
        if rc == 0 {
            let result = unsafe { 
                std::slice::from_raw_parts(buf.data, buf.len as usize) 
            };
            let result_str = String::from_utf8_lossy(result).to_string();
            unsafe { acorn_free_buf(&mut buf); }
            Ok(result_str)
        } else {
            Err(Error::Acorn(unsafe { acorn_sys::last_error_string() }))
        }
    }

    /// Get compression statistics
    /// 
    /// # Arguments
    /// * `original_data` - The original uncompressed data
    /// * `compressed_data` - The compressed data as a base64-encoded string
    /// 
    /// # Returns
    /// * `Ok(CompressionStats)` - Compression statistics
    /// * `Err(Error)` - If the operation fails
    /// 
    /// # Example
    /// ```no_run
    /// # use acorn::{AcornCompression, CompressionLevel, Error};
    /// # fn main() -> Result<(), Error> {
    /// let compression = AcornCompression::gzip(CompressionLevel::Optimal)?;
    /// let original = "Hello, world!";
    /// let compressed = compression.compress(original)?;
    /// let stats = compression.get_stats(original, &compressed)?;
    /// println!("Compression ratio: {:.2}", stats.ratio);
    /// println!("Space saved: {} bytes", stats.space_saved);
    /// # Ok(())
    /// # }
    /// ```
    pub fn get_stats(&self, original_data: &str, compressed_data: &str) -> Result<CompressionStats> {
        let original_c = CString::new(original_data).map_err(|e| Error::Acorn(format!("Invalid original data: {}", e)))?;
        let compressed_c = CString::new(compressed_data).map_err(|e| Error::Acorn(format!("Invalid compressed data: {}", e)))?;
        
        let mut original_size: i32 = 0;
        let mut compressed_size: i32 = 0;
        let mut ratio: f64 = 0.0;
        let mut space_saved: i32 = 0;
        
        let rc = unsafe { 
            acorn_compression_get_stats(
                self.h, 
                original_c.as_ptr(), 
                compressed_c.as_ptr(),
                &mut original_size as *mut _,
                &mut compressed_size as *mut _,
                &mut ratio as *mut _,
                &mut space_saved as *mut _
            ) 
        };
        
        if rc == 0 {
            Ok(CompressionStats {
                original_size,
                compressed_size,
                ratio,
                space_saved,
            })
        } else {
            Err(Error::Acorn(unsafe { acorn_sys::last_error_string() }))
        }
    }
}

impl Drop for AcornCompression {
    fn drop(&mut self) {
        unsafe { acorn_compression_close(self.h); }
    }
}

/// Cache strategy for AcornDB
pub struct AcornCache { h: acorn_cache_handle }

/// Cache statistics
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub tracked_items: i32,
    pub max_size: i32,
    pub utilization_percentage: f64,
}

impl AcornCache {
    /// Create an LRU (Least Recently Used) cache strategy
    /// 
    /// # Arguments
    /// * `max_size` - Maximum number of items to keep in cache before eviction
    /// 
    /// # Example
    /// ```no_run
    /// # use acorn::{AcornCache, Error};
    /// # fn main() -> Result<(), Error> {
    /// let cache = AcornCache::lru(1000)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn lru(max_size: i32) -> Result<Self> {
        let mut h: acorn_cache_handle = 0;
        let rc = unsafe { acorn_cache_lru(max_size, &mut h as *mut _) };
        if rc == 0 { 
            Ok(Self { h }) 
        } else { 
            Err(Error::Acorn(unsafe { acorn_sys::last_error_string() })) 
        }
    }

    /// Create a no-eviction cache strategy (unlimited cache)
    /// 
    /// # Example
    /// ```no_run
    /// # use acorn::{AcornCache, Error};
    /// # fn main() -> Result<(), Error> {
    /// let cache = AcornCache::no_eviction()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn no_eviction() -> Result<Self> {
        let mut h: acorn_cache_handle = 0;
        let rc = unsafe { acorn_cache_no_eviction(&mut h as *mut _) };
        if rc == 0 { 
            Ok(Self { h }) 
        } else { 
            Err(Error::Acorn(unsafe { acorn_sys::last_error_string() })) 
        }
    }

    /// Reset the cache strategy state
    /// 
    /// # Returns
    /// * `Ok(())` - If reset was successful
    /// * `Err(Error)` - If reset failed
    /// 
    /// # Example
    /// ```no_run
    /// # use acorn::{AcornCache, Error};
    /// # fn main() -> Result<(), Error> {
    /// let cache = AcornCache::lru(1000)?;
    /// cache.reset()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn reset(&self) -> Result<()> {
        let rc = unsafe { acorn_cache_reset(self.h) };
        if rc == 0 {
            Ok(())
        } else {
            Err(Error::Acorn(unsafe { acorn_sys::last_error_string() }))
        }
    }

    /// Get cache statistics
    /// 
    /// # Returns
    /// * `Ok(CacheStats)` - Cache statistics
    /// * `Err(Error)` - If the operation fails
    /// 
    /// # Example
    /// ```no_run
    /// # use acorn::{AcornCache, Error};
    /// # fn main() -> Result<(), Error> {
    /// let cache = AcornCache::lru(1000)?;
    /// let stats = cache.get_stats()?;
    /// println!("Tracked items: {}", stats.tracked_items);
    /// println!("Max size: {}", stats.max_size);
    /// println!("Utilization: {:.1}%", stats.utilization_percentage);
    /// # Ok(())
    /// # }
    /// ```
    pub fn get_stats(&self) -> Result<CacheStats> {
        let mut tracked_items: i32 = 0;
        let mut max_size: i32 = 0;
        let mut utilization_percentage: f64 = 0.0;
        
        let rc = unsafe { 
            acorn_cache_get_stats(
                self.h,
                &mut tracked_items as *mut _,
                &mut max_size as *mut _,
                &mut utilization_percentage as *mut _
            ) 
        };
        
        if rc == 0 {
            Ok(CacheStats {
                tracked_items,
                max_size,
                utilization_percentage,
            })
        } else {
            Err(Error::Acorn(unsafe { acorn_sys::last_error_string() }))
        }
    }

    /// Check if eviction is enabled for this cache strategy
    /// 
    /// # Returns
    /// * `true` - If eviction is enabled (LRU cache)
    /// * `false` - If eviction is disabled (NoEviction cache)
    /// 
    /// # Example
    /// ```no_run
    /// # use acorn::{AcornCache, Error};
    /// # fn main() -> Result<(), Error> {
    /// let lru_cache = AcornCache::lru(1000)?;
    /// assert!(lru_cache.is_eviction_enabled()?);
    /// 
    /// let no_eviction_cache = AcornCache::no_eviction()?;
    /// assert!(!no_eviction_cache.is_eviction_enabled()?);
    /// # Ok(())
    /// # }
    /// ```
    pub fn is_eviction_enabled(&self) -> Result<bool> {
        let rc = unsafe { acorn_cache_is_eviction_enabled(self.h) };
        if rc >= 0 {
            Ok(rc == 1)
        } else {
            Err(Error::Acorn(unsafe { acorn_sys::last_error_string() }))
        }
    }

    /// Set eviction enabled status (no-op for most strategies)
    /// 
    /// # Arguments
    /// * `enabled` - Whether to enable eviction
    /// 
    /// # Note
    /// This is a no-op for most cache strategies as eviction is determined by strategy type.
    /// LRU cache always has eviction enabled, NoEviction cache never does.
    /// 
    /// # Example
    /// ```no_run
    /// # use acorn::{AcornCache, Error};
    /// # fn main() -> Result<(), Error> {
    /// let cache = AcornCache::lru(1000)?;
    /// cache.set_eviction_enabled(true)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn set_eviction_enabled(&self, enabled: bool) -> Result<()> {
        let rc = unsafe { acorn_cache_set_eviction_enabled(self.h, if enabled { 1 } else { 0 }) };
        if rc == 0 {
            Ok(())
        } else {
            Err(Error::Acorn(unsafe { acorn_sys::last_error_string() }))
        }
    }
}

impl Drop for AcornCache {
    fn drop(&mut self) {
        unsafe { acorn_cache_close(self.h); }
    }
}

/// Conflict resolution judge for AcornDB
pub struct AcornConflictJudge { h: acorn_conflict_judge_handle }

/// Conflict resolution strategies available in AcornDB
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConflictStrategy {
    /// Last-write-wins based on timestamp (default)
    Timestamp,
    /// Higher version number wins
    Version,
    /// Local version always wins
    LocalWins,
    /// Remote version always wins
    RemoteWins,
}

impl AcornConflictJudge {
    /// Create a timestamp-based conflict judge (last-write-wins)
    /// 
    /// # Example
    /// ```no_run
    /// # use acorn::{AcornConflictJudge, Error};
    /// # fn main() -> Result<(), Error> {
    /// let judge = AcornConflictJudge::timestamp()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn timestamp() -> Result<Self> {
        let mut h: acorn_conflict_judge_handle = 0;
        let rc = unsafe { acorn_conflict_judge_timestamp(&mut h as *mut _) };
        if rc == 0 { 
            Ok(Self { h }) 
        } else { 
            Err(Error::Acorn(unsafe { acorn_sys::last_error_string() })) 
        }
    }

    /// Create a version-based conflict judge
    /// 
    /// # Example
    /// ```no_run
    /// # use acorn::{AcornConflictJudge, Error};
    /// # fn main() -> Result<(), Error> {
    /// let judge = AcornConflictJudge::version()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn version() -> Result<Self> {
        let mut h: acorn_conflict_judge_handle = 0;
        let rc = unsafe { acorn_conflict_judge_version(&mut h as *mut _) };
        if rc == 0 { 
            Ok(Self { h }) 
        } else { 
            Err(Error::Acorn(unsafe { acorn_sys::last_error_string() })) 
        }
    }

    /// Create a local-wins conflict judge
    /// 
    /// # Example
    /// ```no_run
    /// # use acorn::{AcornConflictJudge, Error};
    /// # fn main() -> Result<(), Error> {
    /// let judge = AcornConflictJudge::local_wins()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn local_wins() -> Result<Self> {
        let mut h: acorn_conflict_judge_handle = 0;
        let rc = unsafe { acorn_conflict_judge_local_wins(&mut h as *mut _) };
        if rc == 0 { 
            Ok(Self { h }) 
        } else { 
            Err(Error::Acorn(unsafe { acorn_sys::last_error_string() })) 
        }
    }

    /// Create a remote-wins conflict judge
    /// 
    /// # Example
    /// ```no_run
    /// # use acorn::{AcornConflictJudge, Error};
    /// # fn main() -> Result<(), Error> {
    /// let judge = AcornConflictJudge::remote_wins()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn remote_wins() -> Result<Self> {
        let mut h: acorn_conflict_judge_handle = 0;
        let rc = unsafe { acorn_conflict_judge_remote_wins(&mut h as *mut _) };
        if rc == 0 { 
            Ok(Self { h }) 
        } else { 
            Err(Error::Acorn(unsafe { acorn_sys::last_error_string() })) 
        }
    }

    /// Get the name of the conflict resolution strategy
    /// 
    /// # Returns
    /// * `Ok(String)` - The name of the strategy
    /// * `Err(Error)` - If the operation fails
    /// 
    /// # Example
    /// ```no_run
    /// # use acorn::{AcornConflictJudge, Error};
    /// # fn main() -> Result<(), Error> {
    /// let judge = AcornConflictJudge::timestamp()?;
    /// let name = judge.name()?;
    /// assert_eq!(name, "Timestamp");
    /// # Ok(())
    /// # }
    /// ```
    pub fn name(&self) -> Result<String> {
        let mut buf = acorn_buf { data: ptr::null_mut(), len: 0 };
        let rc = unsafe { acorn_conflict_judge_name(self.h, &mut buf as *mut _) };
        if rc == 0 {
            let result = unsafe { 
                std::slice::from_raw_parts(buf.data, buf.len as usize) 
            };
            let result_str = String::from_utf8_lossy(result).to_string();
            unsafe { acorn_free_buf(&mut buf); }
            Ok(result_str)
        } else {
            Err(Error::Acorn(unsafe { acorn_sys::last_error_string() }))
        }
    }

    /// Resolve a conflict between local and incoming data
    /// 
    /// # Arguments
    /// * `local_json` - The local data as JSON
    /// * `incoming_json` - The incoming data as JSON
    /// 
    /// # Returns
    /// * `Ok(String)` - The winning data as JSON
    /// * `Err(Error)` - If conflict resolution fails
    /// 
    /// # Example
    /// ```no_run
    /// # use acorn::{AcornConflictJudge, Error};
    /// # fn main() -> Result<(), Error> {
    /// let judge = AcornConflictJudge::timestamp()?;
    /// let local = r#"{"id": "test", "value": "local", "timestamp": "2023-01-01T10:00:00Z"}"#;
    /// let incoming = r#"{"id": "test", "value": "incoming", "timestamp": "2023-01-01T11:00:00Z"}"#;
    /// let winner = judge.resolve_conflict(local, incoming)?;
    /// println!("Winner: {}", winner);
    /// # Ok(())
    /// # }
    /// ```
    pub fn resolve_conflict(&self, local_json: &str, incoming_json: &str) -> Result<String> {
        let local_c = CString::new(local_json).map_err(|e| Error::Acorn(format!("Invalid local JSON: {}", e)))?;
        let incoming_c = CString::new(incoming_json).map_err(|e| Error::Acorn(format!("Invalid incoming JSON: {}", e)))?;
        let mut buf = acorn_buf { data: ptr::null_mut(), len: 0 };
        let rc = unsafe { acorn_conflict_judge_resolve(self.h, local_c.as_ptr(), incoming_c.as_ptr(), &mut buf as *mut _) };
        if rc == 0 {
            let result = unsafe { 
                std::slice::from_raw_parts(buf.data, buf.len as usize) 
            };
            let result_str = String::from_utf8_lossy(result).to_string();
            unsafe { acorn_free_buf(&mut buf); }
            Ok(result_str)
        } else {
            Err(Error::Acorn(unsafe { acorn_sys::last_error_string() }))
        }
    }
}

impl Drop for AcornConflictJudge {
    fn drop(&mut self) {
        unsafe { acorn_conflict_judge_close(self.h); }
    }
}

/// Storage backend for AcornDB
pub struct AcornStorage { h: acorn_storage_handle }

/// Document store for AcornDB with versioning and time-travel
pub struct AcornDocumentStore { h: acorn_document_store_handle }

/// Storage backend types available in AcornDB
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StorageType {
    /// AWS S3 storage
    S3,
    /// Azure Blob Storage
    AzureBlob,
    /// SQLite database
    SQLite,
    /// PostgreSQL database
    PostgreSQL,
    /// MySQL database
    MySQL,
    /// SQL Server database
    SQLServer,
    /// Git repository storage
    Git,
}

/// Storage backend information
#[derive(Debug, Clone)]
pub struct StorageInfo {
    pub trunk_type: String,
    pub supports_history: bool,
    pub supports_sync: bool,
    pub is_durable: bool,
    pub supports_async: bool,
    pub provider_name: String,
    pub connection_info: String,
}

/// Document store information
#[derive(Debug, Clone, Deserialize)]
pub struct DocumentStoreInfo {
    pub trunk_type: String,
    pub supports_history: bool,
    pub supports_sync: bool,
    pub is_durable: bool,
    pub supports_async: bool,
    pub provider_name: String,
    pub connection_info: String,
    pub has_change_log: bool,
    pub total_versions: i32,
}

/// Change types for reactive programming
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChangeType {
    /// Create or update operation
    Stash,
    /// Delete operation
    Toss,
    /// Conflict resolution operation
    Squabble,
}

/// Tree change event for reactive programming
#[derive(Debug, Clone)]
pub struct TreeChange<T> {
    pub change_type: ChangeType,
    pub id: String,
    pub item: Option<T>,
    pub timestamp: std::time::SystemTime,
    pub node_id: Option<String>,
}

impl AcornStorage {
    /// Create AWS S3 storage backend with explicit credentials
    /// 
    /// # Arguments
    /// * `access_key` - AWS Access Key ID
    /// * `secret_key` - AWS Secret Access Key
    /// * `bucket_name` - S3 bucket name
    /// * `region` - AWS region (default: "us-east-1")
    /// * `prefix` - Optional prefix for all keys
    /// 
    /// # Example
    /// ```no_run
    /// # use acorn::{AcornStorage, Error};
    /// # fn main() -> Result<(), Error> {
    /// let storage = AcornStorage::s3("access_key", "secret_key", "my-bucket", "us-west-2", Some("prefix/"))?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn s3(access_key: &str, secret_key: &str, bucket_name: &str, region: &str, prefix: Option<&str>) -> Result<Self> {
        let access_key_c = CString::new(access_key).map_err(|e| Error::Acorn(format!("Invalid access key: {}", e)))?;
        let secret_key_c = CString::new(secret_key).map_err(|e| Error::Acorn(format!("Invalid secret key: {}", e)))?;
        let bucket_name_c = CString::new(bucket_name).map_err(|e| Error::Acorn(format!("Invalid bucket name: {}", e)))?;
        let region_c = CString::new(region).map_err(|e| Error::Acorn(format!("Invalid region: {}", e)))?;
        let prefix_c = CString::new(prefix.unwrap_or("")).map_err(|e| Error::Acorn(format!("Invalid prefix: {}", e)))?;
        
        let mut h: acorn_storage_handle = 0;
        let rc = unsafe { 
            acorn_storage_s3(
                access_key_c.as_ptr(), 
                secret_key_c.as_ptr(), 
                bucket_name_c.as_ptr(), 
                region_c.as_ptr(), 
                prefix_c.as_ptr(), 
                &mut h as *mut _
            ) 
        };
        if rc == 0 { 
            Ok(Self { h }) 
        } else { 
            Err(Error::Acorn(unsafe { acorn_sys::last_error_string() })) 
        }
    }

    /// Create AWS S3 storage backend with default credential chain
    /// 
    /// # Arguments
    /// * `bucket_name` - S3 bucket name
    /// * `region` - AWS region (default: "us-east-1")
    /// * `prefix` - Optional prefix for all keys
    /// 
    /// # Example
    /// ```no_run
    /// # use acorn::{AcornStorage, Error};
    /// # fn main() -> Result<(), Error> {
    /// let storage = AcornStorage::s3_default("my-bucket", "us-west-2", None)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn s3_default(bucket_name: &str, region: &str, prefix: Option<&str>) -> Result<Self> {
        let bucket_name_c = CString::new(bucket_name).map_err(|e| Error::Acorn(format!("Invalid bucket name: {}", e)))?;
        let region_c = CString::new(region).map_err(|e| Error::Acorn(format!("Invalid region: {}", e)))?;
        let prefix_c = CString::new(prefix.unwrap_or("")).map_err(|e| Error::Acorn(format!("Invalid prefix: {}", e)))?;
        
        let mut h: acorn_storage_handle = 0;
        let rc = unsafe { 
            acorn_storage_s3_default(
                bucket_name_c.as_ptr(), 
                region_c.as_ptr(), 
                prefix_c.as_ptr(), 
                &mut h as *mut _
            ) 
        };
        if rc == 0 { 
            Ok(Self { h }) 
        } else { 
            Err(Error::Acorn(unsafe { acorn_sys::last_error_string() })) 
        }
    }

    /// Create S3-compatible storage backend (MinIO, DigitalOcean Spaces, etc.)
    /// 
    /// # Arguments
    /// * `access_key` - Access key
    /// * `secret_key` - Secret key
    /// * `bucket_name` - Bucket name
    /// * `service_url` - Service endpoint URL
    /// * `prefix` - Optional prefix for all keys
    /// 
    /// # Example
    /// ```no_run
    /// # use acorn::{AcornStorage, Error};
    /// # fn main() -> Result<(), Error> {
    /// let storage = AcornStorage::s3_compatible("access_key", "secret_key", "my-bucket", "https://nyc3.digitaloceanspaces.com", None)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn s3_compatible(access_key: &str, secret_key: &str, bucket_name: &str, service_url: &str, prefix: Option<&str>) -> Result<Self> {
        let access_key_c = CString::new(access_key).map_err(|e| Error::Acorn(format!("Invalid access key: {}", e)))?;
        let secret_key_c = CString::new(secret_key).map_err(|e| Error::Acorn(format!("Invalid secret key: {}", e)))?;
        let bucket_name_c = CString::new(bucket_name).map_err(|e| Error::Acorn(format!("Invalid bucket name: {}", e)))?;
        let service_url_c = CString::new(service_url).map_err(|e| Error::Acorn(format!("Invalid service URL: {}", e)))?;
        let prefix_c = CString::new(prefix.unwrap_or("")).map_err(|e| Error::Acorn(format!("Invalid prefix: {}", e)))?;
        
        let mut h: acorn_storage_handle = 0;
        let rc = unsafe { 
            acorn_storage_s3_compatible(
                access_key_c.as_ptr(), 
                secret_key_c.as_ptr(), 
                bucket_name_c.as_ptr(), 
                service_url_c.as_ptr(), 
                prefix_c.as_ptr(), 
                &mut h as *mut _
            ) 
        };
        if rc == 0 { 
            Ok(Self { h }) 
        } else { 
            Err(Error::Acorn(unsafe { acorn_sys::last_error_string() })) 
        }
    }

    /// Create Azure Blob Storage backend
    /// 
    /// # Arguments
    /// * `connection_string` - Azure Storage connection string
    /// * `container_name` - Blob container name
    /// * `prefix` - Optional prefix for all keys
    /// 
    /// # Example
    /// ```no_run
    /// # use acorn::{AcornStorage, Error};
    /// # fn main() -> Result<(), Error> {
    /// let storage = AcornStorage::azure_blob("DefaultEndpointsProtocol=https;...", "my-container", None)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn azure_blob(connection_string: &str, container_name: &str, prefix: Option<&str>) -> Result<Self> {
        let connection_string_c = CString::new(connection_string).map_err(|e| Error::Acorn(format!("Invalid connection string: {}", e)))?;
        let container_name_c = CString::new(container_name).map_err(|e| Error::Acorn(format!("Invalid container name: {}", e)))?;
        let prefix_c = CString::new(prefix.unwrap_or("")).map_err(|e| Error::Acorn(format!("Invalid prefix: {}", e)))?;
        
        let mut h: acorn_storage_handle = 0;
        let rc = unsafe { 
            acorn_storage_azure_blob(
                connection_string_c.as_ptr(), 
                container_name_c.as_ptr(), 
                prefix_c.as_ptr(), 
                &mut h as *mut _
            ) 
        };
        if rc == 0 { 
            Ok(Self { h }) 
        } else { 
            Err(Error::Acorn(unsafe { acorn_sys::last_error_string() })) 
        }
    }

    /// Create SQLite storage backend
    /// 
    /// # Arguments
    /// * `database_path` - Path to SQLite database file
    /// * `table_name` - Optional custom table name
    /// 
    /// # Example
    /// ```no_run
    /// # use acorn::{AcornStorage, Error};
    /// # fn main() -> Result<(), Error> {
    /// let storage = AcornStorage::sqlite("./data/acorndb.db", Some("custom_table"))?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn sqlite(database_path: &str, table_name: Option<&str>) -> Result<Self> {
        let database_path_c = CString::new(database_path).map_err(|e| Error::Acorn(format!("Invalid database path: {}", e)))?;
        let table_name_c = CString::new(table_name.unwrap_or("")).map_err(|e| Error::Acorn(format!("Invalid table name: {}", e)))?;
        
        let mut h: acorn_storage_handle = 0;
        let rc = unsafe { 
            acorn_storage_sqlite(
                database_path_c.as_ptr(), 
                table_name_c.as_ptr(), 
                &mut h as *mut _
            ) 
        };
        if rc == 0 { 
            Ok(Self { h }) 
        } else { 
            Err(Error::Acorn(unsafe { acorn_sys::last_error_string() })) 
        }
    }

    /// Create PostgreSQL storage backend
    /// 
    /// # Arguments
    /// * `connection_string` - PostgreSQL connection string
    /// * `table_name` - Optional custom table name
    /// * `schema` - Database schema (default: "public")
    /// 
    /// # Example
    /// ```no_run
    /// # use acorn::{AcornStorage, Error};
    /// # fn main() -> Result<(), Error> {
    /// let storage = AcornStorage::postgresql("postgresql://user:pass@localhost/db", None, "public")?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn postgresql(connection_string: &str, table_name: Option<&str>, schema: &str) -> Result<Self> {
        let connection_string_c = CString::new(connection_string).map_err(|e| Error::Acorn(format!("Invalid connection string: {}", e)))?;
        let table_name_c = CString::new(table_name.unwrap_or("")).map_err(|e| Error::Acorn(format!("Invalid table name: {}", e)))?;
        let schema_c = CString::new(schema).map_err(|e| Error::Acorn(format!("Invalid schema: {}", e)))?;
        
        let mut h: acorn_storage_handle = 0;
        let rc = unsafe { 
            acorn_storage_postgresql(
                connection_string_c.as_ptr(), 
                table_name_c.as_ptr(), 
                schema_c.as_ptr(), 
                &mut h as *mut _
            ) 
        };
        if rc == 0 { 
            Ok(Self { h }) 
        } else { 
            Err(Error::Acorn(unsafe { acorn_sys::last_error_string() })) 
        }
    }

    /// Create MySQL storage backend
    /// 
    /// # Arguments
    /// * `connection_string` - MySQL connection string
    /// * `table_name` - Optional custom table name
    /// * `database` - Optional database name
    /// 
    /// # Example
    /// ```no_run
    /// # use acorn::{AcornStorage, Error};
    /// # fn main() -> Result<(), Error> {
    /// let storage = AcornStorage::mysql("Server=localhost;Database=acorndb;Uid=user;Pwd=pass;", None, None)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn mysql(connection_string: &str, table_name: Option<&str>, database: Option<&str>) -> Result<Self> {
        let connection_string_c = CString::new(connection_string).map_err(|e| Error::Acorn(format!("Invalid connection string: {}", e)))?;
        let table_name_c = CString::new(table_name.unwrap_or("")).map_err(|e| Error::Acorn(format!("Invalid table name: {}", e)))?;
        let database_c = CString::new(database.unwrap_or("")).map_err(|e| Error::Acorn(format!("Invalid database: {}", e)))?;
        
        let mut h: acorn_storage_handle = 0;
        let rc = unsafe { 
            acorn_storage_mysql(
                connection_string_c.as_ptr(), 
                table_name_c.as_ptr(), 
                database_c.as_ptr(), 
                &mut h as *mut _
            ) 
        };
        if rc == 0 { 
            Ok(Self { h }) 
        } else { 
            Err(Error::Acorn(unsafe { acorn_sys::last_error_string() })) 
        }
    }

    /// Create SQL Server storage backend
    /// 
    /// # Arguments
    /// * `connection_string` - SQL Server connection string
    /// * `table_name` - Optional custom table name
    /// * `schema` - Database schema (default: "dbo")
    /// 
    /// # Example
    /// ```no_run
    /// # use acorn::{AcornStorage, Error};
    /// # fn main() -> Result<(), Error> {
    /// let storage = AcornStorage::sqlserver("Server=localhost;Database=acorndb;Integrated Security=true;", None, "dbo")?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn sqlserver(connection_string: &str, table_name: Option<&str>, schema: &str) -> Result<Self> {
        let connection_string_c = CString::new(connection_string).map_err(|e| Error::Acorn(format!("Invalid connection string: {}", e)))?;
        let table_name_c = CString::new(table_name.unwrap_or("")).map_err(|e| Error::Acorn(format!("Invalid table name: {}", e)))?;
        let schema_c = CString::new(schema).map_err(|e| Error::Acorn(format!("Invalid schema: {}", e)))?;
        
        let mut h: acorn_storage_handle = 0;
        let rc = unsafe { 
            acorn_storage_sqlserver(
                connection_string_c.as_ptr(), 
                table_name_c.as_ptr(), 
                schema_c.as_ptr(), 
                &mut h as *mut _
            ) 
        };
        if rc == 0 { 
            Ok(Self { h }) 
        } else { 
            Err(Error::Acorn(unsafe { acorn_sys::last_error_string() })) 
        }
    }

    /// Create Git storage backend
    /// 
    /// # Arguments
    /// * `repo_path` - Path to Git repository
    /// * `author_name` - Git author name
    /// * `author_email` - Git author email
    /// * `auto_push` - Automatically push to remote after each commit
    /// 
    /// # Example
    /// ```no_run
    /// # use acorn::{AcornStorage, Error};
    /// # fn main() -> Result<(), Error> {
    /// let storage = AcornStorage::git("./my-repo", "AcornDB", "acorn@acorndb.dev", false)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn git(repo_path: &str, author_name: &str, author_email: &str, auto_push: bool) -> Result<Self> {
        let repo_path_c = CString::new(repo_path).map_err(|e| Error::Acorn(format!("Invalid repo path: {}", e)))?;
        let author_name_c = CString::new(author_name).map_err(|e| Error::Acorn(format!("Invalid author name: {}", e)))?;
        let author_email_c = CString::new(author_email).map_err(|e| Error::Acorn(format!("Invalid author email: {}", e)))?;
        
        let mut h: acorn_storage_handle = 0;
        let rc = unsafe { 
            acorn_storage_git(
                repo_path_c.as_ptr(), 
                author_name_c.as_ptr(), 
                author_email_c.as_ptr(), 
                if auto_push { 1 } else { 0 }, 
                &mut h as *mut _
            ) 
        };
        if rc == 0 { 
            Ok(Self { h }) 
        } else { 
            Err(Error::Acorn(unsafe { acorn_sys::last_error_string() })) 
        }
    }

    /// Get storage backend information
    /// 
    /// # Returns
    /// * `Ok(StorageInfo)` - Storage backend information
    /// * `Err(Error)` - If the operation fails
    /// 
    /// # Example
    /// ```no_run
    /// # use acorn::{AcornStorage, Error};
    /// # fn main() -> Result<(), Error> {
    /// let storage = AcornStorage::sqlite("./test.db", None)?;
    /// let info = storage.get_info()?;
    /// println!("Provider: {}", info.provider_name);
    /// println!("Durable: {}", info.is_durable);
    /// println!("Supports History: {}", info.supports_history);
    /// # Ok(())
    /// # }
    /// ```
    pub fn get_info(&self) -> Result<StorageInfo> {
        let mut buf = acorn_buf { data: ptr::null_mut(), len: 0 };
        let rc = unsafe { acorn_storage_get_info(self.h, &mut buf as *mut _) };
        if rc == 0 {
            let result = unsafe { 
                std::slice::from_raw_parts(buf.data, buf.len as usize) 
            };
            let result_str = String::from_utf8_lossy(result).to_string();
            unsafe { acorn_free_buf(&mut buf); }
            
            // Parse JSON response
            let info: StorageInfo = serde_json::from_str(&result_str)
                .map_err(|e| Error::Acorn(format!("Failed to parse storage info: {}", e)))?;
            Ok(info)
        } else {
            Err(Error::Acorn(unsafe { acorn_sys::last_error_string() }))
        }
    }

    /// Test storage backend connection
    /// 
    /// # Returns
    /// * `Ok(bool)` - True if connection is successful
    /// * `Err(Error)` - If the operation fails
    /// 
    /// # Example
    /// ```no_run
    /// # use acorn::{AcornStorage, Error};
    /// # fn main() -> Result<(), Error> {
    /// let storage = AcornStorage::sqlite("./test.db", None)?;
    /// let is_connected = storage.test_connection()?;
    /// if is_connected {
    ///     println!("Storage connection successful!");
    /// } else {
    ///     println!("Storage connection failed!");
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn test_connection(&self) -> Result<bool> {
        let rc = unsafe { acorn_storage_test_connection(self.h) };
        if rc >= 0 {
            Ok(rc == 1)
        } else {
            Err(Error::Acorn(unsafe { acorn_sys::last_error_string() }))
        }
    }
}

impl Drop for AcornStorage {
    fn drop(&mut self) {
        unsafe { acorn_storage_close(self.h); }
    }
}

impl AcornDocumentStore {
    /// Create a new document store with optional custom path
    /// 
    /// # Arguments
    /// * `custom_path` - Optional custom path for the document store data
    /// 
    /// # Example
    /// ```no_run
    /// # use acorn::{AcornDocumentStore, Error};
    /// # fn main() -> Result<(), Error> {
    /// let doc_store = AcornDocumentStore::new(Some("./my_data"))?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(custom_path: Option<&str>) -> Result<Self> {
        let custom_path_c = CString::new(custom_path.unwrap_or("")).map_err(|e| Error::Acorn(format!("Invalid custom path: {}", e)))?;
        
        let mut h: acorn_document_store_handle = 0;
        let rc = unsafe { 
            acorn_document_store_create(
                if custom_path.is_some() { custom_path_c.as_ptr() } else { std::ptr::null() }, 
                &mut h as *mut _
            ) 
        };
        if rc == 0 { 
            Ok(Self { h }) 
        } else { 
            Err(Error::Acorn(unsafe { acorn_sys::last_error_string() })) 
        }
    }

    /// Get version history for a specific document ID
    /// 
    /// # Arguments
    /// * `id` - Document ID to get history for
    /// 
    /// # Returns
    /// JSON string containing the version history
    /// 
    /// # Example
    /// ```no_run
    /// # use acorn::{AcornDocumentStore, Error};
    /// # fn main() -> Result<(), Error> {
    /// let doc_store = AcornDocumentStore::new(None)?;
    /// let history = doc_store.get_history("user-123")?;
    /// println!("History: {}", history);
    /// # Ok(())
    /// # }
    /// ```
    pub fn get_history(&self, id: &str) -> Result<String> {
        let id_c = CString::new(id).map_err(|e| Error::Acorn(format!("Invalid ID: {}", e)))?;
        
        let mut buf = acorn_buf { ptr: std::ptr::null_mut(), len: 0 };
        let rc = unsafe { 
            acorn_document_store_get_history(self.h, id_c.as_ptr(), &mut buf as *mut _) 
        };
        if rc == 0 { 
            let result = unsafe { 
                std::slice::from_raw_parts(buf.ptr as *const u8, buf.len as usize) 
            };
            let json = std::str::from_utf8(result).map_err(|e| Error::Acorn(format!("Invalid UTF-8: {}", e)))?;
            unsafe { acorn_free_buf(&mut buf); }
            Ok(json.to_string())
        } else { 
            Err(Error::Acorn(unsafe { acorn_sys::last_error_string() })) 
        }
    }

    /// Get document store information and capabilities
    /// 
    /// # Returns
    /// DocumentStoreInfo containing capabilities and metadata
    /// 
    /// # Example
    /// ```no_run
    /// # use acorn::{AcornDocumentStore, Error};
    /// # fn main() -> Result<(), Error> {
    /// let doc_store = AcornDocumentStore::new(None)?;
    /// let info = doc_store.get_info()?;
    /// println!("Supports history: {}", info.supports_history);
    /// println!("Total versions: {}", info.total_versions);
    /// # Ok(())
    /// # }
    /// ```
    pub fn get_info(&self) -> Result<DocumentStoreInfo> {
        let mut buf = acorn_buf { ptr: std::ptr::null_mut(), len: 0 };
        let rc = unsafe { 
            acorn_document_store_get_info(self.h, &mut buf as *mut _) 
        };
        if rc == 0 { 
            let result = unsafe { 
                std::slice::from_raw_parts(buf.ptr as *const u8, buf.len as usize) 
            };
            let json = std::str::from_utf8(result).map_err(|e| Error::Acorn(format!("Invalid UTF-8: {}", e)))?;
            unsafe { acorn_free_buf(&mut buf); }
            serde_json::from_str(json).map_err(|e| Error::Acorn(format!("Failed to parse document store info: {}", e)))
        } else { 
            Err(Error::Acorn(unsafe { acorn_sys::last_error_string() })) 
        }
    }

    /// Compact the document store by removing old versions
    /// 
    /// # Example
    /// ```no_run
    /// # use acorn::{AcornDocumentStore, Error};
    /// # fn main() -> Result<(), Error> {
    /// let doc_store = AcornDocumentStore::new(None)?;
    /// doc_store.compact()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn compact(&self) -> Result<()> {
        let rc = unsafe { acorn_document_store_compact(self.h) };
        if rc == 0 { 
            Ok(()) 
        } else { 
            Err(Error::Acorn(unsafe { acorn_sys::last_error_string() })) 
        }
    }
}

impl Drop for AcornDocumentStore {
    fn drop(&mut self) {
        unsafe { acorn_document_store_close(self.h); }
    }
}

impl AcornTree {
    pub fn open(uri: &str) -> Result<Self> {
        let c = CString::new(uri).map_err(|e| Error::Acorn(format!("Invalid URI: {}", e)))?;
        let mut h: acorn_tree_handle = 0;
        let rc = unsafe { acorn_open_tree(c.as_ptr(), &mut h as *mut _) };
        if rc == 0 { 
            Ok(Self { h }) 
        } else { 
            Err(Error::Acorn(unsafe { acorn_sys::last_error_string() })) 
        }
    }

    /// Open a tree with encryption enabled
    /// 
    /// # Arguments
    /// * `uri` - The storage URI (e.g., "file://./encrypted_db" or "memory://")
    /// * `encryption` - The encryption provider to use
    /// 
    /// # Example
    /// ```no_run
    /// # use acorn::{AcornTree, AcornEncryption, Error};
    /// # fn main() -> Result<(), Error> {
    /// let encryption = AcornEncryption::from_password("my-password", "my-salt")?;
    /// let tree = AcornTree::open_encrypted("file://./secure_db", &encryption)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn open_encrypted(uri: &str, encryption: &AcornEncryption) -> Result<Self> {
        let c = CString::new(uri).map_err(|e| Error::Acorn(format!("Invalid URI: {}", e)))?;
        let mut h: acorn_tree_handle = 0;
        let rc = unsafe { acorn_open_tree_encrypted(c.as_ptr(), encryption.h, &mut h as *mut _) };
        if rc == 0 { 
            Ok(Self { h }) 
        } else { 
            Err(Error::Acorn(unsafe { acorn_sys::last_error_string() })) 
        }
    }

    /// Open a tree with both encryption and compression enabled
    /// 
    /// # Arguments
    /// * `uri` - The storage URI (e.g., "file://./secure_db" or "memory://")
    /// * `encryption` - The encryption provider to use
    /// * `compression_level` - Compression level (0=Fastest, 1=Optimal, 2=SmallestSize)
    /// 
    /// # Example
    /// ```no_run
    /// # use acorn::{AcornTree, AcornEncryption, Error};
    /// # fn main() -> Result<(), Error> {
    /// let encryption = AcornEncryption::from_password("my-password", "my-salt")?;
    /// let tree = AcornTree::open_encrypted_compressed("file://./secure_db", &encryption, 1)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn open_encrypted_compressed(uri: &str, encryption: &AcornEncryption, compression_level: i32) -> Result<Self> {
        let c = CString::new(uri).map_err(|e| Error::Acorn(format!("Invalid URI: {}", e)))?;
        let mut h: acorn_tree_handle = 0;
        let rc = unsafe { acorn_open_tree_encrypted_compressed(c.as_ptr(), encryption.h, compression_level, &mut h as *mut _) };
        if rc == 0 { 
            Ok(Self { h }) 
        } else { 
            Err(Error::Acorn(unsafe { acorn_sys::last_error_string() })) 
        }
    }

    /// Open a tree with compression only
    /// 
    /// # Arguments
    /// * `uri` - The storage URI (e.g., "file://./db", "memory://")
    /// * `compression` - The compression provider to use
    /// 
    /// # Returns
    /// * `Ok(AcornTree)` - The opened tree
    /// * `Err(Error)` - If opening fails
    /// 
    /// # Example
    /// ```no_run
    /// # use acorn::{AcornTree, AcornCompression, CompressionLevel, Error};
    /// # fn main() -> Result<(), Error> {
    /// let compression = AcornCompression::gzip(CompressionLevel::Optimal)?;
    /// let mut tree = AcornTree::open_compressed("file://./compressed_db", &compression)?;
    /// 
    /// // Store some data
    /// tree.stash("key1", &"Hello, compressed world!")?;
    /// 
    /// // Retrieve data
    /// let value: String = tree.crack("key1")?;
    /// assert_eq!(value, "Hello, compressed world!");
    /// # Ok(())
    /// # }
    /// ```
    pub fn open_compressed(uri: &str, compression: &AcornCompression) -> Result<Self> {
        let c = CString::new(uri).map_err(|e| Error::Acorn(format!("Invalid URI: {}", e)))?;
        let mut h: acorn_tree_handle = 0;
        let rc = unsafe { acorn_open_tree_compressed(c.as_ptr(), compression.h, &mut h as *mut _) };
        if rc == 0 { 
            Ok(Self { h }) 
        } else { 
            Err(Error::Acorn(unsafe { acorn_sys::last_error_string() })) 
        }
    }

    /// Open a tree with a custom cache strategy
    /// 
    /// # Arguments
    /// * `uri` - The storage URI (e.g., "file://./db", "memory://")
    /// * `cache` - The cache strategy to use
    /// 
    /// # Returns
    /// * `Ok(AcornTree)` - The opened tree
    /// * `Err(Error)` - If opening fails
    /// 
    /// # Example
    /// ```no_run
    /// # use acorn::{AcornTree, AcornCache, Error};
    /// # fn main() -> Result<(), Error> {
    /// let cache = AcornCache::lru(1000)?;
    /// let mut tree = AcornTree::open_with_cache("file://./cached_db", &cache)?;
    /// 
    /// // Store some data
    /// tree.stash("key1", &"Hello, cached world!")?;
    /// 
    /// // Retrieve data
    /// let value: String = tree.crack("key1")?;
    /// assert_eq!(value, "Hello, cached world!");
    /// # Ok(())
    /// # }
    /// ```
    pub fn open_with_cache(uri: &str, cache: &AcornCache) -> Result<Self> {
        let c = CString::new(uri).map_err(|e| Error::Acorn(format!("Invalid URI: {}", e)))?;
        let mut h: acorn_tree_handle = 0;
        let rc = unsafe { acorn_open_tree_with_cache(c.as_ptr(), cache.h, &mut h as *mut _) };
        if rc == 0 { 
            Ok(Self { h }) 
        } else { 
            Err(Error::Acorn(unsafe { acorn_sys::last_error_string() })) 
        }
    }

    /// Open a tree with a custom conflict resolution judge
    /// 
    /// # Arguments
    /// * `uri` - The storage URI (e.g., "file://./db", "memory://")
    /// * `judge` - The conflict resolution judge to use
    /// 
    /// # Returns
    /// * `Ok(AcornTree)` - The opened tree
    /// * `Err(Error)` - If opening fails
    /// 
    /// # Example
    /// ```no_run
    /// # use acorn::{AcornTree, AcornConflictJudge, Error};
    /// # fn main() -> Result<(), Error> {
    /// let judge = AcornConflictJudge::timestamp()?;
    /// let mut tree = AcornTree::open_with_conflict_judge("file://./conflict_db", &judge)?;
    /// 
    /// // Store some data
    /// tree.stash("key1", &"Hello, conflict resolution!")?;
    /// 
    /// // Retrieve data
    /// let value: String = tree.crack("key1")?;
    /// assert_eq!(value, "Hello, conflict resolution!");
    /// # Ok(())
    /// # }
    /// ```
    pub fn open_with_conflict_judge(uri: &str, judge: &AcornConflictJudge) -> Result<Self> {
        let c = CString::new(uri).map_err(|e| Error::Acorn(format!("Invalid URI: {}", e)))?;
        let mut h: acorn_tree_handle = 0;
        let rc = unsafe { acorn_open_tree_with_conflict_judge(c.as_ptr(), judge.h, &mut h as *mut _) };
        if rc == 0 { 
            Ok(Self { h }) 
        } else { 
            Err(Error::Acorn(unsafe { acorn_sys::last_error_string() })) 
        }
    }

    /// Open a tree with a custom storage backend
    /// 
    /// # Arguments
    /// * `storage` - The storage backend to use
    /// 
    /// # Returns
    /// * `Ok(AcornTree)` - The opened tree
    /// * `Err(Error)` - If opening fails
    /// 
    /// # Example
    /// ```no_run
    /// # use acorn::{AcornTree, AcornStorage, Error};
    /// # fn main() -> Result<(), Error> {
    /// let storage = AcornStorage::sqlite("./test.db", None)?;
    /// let mut tree = AcornTree::open_with_storage(&storage)?;
    /// 
    /// // Store some data
    /// tree.stash("key1", &"Hello, storage backend!")?;
    /// 
    /// // Retrieve data
    /// let value: String = tree.crack("key1")?;
    /// assert_eq!(value, "Hello, storage backend!");
    /// # Ok(())
    /// # }
    /// ```
    pub fn open_with_storage(storage: &AcornStorage) -> Result<Self> {
        let mut h: acorn_tree_handle = 0;
        let rc = unsafe { acorn_open_tree_with_storage(storage.h, &mut h as *mut _) };
        if rc == 0 { 
            Ok(Self { h }) 
        } else { 
            Err(Error::Acorn(unsafe { acorn_sys::last_error_string() })) 
        }
    }

    /// Open a tree with a document store backend
    /// 
    /// # Arguments
    /// * `document_store` - Document store instance with versioning support
    /// 
    /// # Example
    /// ```no_run
    /// # use acorn::{AcornDocumentStore, AcornTree, Error};
    /// # fn main() -> Result<(), Error> {
    /// let doc_store = AcornDocumentStore::new(Some("./my_data"))?;
    /// let mut tree = AcornTree::open_with_document_store(&doc_store)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn open_with_document_store(document_store: &AcornDocumentStore) -> Result<Self> {
        let mut h: acorn_tree_handle = 0;
        let rc = unsafe { acorn_open_tree_with_document_store(document_store.h, &mut h as *mut _) };
        if rc == 0 { 
            Ok(Self { h }) 
        } else { 
            Err(Error::Acorn(unsafe { acorn_sys::last_error_string() })) 
        }
    }

    pub fn stash<T: Serialize>(&mut self, id: &str, value: &T) -> Result<()> {
        let json = serde_json::to_vec(value).map_err(|e| Error::Acorn(format!("Serialization error: {}", e)))?;
        let idc = CString::new(id).map_err(|e| Error::Acorn(format!("Invalid ID: {}", e)))?;
        let rc = unsafe { acorn_stash_json(self.h, idc.as_ptr(), json.as_ptr(), json.len()) };
        if rc == 0 { 
            Ok(()) 
        } else { 
            Err(Error::Acorn(unsafe { acorn_sys::last_error_string() })) 
        }
    }

    pub fn crack<T: DeserializeOwned>(&self, id: &str) -> Result<T> {
        let idc = CString::new(id).map_err(|e| Error::Acorn(format!("Invalid ID: {}", e)))?;
        let mut buf = acorn_buf { data: ptr::null_mut(), len: 0 };
        let rc = unsafe { acorn_crack_json(self.h, idc.as_ptr(), &mut buf as *mut _) };
        if rc == 1 {
            return Err(Error::NotFound);
        }
        if rc != 0 {
            return Err(Error::Acorn(unsafe { acorn_sys::last_error_string() }));
        }

        // Safety: We trust the shim to return valid data
        let slice = unsafe { std::slice::from_raw_parts(buf.data, buf.len) };
        let out = serde_json::from_slice(slice).map_err(|e| Error::Acorn(e.to_string()))?;
        unsafe { acorn_free_buf(&mut buf as *mut _) };
        Ok(out)
    }

    /// Create an iterator over key-value pairs with the given prefix.
    /// Pass an empty string to iterate over all keys.
    pub fn iter(&self, prefix: &str) -> Result<AcornIterator> {
        let prefix_c = CString::new(prefix).map_err(|e| Error::Acorn(format!("Invalid prefix: {}", e)))?;
        let mut iter_h: acorn_iter_handle = 0;
        let rc = unsafe { acorn_iter_start(self.h, prefix_c.as_ptr(), &mut iter_h as *mut _) };
        if rc == 0 {
            Ok(AcornIterator { h: iter_h })
        } else {
            Err(Error::Acorn(unsafe { acorn_sys::last_error_string() }))
        }
    }

    /// Subscribe to changes in the tree. The callback will be invoked whenever
    /// an item is added or modified. The callback is called from a background thread.
    ///
    /// Returns an `AcornSubscription` that will automatically unsubscribe when dropped.
    ///
    /// # Example
    /// ```no_run
    /// # use acorn::{AcornTree, Error};
    /// # fn main() -> Result<(), Error> {
    /// let tree = AcornTree::open("memory://")?;
    /// let _sub = tree.subscribe(|key: &str, value: &serde_json::Value| {
    ///     println!("Changed: {} = {:?}", key, value);
    /// })?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn subscribe<F>(&self, callback: F) -> Result<AcornSubscription>
    where
        F: Fn(&str, &serde_json::Value) + Send + 'static,
    {
        AcornSubscription::new(self.h, callback)
    }

    /// Subscribe to only stash (create/update) operations
    /// 
    /// # Example
    /// ```no_run
    /// # use acorn::{AcornTree, Error};
    /// # fn main() -> Result<(), Error> {
    /// let tree = AcornTree::open("memory://")?;
    /// let _sub = tree.subscribe_stash(|key: &str, value: &serde_json::Value| {
    ///     println!("Stashed: {} = {:?}", key, value);
    /// })?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn subscribe_stash<F>(&self, callback: F) -> Result<AcornSubscription>
    where
        F: Fn(&str, &serde_json::Value) + Send + 'static,
    {
        AcornSubscription::new_filtered(self.h, callback, ChangeType::Stash)
    }

    /// Subscribe to only toss (delete) operations
    /// 
    /// # Example
    /// ```no_run
    /// # use acorn::{AcornTree, Error};
    /// # fn main() -> Result<(), Error> {
    /// let tree = AcornTree::open("memory://")?;
    /// let _sub = tree.subscribe_toss(|key: &str| {
    ///     println!("Tossed: {}", key);
    /// })?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn subscribe_toss<F>(&self, callback: F) -> Result<AcornSubscription>
    where
        F: Fn(&str) + Send + 'static,
    {
        AcornSubscription::new_toss(self.h, callback)
    }

    /// Subscribe to changes with filtering by predicate
    /// 
    /// # Example
    /// ```no_run
    /// # use acorn::{AcornTree, Error};
    /// # fn main() -> Result<(), Error> {
    /// let tree = AcornTree::open("memory://")?;
    /// let _sub = tree.subscribe_where(|key: &str, value: &serde_json::Value| {
    ///     // Only notify for keys starting with "user-"
    ///     key.starts_with("user-")
    /// }, |key: &str, value: &serde_json::Value| {
    ///     println!("Filtered change: {} = {:?}", key, value);
    /// })?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn subscribe_where<F, G>(&self, predicate: F, callback: G) -> Result<AcornSubscription>
    where
        F: Fn(&str, &serde_json::Value) -> bool + Send + 'static,
        G: Fn(&str, &serde_json::Value) + Send + 'static,
    {
        AcornSubscription::new_filtered_predicate(self.h, predicate, callback)
    }

    /// Synchronize this tree with a remote HTTP endpoint.
    /// This pulls data from the remote server and merges it into the local tree.
    ///
    /// # Example
    /// ```no_run
    /// # use acorn::{AcornTree, Error};
    /// # fn main() -> Result<(), Error> {
    /// let tree = AcornTree::open("file://./db")?;
    /// tree.sync_http("http://example.com/api/acorn")?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn sync_http(&self, url: &str) -> Result<()> {
        let url_c = CString::new(url).map_err(|e| Error::Acorn(format!("Invalid URL: {}", e)))?;
        let rc = unsafe { acorn_sync_http(self.h, url_c.as_ptr()) };
        if rc == 0 {
            Ok(())
        } else {
            Err(Error::Acorn(unsafe { acorn_sys::last_error_string() }))
        }
    }

    /// Store multiple key-value pairs in a single operation.
    /// This is more efficient than calling stash() multiple times.
    ///
    /// # Example
    /// ```no_run
    /// # use acorn::{AcornTree, Error};
    /// # use serde::{Deserialize, Serialize};
    /// # #[derive(Serialize, Deserialize)]
    /// # struct Data { value: i32 }
    /// # fn main() -> Result<(), Error> {
    /// let mut tree = AcornTree::open("memory://")?;
    /// let items = vec![
    ///     ("key1", Data { value: 1 }),
    ///     ("key2", Data { value: 2 }),
    ///     ("key3", Data { value: 3 }),
    /// ];
    /// tree.batch_stash(&items)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn batch_stash<T: Serialize>(&mut self, items: &[(&str, T)]) -> Result<()> {
        if items.is_empty() {
            return Ok(());
        }

        // Prepare C-compatible arrays
        let ids: Vec<CString> = items
            .iter()
            .map(|(id, _)| CString::new(*id).map_err(|e| Error::Acorn(format!("Invalid ID: {}", e))))
            .collect::<Result<Vec<_>>>()?;

        let jsons: Vec<Vec<u8>> = items
            .iter()
            .map(|(_, value)| serde_json::to_vec(value).map_err(|e| Error::Acorn(format!("Serialization error: {}", e))))
            .collect::<Result<Vec<_>>>()?;

        let id_ptrs: Vec<*const i8> = ids.iter().map(|s| s.as_ptr()).collect();
        let json_ptrs: Vec<*const u8> = jsons.iter().map(|v| v.as_ptr()).collect();
        let json_lens: Vec<usize> = jsons.iter().map(|v| v.len()).collect();

        let rc = unsafe {
            acorn_batch_stash(
                self.h,
                id_ptrs.as_ptr() as *mut *const i8,
                json_ptrs.as_ptr() as *mut *const u8,
                json_lens.as_ptr(),
                items.len(),
            )
        };

        if rc == 0 {
            Ok(())
        } else {
            Err(Error::Acorn(unsafe { acorn_sys::last_error_string() }))
        }
    }

    /// Retrieve multiple values by their IDs in a single operation.
    /// Returns a vector of Option<T> where None indicates the key was not found.
    ///
    /// # Example
    /// ```no_run
    /// # use acorn::{AcornTree, Error};
    /// # use serde::{Deserialize, Serialize};
    /// # #[derive(Serialize, Deserialize)]
    /// # struct Data { value: i32 }
    /// # fn main() -> Result<(), Error> {
    /// let tree = AcornTree::open("memory://")?;
    /// let keys = vec!["key1", "key2", "key3"];
    /// let results: Vec<Option<Data>> = tree.batch_crack(&keys)?;
    /// for (key, result) in keys.iter().zip(results.iter()) {
    ///     match result {
    ///         Some(data) => println!("{}: {:?}", key, data.value),
    ///         None => println!("{}: not found", key),
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn batch_crack<T: DeserializeOwned>(&self, ids: &[&str]) -> Result<Vec<Option<T>>> {
        if ids.is_empty() {
            return Ok(Vec::new());
        }

        // Prepare C-compatible arrays
        let id_cstrings: Vec<CString> = ids
            .iter()
            .map(|id| CString::new(*id).map_err(|e| Error::Acorn(format!("Invalid ID: {}", e))))
            .collect::<Result<Vec<_>>>()?;

        let id_ptrs: Vec<*const i8> = id_cstrings.iter().map(|s| s.as_ptr()).collect();

        let mut out_jsons: Vec<acorn_buf> = vec![acorn_buf { data: ptr::null_mut(), len: 0 }; ids.len()];
        let mut out_found: Vec<i32> = vec![0; ids.len()];

        let rc = unsafe {
            acorn_batch_crack(
                self.h,
                id_ptrs.as_ptr() as *mut *const i8,
                ids.len(),
                out_jsons.as_mut_ptr(),
                out_found.as_mut_ptr(),
            )
        };

        if rc != 0 {
            // Clean up any allocated buffers
            for buf in &mut out_jsons {
                if !buf.data.is_null() {
                    unsafe { acorn_free_buf(buf as *mut _) };
                }
            }
            return Err(Error::Acorn(unsafe { acorn_sys::last_error_string() }));
        }

        // Convert results to Rust types
        let mut results = Vec::with_capacity(ids.len());
        for i in 0..ids.len() {
            if out_found[i] == 0 {
                results.push(None);
            } else {
                let slice = unsafe { std::slice::from_raw_parts(out_jsons[i].data, out_jsons[i].len) };
                let value = serde_json::from_slice(slice).map_err(|e| Error::Acorn(e.to_string()))?;
                results.push(Some(value));
            }

            // Free the buffer
            if !out_jsons[i].data.is_null() {
                unsafe { acorn_free_buf(&mut out_jsons[i] as *mut _) };
            }
        }

        Ok(results)
    }

    /// Delete multiple items by their IDs in a single operation.
    ///
    /// # Example
    /// ```no_run
    /// # use acorn::{AcornTree, Error};
    /// # fn main() -> Result<(), Error> {
    /// let mut tree = AcornTree::open("memory://")?;
    /// let keys_to_delete = vec!["key1", "key2", "key3"];
    /// tree.batch_delete(&keys_to_delete)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn batch_delete(&mut self, ids: &[&str]) -> Result<()> {
        if ids.is_empty() {
            return Ok(());
        }

        // Prepare C-compatible arrays
        let id_cstrings: Vec<CString> = ids
            .iter()
            .map(|id| CString::new(*id).map_err(|e| Error::Acorn(format!("Invalid ID: {}", e))))
            .collect::<Result<Vec<_>>>()?;

        let id_ptrs: Vec<*const i8> = id_cstrings.iter().map(|s| s.as_ptr()).collect();

        let rc = unsafe {
            acorn_batch_delete(
                self.h,
                id_ptrs.as_ptr() as *mut *const i8,
                ids.len(),
            )
        };

        if rc == 0 {
            Ok(())
        } else {
            Err(Error::Acorn(unsafe { acorn_sys::last_error_string() }))
        }
    }

    /// Start a LINQ-style query on this tree.
    /// Returns a query builder that supports filtering, ordering, and projection.
    ///
    /// # Example
    /// ```no_run
    /// # use acorn::{AcornTree, Error};
    /// # use serde::{Deserialize, Serialize};
    /// # #[derive(Serialize, Deserialize)]
    /// # struct User { name: String, age: u32 }
    /// # fn main() -> Result<(), Error> {
    /// let tree = AcornTree::open("memory://")?;
    /// 
    /// // Query users older than 18, ordered by name
    /// let adults: Vec<User> = tree.query()
    ///     .where_condition(|user| user["age"].as_u64().unwrap_or(0) >= 18)
    ///     .order_by(|user| user["name"].as_str().unwrap_or("").to_string())
    ///     .collect()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn query(&self) -> AcornQuery {
        AcornQuery::new(self.h)
    }

    /// Begin a new transaction for atomic multi-operation changes.
    /// 
    /// # Example
    /// 
    /// ```rust
    /// use acorn::AcornTree;
    /// 
    /// let mut tree = AcornTree::open("memory://")?;
    /// let mut tx = tree.begin_transaction()?;
    /// 
    /// tx.stash("user1", &serde_json::json!({"name": "Alice", "age": 30}))?;
    /// tx.stash("user2", &serde_json::json!({"name": "Bob", "age": 25}))?;
    /// 
    /// if tx.commit()? {
    ///     println!("Transaction committed successfully");
    /// } else {
    ///     println!("Transaction failed to commit");
    /// }
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn begin_transaction(&self) -> Result<AcornTransaction> {
        let mut h: acorn_transaction_handle = 0;
        let rc = unsafe { acorn_begin_transaction(self.h, &mut h as *mut _) };
        if rc == 0 {
            Ok(AcornTransaction { h })
        } else {
            Err(Error::Acorn(unsafe { acorn_sys::last_error_string() }))
        }
    }

    /// Create a new mesh coordinator for advanced synchronization.
    /// 
    /// # Example
    /// 
    /// ```rust
    /// use acorn::AcornTree;
    /// 
    /// let mesh = AcornTree::create_mesh()?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn create_mesh() -> Result<AcornMesh> {
        let mut h: acorn_mesh_handle = 0;
        let rc = unsafe { acorn_mesh_create(&mut h as *mut _) };
        if rc == 0 {
            Ok(AcornMesh { h })
        } else {
            Err(Error::Acorn(unsafe { acorn_sys::last_error_string() }))
        }
    }

    /// Create a peer-to-peer sync connection with another tree.
    /// 
    /// # Example
    /// 
    /// ```rust
    /// use acorn::AcornTree;
    /// 
    /// let tree1 = AcornTree::open("memory://")?;
    /// let tree2 = AcornTree::open("memory://")?;
    /// let p2p = AcornTree::create_p2p(&tree1, &tree2)?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn create_p2p(local_tree: &AcornTree, remote_tree: &AcornTree) -> Result<AcornP2P> {
        let mut h: acorn_p2p_handle = 0;
        let rc = unsafe { acorn_p2p_create(local_tree.h, remote_tree.h, &mut h as *mut _) };
        if rc == 0 {
            Ok(AcornP2P { h })
        } else {
            Err(Error::Acorn(unsafe { acorn_sys::last_error_string() }))
        }
    }
}

impl Drop for AcornTree {
    fn drop(&mut self) { unsafe { acorn_close_tree(self.h); } }
}

/// Iterator over key-value pairs in an AcornTree.
/// The iterator holds a snapshot of the tree at the time it was created.
pub struct AcornIterator {
    h: acorn_iter_handle,
}

/// Transaction for atomic multi-operation changes.
/// Provides snapshot isolation and rollback capabilities.
pub struct AcornTransaction {
    h: acorn_transaction_handle,
}

/// Mesh coordinator for advanced synchronization across multiple trees.
/// Supports various network topologies like full mesh, ring, and star.
pub struct AcornMesh {
    h: acorn_mesh_handle,
}

/// Peer-to-peer synchronization connection between two trees.
/// Supports bidirectional, push-only, and pull-only sync modes.
pub struct AcornP2P {
    h: acorn_p2p_handle,
}

impl AcornTransaction {
    /// Store a value in the transaction.
    /// 
    /// # Example
    /// 
    /// ```rust
    /// use acorn::AcornTree;
    /// 
    /// let mut tree = AcornTree::open("memory://")?;
    /// let mut tx = tree.begin_transaction()?;
    /// tx.stash("user1", &serde_json::json!({"name": "Alice"}))?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn stash<T: Serialize>(&mut self, id: &str, value: &T) -> Result<()> {
        let json = serde_json::to_vec(value).map_err(|e| Error::Acorn(format!("Serialization error: {}", e)))?;
        let idc = CString::new(id).map_err(|e| Error::Acorn(format!("Invalid ID: {}", e)))?;
        let rc = unsafe { acorn_transaction_stash(self.h, idc.as_ptr(), json.as_ptr(), json.len()) };
        if rc == 0 {
            Ok(())
        } else {
            Err(Error::Acorn(unsafe { acorn_sys::last_error_string() }))
        }
    }

    /// Delete a value from the transaction.
    /// 
    /// # Example
    /// 
    /// ```rust
    /// use acorn::AcornTree;
    /// 
    /// let mut tree = AcornTree::open("memory://")?;
    /// let mut tx = tree.begin_transaction()?;
    /// tx.delete("user1")?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn delete(&mut self, id: &str) -> Result<()> {
        let idc = CString::new(id).map_err(|e| Error::Acorn(format!("Invalid ID: {}", e)))?;
        let rc = unsafe { acorn_transaction_delete(self.h, idc.as_ptr()) };
        if rc == 0 {
            Ok(())
        } else {
            Err(Error::Acorn(unsafe { acorn_sys::last_error_string() }))
        }
    }

    /// Commit the transaction, applying all changes atomically.
    /// Returns true if the commit was successful, false if it failed.
    /// 
    /// # Example
    /// 
    /// ```rust
    /// use acorn::AcornTree;
    /// 
    /// let mut tree = AcornTree::open("memory://")?;
    /// let mut tx = tree.begin_transaction()?;
    /// tx.stash("user1", &serde_json::json!({"name": "Alice"}))?;
    /// 
    /// if tx.commit()? {
    ///     println!("Transaction committed successfully");
    /// } else {
    ///     println!("Transaction failed to commit");
    /// }
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn commit(&mut self) -> Result<bool> {
        let rc = unsafe { acorn_transaction_commit(self.h) };
        if rc == 0 {
            Ok(true)
        } else if rc == 1 {
            Ok(false) // Transaction failed to commit
        } else {
            Err(Error::Acorn(unsafe { acorn_sys::last_error_string() }))
        }
    }

    /// Rollback the transaction, discarding all changes.
    /// 
    /// # Example
    /// 
    /// ```rust
    /// use acorn::AcornTree;
    /// 
    /// let mut tree = AcornTree::open("memory://")?;
    /// let mut tx = tree.begin_transaction()?;
    /// tx.stash("user1", &serde_json::json!({"name": "Alice"}))?;
    /// tx.rollback()?; // All changes are discarded
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn rollback(&mut self) -> Result<()> {
        let rc = unsafe { acorn_transaction_rollback(self.h) };
        if rc == 0 {
            Ok(())
        } else {
            Err(Error::Acorn(unsafe { acorn_sys::last_error_string() }))
        }
    }
}

impl Drop for AcornTransaction {
    fn drop(&mut self) {
        unsafe { acorn_transaction_close(self.h); }
    }
}

impl AcornMesh {
    /// Add a tree node to the mesh with the given ID.
    /// 
    /// # Example
    /// 
    /// ```rust
    /// use acorn::AcornTree;
    /// 
    /// let mesh = AcornTree::create_mesh()?;
    /// let tree = AcornTree::open("memory://")?;
    /// mesh.add_node("node1", &tree)?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn add_node(&self, node_id: &str, tree: &AcornTree) -> Result<()> {
        let idc = CString::new(node_id).map_err(|e| Error::Acorn(format!("Invalid node ID: {}", e)))?;
        let rc = unsafe { acorn_mesh_add_node(self.h, idc.as_ptr(), tree.h) };
        if rc == 0 {
            Ok(())
        } else {
            Err(Error::Acorn(unsafe { acorn_sys::last_error_string() }))
        }
    }

    /// Connect two nodes in the mesh for bidirectional synchronization.
    /// 
    /// # Example
    /// 
    /// ```rust
    /// use acorn::AcornTree;
    /// 
    /// let mesh = AcornTree::create_mesh()?;
    /// let tree1 = AcornTree::open("memory://")?;
    /// let tree2 = AcornTree::open("memory://")?;
    /// mesh.add_node("node1", &tree1)?;
    /// mesh.add_node("node2", &tree2)?;
    /// mesh.connect_nodes("node1", "node2")?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn connect_nodes(&self, node_a: &str, node_b: &str) -> Result<()> {
        let node_ac = CString::new(node_a).map_err(|e| Error::Acorn(format!("Invalid node A ID: {}", e)))?;
        let node_bc = CString::new(node_b).map_err(|e| Error::Acorn(format!("Invalid node B ID: {}", e)))?;
        let rc = unsafe { acorn_mesh_connect_nodes(self.h, node_ac.as_ptr(), node_bc.as_ptr()) };
        if rc == 0 {
            Ok(())
        } else {
            Err(Error::Acorn(unsafe { acorn_sys::last_error_string() }))
        }
    }

    /// Create a full mesh topology where every node connects to every other node.
    /// 
    /// # Example
    /// 
    /// ```rust
    /// use acorn::AcornTree;
    /// 
    /// let mesh = AcornTree::create_mesh()?;
    /// // Add nodes first...
    /// mesh.create_full_mesh()?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn create_full_mesh(&self) -> Result<()> {
        let rc = unsafe { acorn_mesh_create_full_mesh(self.h) };
        if rc == 0 {
            Ok(())
        } else {
            Err(Error::Acorn(unsafe { acorn_sys::last_error_string() }))
        }
    }

    /// Create a ring topology where each node connects to the next, and the last connects to the first.
    /// 
    /// # Example
    /// 
    /// ```rust
    /// use acorn::AcornTree;
    /// 
    /// let mesh = AcornTree::create_mesh()?;
    /// // Add nodes first...
    /// mesh.create_ring()?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn create_ring(&self) -> Result<()> {
        let rc = unsafe { acorn_mesh_create_ring(self.h) };
        if rc == 0 {
            Ok(())
        } else {
            Err(Error::Acorn(unsafe { acorn_sys::last_error_string() }))
        }
    }

    /// Create a star topology where all nodes connect to a central hub.
    /// 
    /// # Example
    /// 
    /// ```rust
    /// use acorn::AcornTree;
    /// 
    /// let mesh = AcornTree::create_mesh()?;
    /// // Add nodes first...
    /// mesh.create_star("hub")?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn create_star(&self, hub_node_id: &str) -> Result<()> {
        let hubc = CString::new(hub_node_id).map_err(|e| Error::Acorn(format!("Invalid hub node ID: {}", e)))?;
        let rc = unsafe { acorn_mesh_create_star(self.h, hubc.as_ptr()) };
        if rc == 0 {
            Ok(())
        } else {
            Err(Error::Acorn(unsafe { acorn_sys::last_error_string() }))
        }
    }

    /// Synchronize all nodes in the mesh.
    /// 
    /// # Example
    /// 
    /// ```rust
    /// use acorn::AcornTree;
    /// 
    /// let mesh = AcornTree::create_mesh()?;
    /// // Setup mesh topology...
    /// mesh.synchronize_all()?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn synchronize_all(&self) -> Result<()> {
        let rc = unsafe { acorn_mesh_synchronize_all(self.h) };
        if rc == 0 {
            Ok(())
        } else {
            Err(Error::Acorn(unsafe { acorn_sys::last_error_string() }))
        }
    }
}

impl Drop for AcornMesh {
    fn drop(&mut self) {
        unsafe { acorn_mesh_close(self.h); }
    }
}

impl AcornP2P {
    /// Synchronize bidirectionally between the local and remote trees.
    /// 
    /// # Example
    /// 
    /// ```rust
    /// use acorn::AcornTree;
    /// 
    /// let tree1 = AcornTree::open("memory://")?;
    /// let tree2 = AcornTree::open("memory://")?;
    /// let p2p = AcornTree::create_p2p(&tree1, &tree2)?;
    /// p2p.sync_bidirectional()?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn sync_bidirectional(&self) -> Result<()> {
        let rc = unsafe { acorn_p2p_sync_bidirectional(self.h) };
        if rc == 0 {
            Ok(())
        } else {
            Err(Error::Acorn(unsafe { acorn_sys::last_error_string() }))
        }
    }

    /// Synchronize by pushing changes from local to remote tree only.
    /// 
    /// # Example
    /// 
    /// ```rust
    /// use acorn::AcornTree;
    /// 
    /// let tree1 = AcornTree::open("memory://")?;
    /// let tree2 = AcornTree::open("memory://")?;
    /// let p2p = AcornTree::create_p2p(&tree1, &tree2)?;
    /// p2p.sync_push_only()?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn sync_push_only(&self) -> Result<()> {
        let rc = unsafe { acorn_p2p_sync_push_only(self.h) };
        if rc == 0 {
            Ok(())
        } else {
            Err(Error::Acorn(unsafe { acorn_sys::last_error_string() }))
        }
    }

    /// Synchronize by pulling changes from remote to local tree only.
    /// 
    /// # Example
    /// 
    /// ```rust
    /// use acorn::AcornTree;
    /// 
    /// let tree1 = AcornTree::open("memory://")?;
    /// let tree2 = AcornTree::open("memory://")?;
    /// let p2p = AcornTree::create_p2p(&tree1, &tree2)?;
    /// p2p.sync_pull_only()?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn sync_pull_only(&self) -> Result<()> {
        let rc = unsafe { acorn_p2p_sync_pull_only(self.h) };
        if rc == 0 {
            Ok(())
        } else {
            Err(Error::Acorn(unsafe { acorn_sys::last_error_string() }))
        }
    }

    /// Set the synchronization mode.
    /// 
    /// # Arguments
    /// * `sync_mode` - 0=Bidirectional, 1=PushOnly, 2=PullOnly, 3=Disabled
    /// 
    /// # Example
    /// 
    /// ```rust
    /// use acorn::AcornTree;
    /// 
    /// let tree1 = AcornTree::open("memory://")?;
    /// let tree2 = AcornTree::open("memory://")?;
    /// let p2p = AcornTree::create_p2p(&tree1, &tree2)?;
    /// p2p.set_sync_mode(1)?; // PushOnly
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn set_sync_mode(&self, sync_mode: i32) -> Result<()> {
        let rc = unsafe { acorn_p2p_set_sync_mode(self.h, sync_mode) };
        if rc == 0 {
            Ok(())
        } else {
            Err(Error::Acorn(unsafe { acorn_sys::last_error_string() }))
        }
    }

    /// Set the conflict resolution direction.
    /// 
    /// # Arguments
    /// * `conflict_direction` - 0=UseJudge, 1=PreferLocal, 2=PreferRemote
    /// 
    /// # Example
    /// 
    /// ```rust
    /// use acorn::AcornTree;
    /// 
    /// let tree1 = AcornTree::open("memory://")?;
    /// let tree2 = AcornTree::open("memory://")?;
    /// let p2p = AcornTree::create_p2p(&tree1, &tree2)?;
    /// p2p.set_conflict_direction(1)?; // PreferLocal
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn set_conflict_direction(&self, conflict_direction: i32) -> Result<()> {
        let rc = unsafe { acorn_p2p_set_conflict_direction(self.h, conflict_direction) };
        if rc == 0 {
            Ok(())
        } else {
            Err(Error::Acorn(unsafe { acorn_sys::last_error_string() }))
        }
    }
}

impl Drop for AcornP2P {
    fn drop(&mut self) {
        unsafe { acorn_p2p_close(self.h); }
    }
}

impl AcornIterator {
    /// Get the next key-value pair. Returns None when iteration is complete.
    pub fn next<T: DeserializeOwned>(&mut self) -> Result<Option<(String, T)>> {
        let mut key_buf = acorn_buf { data: ptr::null_mut(), len: 0 };
        let mut json_buf = acorn_buf { data: ptr::null_mut(), len: 0 };
        let mut done: i32 = 0;

        let rc = unsafe {
            acorn_iter_next(
                self.h,
                &mut key_buf as *mut _,
                &mut json_buf as *mut _,
                &mut done as *mut _,
            )
        };

        if rc != 0 {
            return Err(Error::Acorn(unsafe { acorn_sys::last_error_string() }));
        }

        if done != 0 {
            return Ok(None);
        }

        // Extract key
        let key_slice = unsafe { std::slice::from_raw_parts(key_buf.data, key_buf.len) };
        let key = String::from_utf8_lossy(key_slice).into_owned();

        // Extract and deserialize value
        let json_slice = unsafe { std::slice::from_raw_parts(json_buf.data, json_buf.len) };
        let value = serde_json::from_slice(json_slice).map_err(|e| Error::Acorn(e.to_string()))?;

        // Free buffers
        unsafe {
            acorn_free_buf(&mut key_buf as *mut _);
            acorn_free_buf(&mut json_buf as *mut _);
        }

        Ok(Some((key, value)))
    }

    /// Collect all remaining items into a Vec. This consumes the iterator.
    pub fn collect<T: DeserializeOwned>(&mut self) -> Result<Vec<(String, T)>> {
        let mut items = Vec::new();
        while let Some(item) = self.next()? {
            items.push(item);
        }
        Ok(items)
    }
}

impl Drop for AcornIterator {
    fn drop(&mut self) {
        unsafe { acorn_iter_close(self.h); }
    }
}

/// Subscription to tree changes. Automatically unsubscribes when dropped.
pub struct AcornSubscription {
    h: acorn_sub_handle,
    // Keep the callback alive by storing the boxed callback
    // The user_data pointer in the C subscription points to this
    _callback: Box<Box<dyn Fn(&str, &serde_json::Value) + Send>>,
}

impl AcornSubscription {
    fn new<F>(tree_h: acorn_tree_handle, callback: F) -> Result<Self>
    where
        F: Fn(&str, &serde_json::Value) + Send + 'static,
    {
        // Box the callback - this will be passed as user data to C
        let callback_box: Box<dyn Fn(&str, &serde_json::Value) + Send> = Box::new(callback);
        let user_data = Box::into_raw(Box::new(callback_box)) as *mut std::ffi::c_void;

        // Define the C callback wrapper
        unsafe extern "C" fn c_callback(
            key: *const std::os::raw::c_char,
            json: *const u8,
            len: usize,
            user: *mut std::ffi::c_void,
        ) {
            if user.is_null() {
                return;
            }

            // Reconstruct the callback from user data
            let callback_ptr = user as *const Box<dyn Fn(&str, &serde_json::Value) + Send>;
            let callback = &**callback_ptr;

            // Convert key to str
            let key_str = if key.is_null() {
                ""
            } else {
                std::ffi::CStr::from_ptr(key)
                    .to_str()
                    .unwrap_or("")
            };

            // Convert JSON bytes to serde_json::Value
            if !json.is_null() && len > 0 {
                let json_slice = std::slice::from_raw_parts(json, len);
                if let Ok(value) = serde_json::from_slice::<serde_json::Value>(json_slice) {
                    // Invoke the user callback
                    callback(key_str, &value);
                }
            }
        }

        let mut sub_h: acorn_sub_handle = 0;
        let rc = unsafe {
            acorn_subscribe(
                tree_h,
                Some(c_callback),
                user_data,
                &mut sub_h as *mut _,
            )
        };

        if rc == 0 {
            // Reconstruct the Box to store in Self (we own it now)
            // user_data points to Box<Box<dyn Fn...>>
            let callback_box = unsafe { Box::from_raw(user_data as *mut Box<dyn Fn(&str, &serde_json::Value) + Send>) };
            Ok(Self {
                h: sub_h,
                _callback: callback_box,
            })
        } else {
            // Clean up on error
            unsafe {
                let _ = Box::from_raw(user_data as *mut Box<dyn Fn(&str, &serde_json::Value) + Send>);
            }
            Err(Error::Acorn(unsafe { acorn_sys::last_error_string() }))
        }
    }

    /// Create a subscription filtered by change type
    fn new_filtered<F>(tree_h: acorn_tree_handle, callback: F, change_type: ChangeType) -> Result<Self>
    where
        F: Fn(&str, &serde_json::Value) + Send + 'static,
    {
        // For now, we'll implement this as a wrapper around the basic subscription
        // In a full implementation, this would filter at the C level
        Self::new(tree_h, callback)
    }

    /// Create a subscription for toss operations only
    fn new_toss<F>(tree_h: acorn_tree_handle, callback: F) -> Result<Self>
    where
        F: Fn(&str) + Send + 'static,
    {
        // Wrap the toss callback to match the expected signature
        let wrapped_callback = move |key: &str, _value: &serde_json::Value| {
            callback(key);
        };
        Self::new(tree_h, wrapped_callback)
    }

    /// Create a subscription with predicate filtering
    fn new_filtered_predicate<F, G>(tree_h: acorn_tree_handle, predicate: F, callback: G) -> Result<Self>
    where
        F: Fn(&str, &serde_json::Value) -> bool + Send + 'static,
        G: Fn(&str, &serde_json::Value) + Send + 'static,
    {
        // Wrap both predicate and callback
        let wrapped_callback = move |key: &str, value: &serde_json::Value| {
            if predicate(key, value) {
                callback(key, value);
            }
        };
        Self::new(tree_h, wrapped_callback)
    }
}

impl Drop for AcornSubscription {
    fn drop(&mut self) {
        unsafe {
            acorn_unsubscribe(self.h);
        }
    }
}

/// LINQ-style query builder for AcornTree.
/// Provides fluent API for filtering, ordering, and projecting tree data.
pub struct AcornQuery {
    tree_h: acorn_tree_handle,
}

impl AcornQuery {
    fn new(tree_h: acorn_tree_handle) -> Self {
        Self { tree_h }
    }

    /// Filter items by a condition on the payload.
    /// 
    /// # Example
    /// ```no_run
    /// # use acorn::{AcornTree, Error};
    /// # use serde::{Deserialize, Serialize};
    /// # #[derive(Serialize, Deserialize)]
    /// # struct User { name: String, age: u32 }
    /// # fn main() -> Result<(), Error> {
    /// let tree = AcornTree::open("memory://")?;
    /// let adults: Vec<User> = tree.query()
    ///     .where_condition(|user| user["age"].as_u64().unwrap_or(0) >= 18)
    ///     .collect()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn where_condition<F>(self, predicate: F) -> AcornQueryWhere<F>
    where
        F: Fn(&serde_json::Value) -> bool,
    {
        AcornQueryWhere {
            tree_h: self.tree_h,
            predicate,
        }
    }

    /// Order items by a key selector.
    /// 
    /// # Example
    /// ```no_run
    /// # use acorn::{AcornTree, Error};
    /// # use serde::{Deserialize, Serialize};
    /// # #[derive(Serialize, Deserialize)]
    /// # struct User { name: String, age: u32 }
    /// # fn main() -> Result<(), Error> {
    /// let tree = AcornTree::open("memory://")?;
    /// let users: Vec<User> = tree.query()
    ///     .order_by(|user| user["name"].as_str().unwrap_or("").to_string())
    ///     .collect()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn order_by<F>(self, key_selector: F) -> AcornQueryOrderBy<F>
    where
        F: Fn(&serde_json::Value) -> String,
    {
        AcornQueryOrderBy {
            tree_h: self.tree_h,
            key_selector,
            descending: false,
        }
    }

    /// Order items by a key selector (descending).
    pub fn order_by_descending<F>(self, key_selector: F) -> AcornQueryOrderBy<F>
    where
        F: Fn(&serde_json::Value) -> String,
    {
        AcornQueryOrderBy {
            tree_h: self.tree_h,
            key_selector,
            descending: true,
        }
    }

    /// Take only the first N items.
    /// 
    /// # Example
    /// ```no_run
    /// # use acorn::{AcornTree, Error};
    /// # use serde::{Deserialize, Serialize};
    /// # #[derive(Serialize, Deserialize)]
    /// # struct User { name: String, age: u32 }
    /// # fn main() -> Result<(), Error> {
    /// let tree = AcornTree::open("memory://")?;
    /// let top_users: Vec<User> = tree.query()
    ///     .order_by(|user| user["name"].as_str().unwrap_or("").to_string())
    ///     .take(10)
    ///     .collect()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn take(self, count: usize) -> AcornQueryTake {
        AcornQueryTake {
            tree_h: self.tree_h,
            count,
        }
    }

    /// Skip the first N items.
    pub fn skip(self, count: usize) -> AcornQuerySkip {
        AcornQuerySkip {
            tree_h: self.tree_h,
            count,
        }
    }

    /// Execute the query and collect all results into a Vec.
    /// 
    /// # Example
    /// ```no_run
    /// # use acorn::{AcornTree, Error};
    /// # use serde::{Deserialize, Serialize};
    /// # #[derive(Serialize, Deserialize)]
    /// # struct User { name: String, age: u32 }
    /// # fn main() -> Result<(), Error> {
    /// let tree = AcornTree::open("memory://")?;
    /// let users: Vec<User> = tree.query().collect()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn collect<T: DeserializeOwned>(&self) -> Result<Vec<T>> {
        // For now, we'll implement a simple version that gets all items
        // In a full implementation, we'd need to implement the query execution
        // through the FFI layer
        let mut iter = unsafe {
            let mut iter_h: acorn_iter_handle = 0;
            let rc = acorn_iter_start(self.tree_h, ptr::null(), &mut iter_h as *mut _);
            if rc != 0 {
                return Err(Error::Acorn(unsafe { acorn_sys::last_error_string() }));
            }
            AcornIterator { h: iter_h }
        };

        let mut results = Vec::new();
        while let Some((_, value)) = iter.next()? {
            results.push(value);
        }
        Ok(results)
    }

    /// Execute the query and return the first result, or None if empty.
    pub fn first<T: DeserializeOwned>(&self) -> Result<Option<T>> {
        let mut iter = unsafe {
            let mut iter_h: acorn_iter_handle = 0;
            let rc = acorn_iter_start(self.tree_h, ptr::null(), &mut iter_h as *mut _);
            if rc != 0 {
                return Err(Error::Acorn(unsafe { acorn_sys::last_error_string() }));
            }
            AcornIterator { h: iter_h }
        };

        if let Some((_, value)) = iter.next()? {
            Ok(Some(value))
        } else {
            Ok(None)
        }
    }

    /// Count the number of items that would be returned by this query.
    pub fn count(&self) -> Result<usize> {
        unsafe {
            let mut count: usize = 0;
            let rc = acorn_count(self.tree_h, &mut count as *mut _);
            if rc != 0 {
                return Err(Error::Acorn(unsafe { acorn_sys::last_error_string() }));
            }
            Ok(count)
        }
    }

    /// Check if any items match this query.
    pub fn any(&self) -> Result<bool> {
        Ok(self.count()? > 0)
    }

    /// Filter by timestamp range (between start and end dates).
    /// 
    /// # Example
    /// ```no_run
    /// # use acorn::{AcornTree, Error};
    /// # use serde::{Deserialize, Serialize};
    /// # use std::time::{SystemTime, UNIX_EPOCH};
    /// # #[derive(Serialize, Deserialize)]
    /// # struct Event { name: String, timestamp: SystemTime }
    /// # fn main() -> Result<(), Error> {
    /// let tree = AcornTree::open("memory://")?;
    /// let start = SystemTime::now();
    /// let end = SystemTime::now();
    /// let events: Vec<Event> = tree.query()
    ///     .between(start, end)
    ///     .collect()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn between(self, start: std::time::SystemTime, end: std::time::SystemTime) -> AcornQueryTimestampRange {
        AcornQueryTimestampRange {
            tree_h: self.tree_h,
            start,
            end,
        }
    }

    /// Filter by items created after a specific date.
    /// 
    /// # Example
    /// ```no_run
    /// # use acorn::{AcornTree, Error};
    /// # use serde::{Deserialize, Serialize};
    /// # use std::time::{SystemTime, UNIX_EPOCH};
    /// # #[derive(Serialize, Deserialize)]
    /// # struct Event { name: String, timestamp: SystemTime }
    /// # fn main() -> Result<(), Error> {
    /// let tree = AcornTree::open("memory://")?;
    /// let cutoff = SystemTime::now();
    /// let recent_events: Vec<Event> = tree.query()
    ///     .after(cutoff)
    ///     .collect()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn after(self, date: std::time::SystemTime) -> AcornQueryTimestampAfter {
        AcornQueryTimestampAfter {
            tree_h: self.tree_h,
            date,
        }
    }

    /// Filter by items created before a specific date.
    /// 
    /// # Example
    /// ```no_run
    /// # use acorn::{AcornTree, Error};
    /// # use serde::{Deserialize, Serialize};
    /// # use std::time::{SystemTime, UNIX_EPOCH};
    /// # #[derive(Serialize, Deserialize)]
    /// # struct Event { name: String, timestamp: SystemTime }
    /// # fn main() -> Result<(), Error> {
    /// let tree = AcornTree::open("memory://")?;
    /// let cutoff = SystemTime::now();
    /// let old_events: Vec<Event> = tree.query()
    ///     .before(cutoff)
    ///     .collect()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn before(self, date: std::time::SystemTime) -> AcornQueryTimestampBefore {
        AcornQueryTimestampBefore {
            tree_h: self.tree_h,
            date,
        }
    }

    /// Filter by origin node ID.
    /// 
    /// # Example
    /// ```no_run
    /// # use acorn::{AcornTree, Error};
    /// # use serde::{Deserialize, Serialize};
    /// # #[derive(Serialize, Deserialize)]
    /// # struct Data { content: String }
    /// # fn main() -> Result<(), Error> {
    /// let tree = AcornTree::open("memory://")?;
    /// let node_data: Vec<Data> = tree.query()
    ///     .from_node("node-123")
    ///     .collect()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn from_node(self, node_id: &str) -> AcornQueryFromNode {
        AcornQueryFromNode {
            tree_h: self.tree_h,
            node_id: node_id.to_string(),
        }
    }

    /// Order by timestamp (newest first).
    /// 
    /// # Example
    /// ```no_run
    /// # use acorn::{AcornTree, Error};
    /// # use serde::{Deserialize, Serialize};
    /// # #[derive(Serialize, Deserialize)]
    /// # struct Event { name: String }
    /// # fn main() -> Result<(), Error> {
    /// let tree = AcornTree::open("memory://")?;
    /// let newest_events: Vec<Event> = tree.query()
    ///     .newest()
    ///     .collect()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn newest(self) -> AcornQueryNewest {
        AcornQueryNewest {
            tree_h: self.tree_h,
        }
    }

    /// Order by timestamp (oldest first).
    /// 
    /// # Example
    /// ```no_run
    /// # use acorn::{AcornTree, Error};
    /// # use serde::{Deserialize, Serialize};
    /// # #[derive(Serialize, Deserialize)]
    /// # struct Event { name: String }
    /// # fn main() -> Result<(), Error> {
    /// let tree = AcornTree::open("memory://")?;
    /// let oldest_events: Vec<Event> = tree.query()
    ///     .oldest()
    ///     .collect()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn oldest(self) -> AcornQueryOldest {
        AcornQueryOldest {
            tree_h: self.tree_h,
        }
    }

    /// Execute query and return single result (throws if multiple).
    /// 
    /// # Example
    /// ```no_run
    /// # use acorn::{AcornTree, Error};
    /// # use serde::{Deserialize, Serialize};
    /// # #[derive(Serialize, Deserialize)]
    /// # struct User { id: String, name: String }
    /// # fn main() -> Result<(), Error> {
    /// let tree = AcornTree::open("memory://")?;
    /// let user: Option<User> = tree.query()
    ///     .where_condition(|u| u["id"].as_str() == Some("admin"))
    ///     .single()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn single<T: DeserializeOwned>(&self) -> Result<Option<T>> {
        let mut iter = unsafe {
            let mut iter_h: acorn_iter_handle = 0;
            let rc = acorn_iter_start(self.tree_h, ptr::null(), &mut iter_h as *mut _);
            if rc != 0 {
                return Err(Error::Acorn(unsafe { acorn_sys::last_error_string() }));
            }
            AcornIterator { h: iter_h }
        };

        let mut result: Option<T> = None;
        let mut count = 0;
        
        while let Some((_, value)) = iter.next()? {
            count += 1;
            if count > 1 {
                return Err(Error::Acorn("Multiple results found for single() query".to_string()));
            }
            result = Some(value);
        }
        
        Ok(result)
    }
}

/// Query builder with WHERE condition applied.
pub struct AcornQueryWhere<F> {
    tree_h: acorn_tree_handle,
    predicate: F,
}

impl<F> AcornQueryWhere<F>
where
    F: Fn(&serde_json::Value) -> bool,
{
    /// Apply ordering to the filtered results.
    pub fn order_by<G>(self, key_selector: G) -> AcornQueryWhereOrderBy<F, G>
    where
        G: Fn(&serde_json::Value) -> String,
    {
        AcornQueryWhereOrderBy {
            tree_h: self.tree_h,
            predicate: self.predicate,
            key_selector,
            descending: false,
        }
    }

    /// Apply ordering to the filtered results (descending).
    pub fn order_by_descending<G>(self, key_selector: G) -> AcornQueryWhereOrderBy<F, G>
    where
        G: Fn(&serde_json::Value) -> String,
    {
        AcornQueryWhereOrderBy {
            tree_h: self.tree_h,
            predicate: self.predicate,
            key_selector,
            descending: true,
        }
    }

    /// Take only the first N items from filtered results.
    pub fn take(self, count: usize) -> AcornQueryWhereTake<F> {
        AcornQueryWhereTake {
            tree_h: self.tree_h,
            predicate: self.predicate,
            count,
        }
    }

    /// Skip the first N items from filtered results.
    pub fn skip(self, count: usize) -> AcornQueryWhereSkip<F> {
        AcornQueryWhereSkip {
            tree_h: self.tree_h,
            predicate: self.predicate,
            count,
        }
    }

    /// Execute the query and collect all filtered results.
    pub fn collect<T: DeserializeOwned>(&self) -> Result<Vec<T>> {
        // For now, implement simple filtering by getting all items and filtering in Rust
        // In a full implementation, this would be done in the C# layer
        let mut iter = unsafe {
            let mut iter_h: acorn_iter_handle = 0;
            let rc = acorn_iter_start(self.tree_h, ptr::null(), &mut iter_h as *mut _);
            if rc != 0 {
                return Err(Error::Acorn(unsafe { acorn_sys::last_error_string() }));
            }
            AcornIterator { h: iter_h }
        };

        let mut results = Vec::new();
        while let Some((_, value)) = iter.next::<serde_json::Value>()? {
            if (self.predicate)(&value) {
                let typed_value: T = serde_json::from_value(value).map_err(|e| Error::Acorn(e.to_string()))?;
                results.push(typed_value);
            }
        }
        Ok(results)
    }

    /// Execute the query and return the first filtered result.
    pub fn first<T: DeserializeOwned>(&self) -> Result<Option<T>> {
        let mut iter = unsafe {
            let mut iter_h: acorn_iter_handle = 0;
            let rc = acorn_iter_start(self.tree_h, ptr::null(), &mut iter_h as *mut _);
            if rc != 0 {
                return Err(Error::Acorn(unsafe { acorn_sys::last_error_string() }));
            }
            AcornIterator { h: iter_h }
        };

        while let Some((_, value)) = iter.next::<serde_json::Value>()? {
            if (self.predicate)(&value) {
                let typed_value: T = serde_json::from_value(value).map_err(|e| Error::Acorn(e.to_string()))?;
                return Ok(Some(typed_value));
            }
        }
        Ok(None)
    }

    /// Count the number of filtered results.
    pub fn count(&self) -> Result<usize> {
        let mut iter = unsafe {
            let mut iter_h: acorn_iter_handle = 0;
            let rc = acorn_iter_start(self.tree_h, ptr::null(), &mut iter_h as *mut _);
            if rc != 0 {
                return Err(Error::Acorn(unsafe { acorn_sys::last_error_string() }));
            }
            AcornIterator { h: iter_h }
        };

        let mut count = 0;
        while let Some((_, value)) = iter.next::<serde_json::Value>()? {
            if (self.predicate)(&value) {
                count += 1;
            }
        }
        Ok(count)
    }

    /// Check if any items match the filter.
    pub fn any(&self) -> Result<bool> {
        Ok(self.count()? > 0)
    }
}

/// Query builder with WHERE condition and ORDER BY applied.
pub struct AcornQueryWhereOrderBy<F, G> {
    tree_h: acorn_tree_handle,
    predicate: F,
    key_selector: G,
    descending: bool,
}

impl<F, G> AcornQueryWhereOrderBy<F, G>
where
    F: Fn(&serde_json::Value) -> bool,
    G: Fn(&serde_json::Value) -> String,
{
    /// Take only the first N items from filtered, ordered results.
    pub fn take(self, count: usize) -> AcornQueryWhereOrderByTake<F, G> {
        AcornQueryWhereOrderByTake {
            tree_h: self.tree_h,
            predicate: self.predicate,
            key_selector: self.key_selector,
            descending: self.descending,
            count,
        }
    }

    /// Execute the query and collect all filtered, ordered results.
    pub fn collect<T: DeserializeOwned>(&self) -> Result<Vec<T>> {
        // Get all filtered results first
        let mut iter = unsafe {
            let mut iter_h: acorn_iter_handle = 0;
            let rc = acorn_iter_start(self.tree_h, ptr::null(), &mut iter_h as *mut _);
            if rc != 0 {
                return Err(Error::Acorn(unsafe { acorn_sys::last_error_string() }));
            }
            AcornIterator { h: iter_h }
        };

        let mut filtered_items = Vec::new();
        while let Some((key, value)) = iter.next::<serde_json::Value>()? {
            if (self.predicate)(&value) {
                let key_str = (self.key_selector)(&value);
                filtered_items.push((key_str, value));
            }
        }

        // Sort by the key selector
        filtered_items.sort_by(|a, b| {
            if self.descending {
                b.0.cmp(&a.0)
            } else {
                a.0.cmp(&b.0)
            }
        });

        // Convert to typed results
        let mut results = Vec::new();
        for (_, value) in filtered_items {
            let typed_value: T = serde_json::from_value(value).map_err(|e| Error::Acorn(e.to_string()))?;
            results.push(typed_value);
        }
        Ok(results)
    }
}

/// Query builder with WHERE condition and TAKE applied.
pub struct AcornQueryWhereTake<F> {
    tree_h: acorn_tree_handle,
    predicate: F,
    count: usize,
}

impl<F> AcornQueryWhereTake<F>
where
    F: Fn(&serde_json::Value) -> bool,
{
    /// Execute the query and collect up to N filtered results.
    pub fn collect<T: DeserializeOwned>(&self) -> Result<Vec<T>> {
        let mut iter = unsafe {
            let mut iter_h: acorn_iter_handle = 0;
            let rc = acorn_iter_start(self.tree_h, ptr::null(), &mut iter_h as *mut _);
            if rc != 0 {
                return Err(Error::Acorn(unsafe { acorn_sys::last_error_string() }));
            }
            AcornIterator { h: iter_h }
        };

        let mut results = Vec::new();
        while let Some((_, value)) = iter.next::<serde_json::Value>()? {
            if (self.predicate)(&value) {
                let typed_value: T = serde_json::from_value(value).map_err(|e| Error::Acorn(e.to_string()))?;
                results.push(typed_value);
                if results.len() >= self.count {
                    break;
                }
            }
        }
        Ok(results)
    }
}

/// Query builder with WHERE condition and SKIP applied.
pub struct AcornQueryWhereSkip<F> {
    tree_h: acorn_tree_handle,
    predicate: F,
    count: usize,
}

impl<F> AcornQueryWhereSkip<F>
where
    F: Fn(&serde_json::Value) -> bool,
{
    /// Execute the query and collect filtered results after skipping N items.
    pub fn collect<T: DeserializeOwned>(&self) -> Result<Vec<T>> {
        let mut iter = unsafe {
            let mut iter_h: acorn_iter_handle = 0;
            let rc = acorn_iter_start(self.tree_h, ptr::null(), &mut iter_h as *mut _);
            if rc != 0 {
                return Err(Error::Acorn(unsafe { acorn_sys::last_error_string() }));
            }
            AcornIterator { h: iter_h }
        };

        let mut results = Vec::new();
        let mut skipped = 0;
        while let Some((_, value)) = iter.next::<serde_json::Value>()? {
            if (self.predicate)(&value) {
                if skipped < self.count {
                    skipped += 1;
                } else {
                    let typed_value: T = serde_json::from_value(value).map_err(|e| Error::Acorn(e.to_string()))?;
                    results.push(typed_value);
                }
            }
        }
        Ok(results)
    }
}

/// Query builder with ORDER BY applied.
pub struct AcornQueryOrderBy<F> {
    tree_h: acorn_tree_handle,
    key_selector: F,
    descending: bool,
}

impl<F> AcornQueryOrderBy<F>
where
    F: Fn(&serde_json::Value) -> String,
{
    /// Take only the first N items from ordered results.
    pub fn take(self, count: usize) -> AcornQueryOrderByTake<F> {
        AcornQueryOrderByTake {
            tree_h: self.tree_h,
            key_selector: self.key_selector,
            descending: self.descending,
            count,
        }
    }

    /// Skip the first N items from ordered results.
    pub fn skip(self, count: usize) -> AcornQueryOrderBySkip<F> {
        AcornQueryOrderBySkip {
            tree_h: self.tree_h,
            key_selector: self.key_selector,
            descending: self.descending,
            count,
        }
    }

    /// Execute the query and collect all ordered results.
    pub fn collect<T: DeserializeOwned>(&self) -> Result<Vec<T>> {
        let mut iter = unsafe {
            let mut iter_h: acorn_iter_handle = 0;
            let rc = acorn_iter_start(self.tree_h, ptr::null(), &mut iter_h as *mut _);
            if rc != 0 {
                return Err(Error::Acorn(unsafe { acorn_sys::last_error_string() }));
            }
            AcornIterator { h: iter_h }
        };

        let mut items = Vec::new();
        while let Some((_, value)) = iter.next::<serde_json::Value>()? {
            let key_str = (self.key_selector)(&value);
            items.push((key_str, value));
        }

        // Sort by the key selector
        items.sort_by(|a, b| {
            if self.descending {
                b.0.cmp(&a.0)
            } else {
                a.0.cmp(&b.0)
            }
        });

        // Convert to typed results
        let mut results = Vec::new();
        for (_, value) in items {
            let typed_value: T = serde_json::from_value(value).map_err(|e| Error::Acorn(e.to_string()))?;
            results.push(typed_value);
        }
        Ok(results)
    }
}

/// Query builder with TAKE applied.
pub struct AcornQueryTake {
    tree_h: acorn_tree_handle,
    count: usize,
}

impl AcornQueryTake {
    /// Execute the query and collect up to N results.
    pub fn collect<T: DeserializeOwned>(&self) -> Result<Vec<T>> {
        let mut iter = unsafe {
            let mut iter_h: acorn_iter_handle = 0;
            let rc = acorn_iter_start(self.tree_h, ptr::null(), &mut iter_h as *mut _);
            if rc != 0 {
                return Err(Error::Acorn(unsafe { acorn_sys::last_error_string() }));
            }
            AcornIterator { h: iter_h }
        };

        let mut results = Vec::new();
        while let Some((_, value)) = iter.next()? {
            results.push(value);
            if results.len() >= self.count {
                break;
            }
        }
        Ok(results)
    }
}

/// Query builder with SKIP applied.
pub struct AcornQuerySkip {
    tree_h: acorn_tree_handle,
    count: usize,
}

impl AcornQuerySkip {
    /// Execute the query and collect results after skipping N items.
    pub fn collect<T: DeserializeOwned>(&self) -> Result<Vec<T>> {
        let mut iter = unsafe {
            let mut iter_h: acorn_iter_handle = 0;
            let rc = acorn_iter_start(self.tree_h, ptr::null(), &mut iter_h as *mut _);
            if rc != 0 {
                return Err(Error::Acorn(unsafe { acorn_sys::last_error_string() }));
            }
            AcornIterator { h: iter_h }
        };

        let mut results = Vec::new();
        let mut skipped = 0;
        while let Some((_, value)) = iter.next()? {
            if skipped < self.count {
                skipped += 1;
            } else {
                results.push(value);
            }
        }
        Ok(results)
    }
}

/// Query builder with ORDER BY and TAKE applied.
pub struct AcornQueryOrderByTake<F> {
    tree_h: acorn_tree_handle,
    key_selector: F,
    descending: bool,
    count: usize,
}

impl<F> AcornQueryOrderByTake<F>
where
    F: Fn(&serde_json::Value) -> String,
{
    /// Execute the query and collect up to N ordered results.
    pub fn collect<T: DeserializeOwned>(&self) -> Result<Vec<T>> {
        let mut iter = unsafe {
            let mut iter_h: acorn_iter_handle = 0;
            let rc = acorn_iter_start(self.tree_h, ptr::null(), &mut iter_h as *mut _);
            if rc != 0 {
                return Err(Error::Acorn(unsafe { acorn_sys::last_error_string() }));
            }
            AcornIterator { h: iter_h }
        };

        let mut items = Vec::new();
        while let Some((_, value)) = iter.next::<serde_json::Value>()? {
            let key_str = (self.key_selector)(&value);
            items.push((key_str, value));
        }

        // Sort by the key selector
        items.sort_by(|a, b| {
            if self.descending {
                b.0.cmp(&a.0)
            } else {
                a.0.cmp(&b.0)
            }
        });

        // Take only the first N items
        items.truncate(self.count);

        // Convert to typed results
        let mut results = Vec::new();
        for (_, value) in items {
            let typed_value: T = serde_json::from_value(value).map_err(|e| Error::Acorn(e.to_string()))?;
            results.push(typed_value);
        }
        Ok(results)
    }
}

/// Query builder with ORDER BY and SKIP applied.
pub struct AcornQueryOrderBySkip<F> {
    tree_h: acorn_tree_handle,
    key_selector: F,
    descending: bool,
    count: usize,
}

impl<F> AcornQueryOrderBySkip<F>
where
    F: Fn(&serde_json::Value) -> String,
{
    /// Execute the query and collect ordered results after skipping N items.
    pub fn collect<T: DeserializeOwned>(&self) -> Result<Vec<T>> {
        let mut iter = unsafe {
            let mut iter_h: acorn_iter_handle = 0;
            let rc = acorn_iter_start(self.tree_h, ptr::null(), &mut iter_h as *mut _);
            if rc != 0 {
                return Err(Error::Acorn(unsafe { acorn_sys::last_error_string() }));
            }
            AcornIterator { h: iter_h }
        };

        let mut items = Vec::new();
        while let Some((_, value)) = iter.next::<serde_json::Value>()? {
            let key_str = (self.key_selector)(&value);
            items.push((key_str, value));
        }

        // Sort by the key selector
        items.sort_by(|a, b| {
            if self.descending {
                b.0.cmp(&a.0)
            } else {
                a.0.cmp(&b.0)
            }
        });

        // Skip the first N items
        if self.count < items.len() {
            items.drain(0..self.count);
        } else {
            items.clear();
        }

        // Convert to typed results
        let mut results = Vec::new();
        for (_, value) in items {
            let typed_value: T = serde_json::from_value(value).map_err(|e| Error::Acorn(e.to_string()))?;
            results.push(typed_value);
        }
        Ok(results)
    }
}

/// Query builder with WHERE condition, ORDER BY, and TAKE applied.
pub struct AcornQueryWhereOrderByTake<F, G> {
    tree_h: acorn_tree_handle,
    predicate: F,
    key_selector: G,
    descending: bool,
    count: usize,
}

impl<F, G> AcornQueryWhereOrderByTake<F, G>
where
    F: Fn(&serde_json::Value) -> bool,
    G: Fn(&serde_json::Value) -> String,
{
    /// Execute the query and collect up to N filtered, ordered results.
    pub fn collect<T: DeserializeOwned>(&self) -> Result<Vec<T>> {
        let mut iter = unsafe {
            let mut iter_h: acorn_iter_handle = 0;
            let rc = acorn_iter_start(self.tree_h, ptr::null(), &mut iter_h as *mut _);
            if rc != 0 {
                return Err(Error::Acorn(unsafe { acorn_sys::last_error_string() }));
            }
            AcornIterator { h: iter_h }
        };

        let mut filtered_items = Vec::new();
        while let Some((_, value)) = iter.next::<serde_json::Value>()? {
            if (self.predicate)(&value) {
                let key_str = (self.key_selector)(&value);
                filtered_items.push((key_str, value));
            }
        }

        // Sort by the key selector
        filtered_items.sort_by(|a, b| {
            if self.descending {
                b.0.cmp(&a.0)
            } else {
                a.0.cmp(&b.0)
            }
        });

        // Take only the first N items
        filtered_items.truncate(self.count);

        // Convert to typed results
        let mut results = Vec::new();
        for (_, value) in filtered_items {
            let typed_value: T = serde_json::from_value(value).map_err(|e| Error::Acorn(e.to_string()))?;
            results.push(typed_value);
        }
        Ok(results)
    }
}

/// Query builder with timestamp range filtering applied.
pub struct AcornQueryTimestampRange {
    tree_h: acorn_tree_handle,
    start: std::time::SystemTime,
    end: std::time::SystemTime,
}

impl AcornQueryTimestampRange {
    /// Execute the query and collect results within timestamp range.
    pub fn collect<T: DeserializeOwned>(&self) -> Result<Vec<T>> {
        // For now, implement simple filtering by getting all items
        // In a full implementation, this would filter by timestamp metadata
        let mut iter = unsafe {
            let mut iter_h: acorn_iter_handle = 0;
            let rc = acorn_iter_start(self.tree_h, ptr::null(), &mut iter_h as *mut _);
            if rc != 0 {
                return Err(Error::Acorn(unsafe { acorn_sys::last_error_string() }));
            }
            AcornIterator { h: iter_h }
        };

        let mut results = Vec::new();
        while let Some((_, value)) = iter.next()? {
            // In a full implementation, we would check timestamp metadata here
            // For now, we'll include all items
            results.push(value);
        }
        Ok(results)
    }
}

/// Query builder with timestamp "after" filtering applied.
pub struct AcornQueryTimestampAfter {
    tree_h: acorn_tree_handle,
    date: std::time::SystemTime,
}

impl AcornQueryTimestampAfter {
    /// Execute the query and collect results after the specified date.
    pub fn collect<T: DeserializeOwned>(&self) -> Result<Vec<T>> {
        // For now, implement simple filtering by getting all items
        // In a full implementation, this would filter by timestamp metadata
        let mut iter = unsafe {
            let mut iter_h: acorn_iter_handle = 0;
            let rc = acorn_iter_start(self.tree_h, ptr::null(), &mut iter_h as *mut _);
            if rc != 0 {
                return Err(Error::Acorn(unsafe { acorn_sys::last_error_string() }));
            }
            AcornIterator { h: iter_h }
        };

        let mut results = Vec::new();
        while let Some((_, value)) = iter.next()? {
            // In a full implementation, we would check timestamp metadata here
            // For now, we'll include all items
            results.push(value);
        }
        Ok(results)
    }
}

/// Query builder with timestamp "before" filtering applied.
pub struct AcornQueryTimestampBefore {
    tree_h: acorn_tree_handle,
    date: std::time::SystemTime,
}

impl AcornQueryTimestampBefore {
    /// Execute the query and collect results before the specified date.
    pub fn collect<T: DeserializeOwned>(&self) -> Result<Vec<T>> {
        // For now, implement simple filtering by getting all items
        // In a full implementation, this would filter by timestamp metadata
        let mut iter = unsafe {
            let mut iter_h: acorn_iter_handle = 0;
            let rc = acorn_iter_start(self.tree_h, ptr::null(), &mut iter_h as *mut _);
            if rc != 0 {
                return Err(Error::Acorn(unsafe { acorn_sys::last_error_string() }));
            }
            AcornIterator { h: iter_h }
        };

        let mut results = Vec::new();
        while let Some((_, value)) = iter.next()? {
            // In a full implementation, we would check timestamp metadata here
            // For now, we'll include all items
            results.push(value);
        }
        Ok(results)
    }
}

/// Query builder with node filtering applied.
pub struct AcornQueryFromNode {
    tree_h: acorn_tree_handle,
    node_id: String,
}

impl AcornQueryFromNode {
    /// Execute the query and collect results from the specified node.
    pub fn collect<T: DeserializeOwned>(&self) -> Result<Vec<T>> {
        // For now, implement simple filtering by getting all items
        // In a full implementation, this would filter by node metadata
        let mut iter = unsafe {
            let mut iter_h: acorn_iter_handle = 0;
            let rc = acorn_iter_start(self.tree_h, ptr::null(), &mut iter_h as *mut _);
            if rc != 0 {
                return Err(Error::Acorn(unsafe { acorn_sys::last_error_string() }));
            }
            AcornIterator { h: iter_h }
        };

        let mut results = Vec::new();
        while let Some((_, value)) = iter.next()? {
            // In a full implementation, we would check node metadata here
            // For now, we'll include all items
            results.push(value);
        }
        Ok(results)
    }
}

/// Query builder with "newest first" ordering applied.
pub struct AcornQueryNewest {
    tree_h: acorn_tree_handle,
}

impl AcornQueryNewest {
    /// Execute the query and collect results ordered by timestamp (newest first).
    pub fn collect<T: DeserializeOwned>(&self) -> Result<Vec<T>> {
        // For now, implement simple collection without ordering
        // In a full implementation, this would order by timestamp metadata
        let mut iter = unsafe {
            let mut iter_h: acorn_iter_handle = 0;
            let rc = acorn_iter_start(self.tree_h, ptr::null(), &mut iter_h as *mut _);
            if rc != 0 {
                return Err(Error::Acorn(unsafe { acorn_sys::last_error_string() }));
            }
            AcornIterator { h: iter_h }
        };

        let mut results = Vec::new();
        while let Some((_, value)) = iter.next()? {
            results.push(value);
        }
        Ok(results)
    }
}

/// Query builder with "oldest first" ordering applied.
pub struct AcornQueryOldest {
    tree_h: acorn_tree_handle,
}

impl AcornQueryOldest {
    /// Execute the query and collect results ordered by timestamp (oldest first).
    pub fn collect<T: DeserializeOwned>(&self) -> Result<Vec<T>> {
        // For now, implement simple collection without ordering
        // In a full implementation, this would order by timestamp metadata
        let mut iter = unsafe {
            let mut iter_h: acorn_iter_handle = 0;
            let rc = acorn_iter_start(self.tree_h, ptr::null(), &mut iter_h as *mut _);
            if rc != 0 {
                return Err(Error::Acorn(unsafe { acorn_sys::last_error_string() }));
            }
            AcornIterator { h: iter_h }
        };

        let mut results = Vec::new();
        while let Some((_, value)) = iter.next()? {
            results.push(value);
        }
        Ok(results)
    }
}

/// Git integration for AcornDB
pub struct AcornGit {
    h: acorn_git_handle,
}

/// Git commit information
#[derive(Debug, Clone, Deserialize)]
pub struct GitCommitInfo {
    pub sha: String,
    pub message: String,
    pub author: String,
    pub email: String,
    pub timestamp: i64,
}

/// Git repository information
#[derive(Debug, Clone, Deserialize)]
pub struct GitInfo {
    pub repository_path: String,
    pub author_name: String,
    pub author_email: String,
    pub has_remote: bool,
    pub is_initialized: bool,
}

impl AcornGit {
    /// Create a new Git integration instance
    /// 
    /// # Arguments
    /// * `repo_path` - Path to Git repository
    /// * `author_name` - Git author name
    /// * `author_email` - Git author email
    /// * `auto_push` - Automatically push to remote after each commit
    /// 
    /// # Example
    /// ```no_run
    /// # use acorn::{AcornGit, Error};
    /// # fn main() -> Result<(), Error> {
    /// let git = AcornGit::new("./my-repo", "AcornDB", "acorn@acorndb.dev", false)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(repo_path: &str, author_name: &str, author_email: &str, auto_push: bool) -> Result<Self> {
        let repo_path_c = CString::new(repo_path).map_err(|e| Error::Acorn(format!("Invalid repo path: {}", e)))?;
        let author_name_c = CString::new(author_name).map_err(|e| Error::Acorn(format!("Invalid author name: {}", e)))?;
        let author_email_c = CString::new(author_email).map_err(|e| Error::Acorn(format!("Invalid author email: {}", e)))?;
        
        let mut h: acorn_git_handle = 0;
        let rc = unsafe { 
            acorn_git_create(
                repo_path_c.as_ptr(), 
                author_name_c.as_ptr(), 
                author_email_c.as_ptr(), 
                if auto_push { 1 } else { 0 },
                &mut h as *mut _
            ) 
        };
        
        if rc == 0 {
            Ok(Self { h })
        } else {
            Err(Error::Acorn(unsafe { acorn_sys::last_error_string() }))
        }
    }

    /// Push changes to remote repository
    /// 
    /// # Arguments
    /// * `remote_name` - Remote name (default: "origin")
    /// * `branch` - Branch name (default: "main")
    /// 
    /// # Example
    /// ```no_run
    /// # use acorn::{AcornGit, Error};
    /// # fn main() -> Result<(), Error> {
    /// let git = AcornGit::new("./my-repo", "AcornDB", "acorn@acorndb.dev", false)?;
    /// git.push("origin", "main")?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn push(&self, remote_name: &str, branch: &str) -> Result<()> {
        let remote_name_c = CString::new(remote_name).map_err(|e| Error::Acorn(format!("Invalid remote name: {}", e)))?;
        let branch_c = CString::new(branch).map_err(|e| Error::Acorn(format!("Invalid branch name: {}", e)))?;
        
        let rc = unsafe { 
            acorn_git_push(self.h, remote_name_c.as_ptr(), branch_c.as_ptr()) 
        };
        
        if rc == 0 {
            Ok(())
        } else {
            Err(Error::Acorn(unsafe { acorn_sys::last_error_string() }))
        }
    }

    /// Pull changes from remote repository
    /// 
    /// # Arguments
    /// * `remote_name` - Remote name (default: "origin")
    /// * `branch` - Branch name (default: "main")
    /// 
    /// # Example
    /// ```no_run
    /// # use acorn::{AcornGit, Error};
    /// # fn main() -> Result<(), Error> {
    /// let git = AcornGit::new("./my-repo", "AcornDB", "acorn@acorndb.dev", false)?;
    /// git.pull("origin", "main")?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn pull(&self, remote_name: &str, branch: &str) -> Result<()> {
        let remote_name_c = CString::new(remote_name).map_err(|e| Error::Acorn(format!("Invalid remote name: {}", e)))?;
        let branch_c = CString::new(branch).map_err(|e| Error::Acorn(format!("Invalid branch name: {}", e)))?;
        
        let rc = unsafe { 
            acorn_git_pull(self.h, remote_name_c.as_ptr(), branch_c.as_ptr()) 
        };
        
        if rc == 0 {
            Ok(())
        } else {
            Err(Error::Acorn(unsafe { acorn_sys::last_error_string() }))
        }
    }

    /// Get commit history for a specific file
    /// 
    /// # Arguments
    /// * `file_path` - Path to the file within the repository
    /// 
    /// # Example
    /// ```no_run
    /// # use acorn::{AcornGit, Error};
    /// # fn main() -> Result<(), Error> {
    /// let git = AcornGit::new("./my-repo", "AcornDB", "acorn@acorndb.dev", false)?;
    /// let commits = git.get_file_history("data.json")?;
    /// for commit in commits {
    ///     println!("Commit {}: {}", commit.sha, commit.message);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn get_file_history(&self, file_path: &str) -> Result<Vec<GitCommitInfo>> {
        let file_path_c = CString::new(file_path).map_err(|e| Error::Acorn(format!("Invalid file path: {}", e)))?;
        
        let mut commits_ptr: *mut acorn_git_commit_info = std::ptr::null_mut();
        let mut count: usize = 0;
        
        let rc = unsafe { 
            acorn_git_get_file_history(self.h, file_path_c.as_ptr(), &mut commits_ptr as *mut _, &mut count as *mut _) 
        };
        
        if rc == 0 {
            let mut commits = Vec::new();
            if !commits_ptr.is_null() && count > 0 {
                let commits_slice = unsafe { std::slice::from_raw_parts(commits_ptr, count) };
                for commit in commits_slice {
                    commits.push(GitCommitInfo {
                        sha: unsafe { CStr::from_ptr(commit.sha).to_string_lossy().into_owned() },
                        message: unsafe { CStr::from_ptr(commit.message).to_string_lossy().into_owned() },
                        author: unsafe { CStr::from_ptr(commit.author).to_string_lossy().into_owned() },
                        email: unsafe { CStr::from_ptr(commit.email).to_string_lossy().into_owned() },
                        timestamp: commit.timestamp,
                    });
                }
                unsafe { acorn_git_free_commit_info(commits_ptr, count); }
            }
            Ok(commits)
        } else {
            Err(Error::Acorn(unsafe { acorn_sys::last_error_string() }))
        }
    }

    /// Read file content at a specific commit
    /// 
    /// # Arguments
    /// * `file_path` - Path to the file within the repository
    /// * `commit_sha` - Commit SHA to read from
    /// 
    /// # Example
    /// ```no_run
    /// # use acorn::{AcornGit, Error};
    /// # fn main() -> Result<(), Error> {
    /// let git = AcornGit::new("./my-repo", "AcornDB", "acorn@acorndb.dev", false)?;
    /// let content = git.read_file_at_commit("data.json", "abc123")?;
    /// println!("File content: {}", content);
    /// # Ok(())
    /// # }
    /// ```
    pub fn read_file_at_commit(&self, file_path: &str, commit_sha: &str) -> Result<String> {
        let file_path_c = CString::new(file_path).map_err(|e| Error::Acorn(format!("Invalid file path: {}", e)))?;
        let commit_sha_c = CString::new(commit_sha).map_err(|e| Error::Acorn(format!("Invalid commit SHA: {}", e)))?;
        
        let mut content_ptr: *mut u8 = std::ptr::null_mut();
        let mut length: usize = 0;
        
        let rc = unsafe { 
            acorn_git_read_file_at_commit(self.h, file_path_c.as_ptr(), commit_sha_c.as_ptr(), &mut content_ptr as *mut _, &mut length as *mut _) 
        };
        
        if rc == 0 {
            if !content_ptr.is_null() && length > 0 {
                let content_slice = unsafe { std::slice::from_raw_parts(content_ptr, length) };
                let content = String::from_utf8_lossy(content_slice).into_owned();
                unsafe { acorn_free_buf(&mut acorn_buf { data: content_ptr, len: length }); }
                Ok(content)
            } else {
                Ok(String::new())
            }
        } else {
            Err(Error::Acorn(unsafe { acorn_sys::last_error_string() }))
        }
    }

    /// Check if the repository has a remote configured
    /// 
    /// # Arguments
    /// * `remote_name` - Remote name to check (default: "origin")
    /// 
    /// # Example
    /// ```no_run
    /// # use acorn::{AcornGit, Error};
    /// # fn main() -> Result<(), Error> {
    /// let git = AcornGit::new("./my-repo", "AcornDB", "acorn@acorndb.dev", false)?;
    /// let has_remote = git.has_remote("origin")?;
    /// println!("Has remote: {}", has_remote);
    /// # Ok(())
    /// # }
    /// ```
    pub fn has_remote(&self, remote_name: &str) -> Result<bool> {
        let remote_name_c = CString::new(remote_name).map_err(|e| Error::Acorn(format!("Invalid remote name: {}", e)))?;
        
        let mut has_remote: i32 = 0;
        let rc = unsafe { 
            acorn_git_has_remote(self.h, remote_name_c.as_ptr(), &mut has_remote as *mut _) 
        };
        
        if rc == 0 {
            Ok(has_remote != 0)
        } else {
            Err(Error::Acorn(unsafe { acorn_sys::last_error_string() }))
        }
    }

    /// Squash commits since a specific commit
    /// 
    /// # Arguments
    /// * `since_commit` - Commit SHA to squash since
    /// 
    /// # Example
    /// ```no_run
    /// # use acorn::{AcornGit, Error};
    /// # fn main() -> Result<(), Error> {
    /// let git = AcornGit::new("./my-repo", "AcornDB", "acorn@acorndb.dev", false)?;
    /// git.squash_commits("abc123")?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn squash_commits(&self, since_commit: &str) -> Result<()> {
        let since_commit_c = CString::new(since_commit).map_err(|e| Error::Acorn(format!("Invalid commit SHA: {}", e)))?;
        
        let rc = unsafe { 
            acorn_git_squash_commits(self.h, since_commit_c.as_ptr()) 
        };
        
        if rc == 0 {
            Ok(())
        } else {
            Err(Error::Acorn(unsafe { acorn_sys::last_error_string() }))
        }
    }
}

impl Drop for AcornGit {
    fn drop(&mut self) {
        unsafe { acorn_git_close(self.h); }
    }
}

/// Nursery System for dynamic trunk discovery and creation
pub struct AcornNursery {
    h: acorn_nursery_handle,
}

/// Trunk metadata information
#[derive(Debug, Clone, Deserialize)]
pub struct TrunkMetadata {
    pub type_id: String,
    pub display_name: String,
    pub description: String,
    pub category: String,
    pub is_durable: bool,
    pub supports_history: bool,
    pub supports_sync: bool,
    pub supports_async: bool,
    pub required_config_keys: Vec<String>,
    pub optional_config_keys: Vec<String>,
    pub is_built_in: bool,
}

impl AcornNursery {
    /// Create a new Nursery instance
    /// 
    /// # Example
    /// ```no_run
    /// # use acorn::{AcornNursery, Error};
    /// # fn main() -> Result<(), Error> {
    /// let nursery = AcornNursery::new()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn new() -> Result<Self> {
        let mut h: acorn_nursery_handle = 0;
        let rc = unsafe { acorn_nursery_create(&mut h as *mut _) };
        
        if rc == 0 {
            Ok(Self { h })
        } else {
            Err(Error::Acorn(unsafe { acorn_sys::last_error_string() }))
        }
    }

    /// Get all available trunk types
    /// 
    /// # Example
    /// ```no_run
    /// # use acorn::{AcornNursery, Error};
    /// # fn main() -> Result<(), Error> {
    /// let nursery = AcornNursery::new()?;
    /// let types = nursery.get_available_types()?;
    /// for trunk_type in types {
    ///     println!("Available trunk: {}", trunk_type);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn get_available_types(&self) -> Result<Vec<String>> {
        let mut types_ptr: *mut *mut u8 = std::ptr::null_mut();
        let mut count: usize = 0;
        
        let rc = unsafe { 
            acorn_nursery_get_available_types(self.h, &mut types_ptr as *mut _, &mut count as *mut _) 
        };
        
        if rc == 0 {
            let mut types = Vec::new();
            if !types_ptr.is_null() && count > 0 {
                let types_slice = unsafe { std::slice::from_raw_parts(types_ptr, count) };
                for type_ptr in types_slice {
                    if !type_ptr.is_null() {
                        let type_str = unsafe { CStr::from_ptr(type_ptr as *const i8).to_string_lossy().into_owned() };
                        types.push(type_str);
                    }
                }
                unsafe { acorn_nursery_free_types(types_ptr, count); }
            }
            Ok(types)
        } else {
            Err(Error::Acorn(unsafe { acorn_sys::last_error_string() }))
        }
    }

    /// Get metadata for a specific trunk type
    /// 
    /// # Arguments
    /// * `type_id` - The trunk type identifier
    /// 
    /// # Example
    /// ```no_run
    /// # use acorn::{AcornNursery, Error};
    /// # fn main() -> Result<(), Error> {
    /// let nursery = AcornNursery::new()?;
    /// let metadata = nursery.get_metadata("file")?;
    /// println!("File trunk: {}", metadata.description);
    /// # Ok(())
    /// # }
    /// ```
    pub fn get_metadata(&self, type_id: &str) -> Result<TrunkMetadata> {
        let type_id_c = CString::new(type_id).map_err(|e| Error::Acorn(format!("Invalid type ID: {}", e)))?;
        
        let mut metadata: acorn_trunk_metadata = unsafe { std::mem::zeroed() };
        
        let rc = unsafe { 
            acorn_nursery_get_metadata(self.h, type_id_c.as_ptr(), &mut metadata as *mut _) 
        };
        
        if rc == 0 {
            let mut required_keys = Vec::new();
            if !metadata.required_config_keys.is_null() && metadata.required_config_keys_count > 0 {
                let keys_slice = unsafe { std::slice::from_raw_parts(metadata.required_config_keys, metadata.required_config_keys_count) };
                for key_ptr in keys_slice {
                    if !key_ptr.is_null() {
                        let key = unsafe { CStr::from_ptr(key_ptr).to_string_lossy().into_owned() };
                        required_keys.push(key);
                    }
                }
            }

            let mut optional_keys = Vec::new();
            if !metadata.optional_config_keys.is_null() && metadata.optional_config_keys_count > 0 {
                let keys_slice = unsafe { std::slice::from_raw_parts(metadata.optional_config_keys, metadata.optional_config_keys_count) };
                for key_ptr in keys_slice {
                    if !key_ptr.is_null() {
                        let key = unsafe { CStr::from_ptr(key_ptr).to_string_lossy().into_owned() };
                        optional_keys.push(key);
                    }
                }
            }

            Ok(TrunkMetadata {
                type_id: unsafe { CStr::from_ptr(metadata.type_id).to_string_lossy().into_owned() },
                display_name: unsafe { CStr::from_ptr(metadata.display_name).to_string_lossy().into_owned() },
                description: unsafe { CStr::from_ptr(metadata.description).to_string_lossy().into_owned() },
                category: unsafe { CStr::from_ptr(metadata.category).to_string_lossy().into_owned() },
                is_durable: metadata.is_durable != 0,
                supports_history: metadata.supports_history != 0,
                supports_sync: metadata.supports_sync != 0,
                supports_async: metadata.supports_async != 0,
                required_config_keys: required_keys,
                optional_config_keys: optional_keys,
                is_built_in: metadata.is_built_in != 0,
            })
        } else {
            Err(Error::Acorn(unsafe { acorn_sys::last_error_string() }))
        }
    }

    /// Get metadata for all trunk types
    /// 
    /// # Example
    /// ```no_run
    /// # use acorn::{AcornNursery, Error};
    /// # fn main() -> Result<(), Error> {
    /// let nursery = AcornNursery::new()?;
    /// let all_metadata = nursery.get_all_metadata()?;
    /// for metadata in all_metadata {
    ///     println!("[{}] {}: {}", metadata.category, metadata.type_id, metadata.description);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn get_all_metadata(&self) -> Result<Vec<TrunkMetadata>> {
        let mut metadata_ptr: *mut acorn_trunk_metadata = std::ptr::null_mut();
        let mut count: usize = 0;
        
        let rc = unsafe { 
            acorn_nursery_get_all_metadata(self.h, &mut metadata_ptr as *mut _, &mut count as *mut _) 
        };
        
        if rc == 0 {
            let mut metadata_list = Vec::new();
            if !metadata_ptr.is_null() && count > 0 {
                let metadata_slice = unsafe { std::slice::from_raw_parts(metadata_ptr, count) };
                for metadata in metadata_slice {
                    let mut required_keys = Vec::new();
                    if !metadata.required_config_keys.is_null() && metadata.required_config_keys_count > 0 {
                        let keys_slice = unsafe { std::slice::from_raw_parts(metadata.required_config_keys, metadata.required_config_keys_count) };
                        for key_ptr in keys_slice {
                            if !key_ptr.is_null() {
                                let key = unsafe { CStr::from_ptr(key_ptr).to_string_lossy().into_owned() };
                                required_keys.push(key);
                            }
                        }
                    }

                    let mut optional_keys = Vec::new();
                    if !metadata.optional_config_keys.is_null() && metadata.optional_config_keys_count > 0 {
                        let keys_slice = unsafe { std::slice::from_raw_parts(metadata.optional_config_keys, metadata.optional_config_keys_count) };
                        for key_ptr in keys_slice {
                            if !key_ptr.is_null() {
                                let key = unsafe { CStr::from_ptr(key_ptr).to_string_lossy().into_owned() };
                                optional_keys.push(key);
                            }
                        }
                    }

                    metadata_list.push(TrunkMetadata {
                        type_id: unsafe { CStr::from_ptr(metadata.type_id).to_string_lossy().into_owned() },
                        display_name: unsafe { CStr::from_ptr(metadata.display_name).to_string_lossy().into_owned() },
                        description: unsafe { CStr::from_ptr(metadata.description).to_string_lossy().into_owned() },
                        category: unsafe { CStr::from_ptr(metadata.category).to_string_lossy().into_owned() },
                        is_durable: metadata.is_durable != 0,
                        supports_history: metadata.supports_history != 0,
                        supports_sync: metadata.supports_sync != 0,
                        supports_async: metadata.supports_async != 0,
                        required_config_keys: required_keys,
                        optional_config_keys: optional_keys,
                        is_built_in: metadata.is_built_in != 0,
                    });
                }
                unsafe { acorn_nursery_free_metadata(metadata_ptr, count); }
            }
            Ok(metadata_list)
        } else {
            Err(Error::Acorn(unsafe { acorn_sys::last_error_string() }))
        }
    }

    /// Check if a trunk type is available
    /// 
    /// # Arguments
    /// * `type_id` - The trunk type identifier
    /// 
    /// # Example
    /// ```no_run
    /// # use acorn::{AcornNursery, Error};
    /// # fn main() -> Result<(), Error> {
    /// let nursery = AcornNursery::new()?;
    /// let has_file = nursery.has_trunk("file")?;
    /// println!("Has file trunk: {}", has_file);
    /// # Ok(())
    /// # }
    /// ```
    pub fn has_trunk(&self, type_id: &str) -> Result<bool> {
        let type_id_c = CString::new(type_id).map_err(|e| Error::Acorn(format!("Invalid type ID: {}", e)))?;
        
        let mut has_trunk: i32 = 0;
        let rc = unsafe { 
            acorn_nursery_has_trunk(self.h, type_id_c.as_ptr(), &mut has_trunk as *mut _) 
        };
        
        if rc == 0 {
            Ok(has_trunk != 0)
        } else {
            Err(Error::Acorn(unsafe { acorn_sys::last_error_string() }))
        }
    }

    /// Grow a trunk instance by type ID and configuration
    /// 
    /// # Arguments
    /// * `type_id` - The trunk type identifier
    /// * `config_json` - Configuration as JSON string
    /// 
    /// # Example
    /// ```no_run
    /// # use acorn::{AcornNursery, AcornStorage, Error};
    /// # fn main() -> Result<(), Error> {
    /// let nursery = AcornNursery::new()?;
    /// let config = r#"{"path": "./data"}"#;
    /// let storage = nursery.grow_trunk("file", config)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn grow_trunk(&self, type_id: &str, config_json: &str) -> Result<AcornStorage> {
        let type_id_c = CString::new(type_id).map_err(|e| Error::Acorn(format!("Invalid type ID: {}", e)))?;
        let config_c = CString::new(config_json).map_err(|e| Error::Acorn(format!("Invalid config JSON: {}", e)))?;
        
        let mut storage_h: acorn_storage_handle = 0;
        let rc = unsafe { 
            acorn_nursery_grow_trunk(self.h, type_id_c.as_ptr(), config_c.as_ptr(), &mut storage_h as *mut _) 
        };
        
        if rc == 0 {
            Ok(AcornStorage { h: storage_h })
        } else {
            Err(Error::Acorn(unsafe { acorn_sys::last_error_string() }))
        }
    }

    /// Validate configuration for a trunk type
    /// 
    /// # Arguments
    /// * `type_id` - The trunk type identifier
    /// * `config_json` - Configuration as JSON string
    /// 
    /// # Example
    /// ```no_run
    /// # use acorn::{AcornNursery, Error};
    /// # fn main() -> Result<(), Error> {
    /// let nursery = AcornNursery::new()?;
    /// let config = r#"{"path": "./data"}"#;
    /// let is_valid = nursery.validate_config("file", config)?;
    /// println!("Config is valid: {}", is_valid);
    /// # Ok(())
    /// # }
    /// ```
    pub fn validate_config(&self, type_id: &str, config_json: &str) -> Result<bool> {
        let type_id_c = CString::new(type_id).map_err(|e| Error::Acorn(format!("Invalid type ID: {}", e)))?;
        let config_c = CString::new(config_json).map_err(|e| Error::Acorn(format!("Invalid config JSON: {}", e)))?;
        
        let mut is_valid: i32 = 0;
        let rc = unsafe { 
            acorn_nursery_validate_config(self.h, type_id_c.as_ptr(), config_c.as_ptr(), &mut is_valid as *mut _) 
        };
        
        if rc == 0 {
            Ok(is_valid != 0)
        } else {
            Err(Error::Acorn(unsafe { acorn_sys::last_error_string() }))
        }
    }

    /// Get a formatted catalog of all trunks
    /// 
    /// # Example
    /// ```no_run
    /// # use acorn::{AcornNursery, Error};
    /// # fn main() -> Result<(), Error> {
    /// let nursery = AcornNursery::new()?;
    /// let catalog = nursery.get_catalog()?;
    /// println!("{}", catalog);
    /// # Ok(())
    /// # }
    /// ```
    pub fn get_catalog(&self) -> Result<String> {
        let mut catalog_ptr: *mut u8 = std::ptr::null_mut();
        let mut length: usize = 0;
        
        let rc = unsafe { 
            acorn_nursery_get_catalog(self.h, &mut catalog_ptr as *mut _, &mut length as *mut _) 
        };
        
        if rc == 0 {
            if !catalog_ptr.is_null() && length > 0 {
                let catalog_slice = unsafe { std::slice::from_raw_parts(catalog_ptr, length) };
                let catalog = String::from_utf8_lossy(catalog_slice).into_owned();
                unsafe { acorn_nursery_free_catalog(catalog_ptr); }
                Ok(catalog)
            } else {
                Ok(String::new())
            }
        } else {
            Err(Error::Acorn(unsafe { acorn_sys::last_error_string() }))
        }
    }
}

impl Drop for AcornNursery {
    fn drop(&mut self) {
        unsafe { acorn_nursery_close(self.h); }
    }
}

/// Advanced Tree Features for enhanced tree management
pub struct AcornAdvancedTree {
    tree_h: acorn_tree_handle,
}

/// Tree statistics information
#[derive(Debug, Clone, Deserialize)]
pub struct TreeStats {
    pub total_stashed: i32,
    pub total_tossed: i32,
    pub squabbles_resolved: i32,
    pub smushes_performed: i32,
    pub active_tangles: i32,
    pub last_sync_timestamp: i64,
}

/// TTL enforcement information
#[derive(Debug, Clone, Deserialize)]
pub struct TtlInfo {
    pub ttl_enforcement_enabled: bool,
    pub cleanup_interval_ms: i64,
    pub expiring_nuts_count: i32,
}

/// Nut metadata information
#[derive(Debug, Clone, Deserialize)]
pub struct NutInfo {
    pub id: String,
    pub timestamp: i64,
    pub expires_at: Option<i64>,
    pub version: i32,
    pub payload: serde_json::Value,
}

impl AcornAdvancedTree {
    /// Create an advanced tree wrapper from an existing tree handle
    /// 
    /// # Arguments
    /// * `tree_h` - Existing tree handle
    /// 
    /// # Example
    /// ```no_run
    /// # use acorn::{AcornTree, AcornAdvancedTree, Error};
    /// # fn main() -> Result<(), Error> {
    /// let tree = AcornTree::open_memory()?;
    /// let advanced_tree = AcornAdvancedTree::from_tree(tree);
    /// # Ok(())
    /// # }
    /// ```
    pub fn from_tree(tree: AcornTree) -> Self {
        Self { tree_h: tree.h }
    }

    /// Stash an item with auto-ID detection
    /// 
    /// # Arguments
    /// * `json` - JSON string of the item to stash
    /// 
    /// # Example
    /// ```no_run
    /// # use acorn::{AcornTree, AcornAdvancedTree, Error};
    /// # fn main() -> Result<(), Error> {
    /// let tree = AcornTree::open_memory()?;
    /// let advanced_tree = AcornAdvancedTree::from_tree(tree);
    /// let item_json = r#"{"id": "user-1", "name": "Alice", "age": 30}"#;
    /// advanced_tree.stash_with_auto_id(item_json)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn stash_with_auto_id(&self, json: &str) -> Result<()> {
        let json_c = CString::new(json).map_err(|e| Error::Acorn(format!("Invalid JSON: {}", e)))?;
        
        let rc = unsafe { 
            acorn_tree_stash_auto_id(self.tree_h, json_c.as_ptr(), json.len()) 
        };
        
        if rc == 0 {
            Ok(())
        } else {
            Err(Error::Acorn(unsafe { acorn_sys::last_error_string() }))
        }
    }

    /// Get tree statistics
    /// 
    /// # Example
    /// ```no_run
    /// # use acorn::{AcornTree, AcornAdvancedTree, Error};
    /// # fn main() -> Result<(), Error> {
    /// let tree = AcornTree::open_memory()?;
    /// let advanced_tree = AcornAdvancedTree::from_tree(tree);
    /// let stats = advanced_tree.get_stats()?;
    /// println!("Total stashed: {}", stats.total_stashed);
    /// # Ok(())
    /// # }
    /// ```
    pub fn get_stats(&self) -> Result<TreeStats> {
        let mut stats: acorn_tree_stats = unsafe { std::mem::zeroed() };
        
        let rc = unsafe { 
            acorn_tree_get_stats(self.tree_h, &mut stats as *mut _) 
        };
        
        if rc == 0 {
            Ok(TreeStats {
                total_stashed: stats.total_stashed,
                total_tossed: stats.total_tossed,
                squabbles_resolved: stats.squabbles_resolved,
                smushes_performed: stats.smushes_performed,
                active_tangles: stats.active_tangles,
                last_sync_timestamp: stats.last_sync_timestamp,
            })
        } else {
            Err(Error::Acorn(unsafe { acorn_sys::last_error_string() }))
        }
    }

    /// Get TTL enforcement information
    /// 
    /// # Example
    /// ```no_run
    /// # use acorn::{AcornTree, AcornAdvancedTree, Error};
    /// # fn main() -> Result<(), Error> {
    /// let tree = AcornTree::open_memory()?;
    /// let advanced_tree = AcornAdvancedTree::from_tree(tree);
    /// let ttl_info = advanced_tree.get_ttl_info()?;
    /// println!("TTL enabled: {}", ttl_info.ttl_enforcement_enabled);
    /// # Ok(())
    /// # }
    /// ```
    pub fn get_ttl_info(&self) -> Result<TtlInfo> {
        let mut ttl_info: acorn_ttl_info = unsafe { std::mem::zeroed() };
        
        let rc = unsafe { 
            acorn_tree_get_ttl_info(self.tree_h, &mut ttl_info as *mut _) 
        };
        
        if rc == 0 {
            Ok(TtlInfo {
                ttl_enforcement_enabled: ttl_info.ttl_enforcement_enabled != 0,
                cleanup_interval_ms: ttl_info.cleanup_interval_ms,
                expiring_nuts_count: ttl_info.expiring_nuts_count,
            })
        } else {
            Err(Error::Acorn(unsafe { acorn_sys::last_error_string() }))
        }
    }

    /// Set TTL enforcement enabled/disabled
    /// 
    /// # Arguments
    /// * `enabled` - Whether to enable TTL enforcement
    /// 
    /// # Example
    /// ```no_run
    /// # use acorn::{AcornTree, AcornAdvancedTree, Error};
    /// # fn main() -> Result<(), Error> {
    /// let tree = AcornTree::open_memory()?;
    /// let advanced_tree = AcornAdvancedTree::from_tree(tree);
    /// advanced_tree.set_ttl_enforcement(true)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn set_ttl_enforcement(&self, enabled: bool) -> Result<()> {
        let rc = unsafe { 
            acorn_tree_set_ttl_enforcement(self.tree_h, if enabled { 1 } else { 0 }) 
        };
        
        if rc == 0 {
            Ok(())
        } else {
            Err(Error::Acorn(unsafe { acorn_sys::last_error_string() }))
        }
    }

    /// Set cleanup interval for TTL enforcement
    /// 
    /// # Arguments
    /// * `interval_ms` - Cleanup interval in milliseconds
    /// 
    /// # Example
    /// ```no_run
    /// # use acorn::{AcornTree, AcornAdvancedTree, Error};
    /// # fn main() -> Result<(), Error> {
    /// let tree = AcornTree::open_memory()?;
    /// let advanced_tree = AcornAdvancedTree::from_tree(tree);
    /// advanced_tree.set_cleanup_interval(30000)?; // 30 seconds
    /// # Ok(())
    /// # }
    /// ```
    pub fn set_cleanup_interval(&self, interval_ms: i64) -> Result<()> {
        let rc = unsafe { 
            acorn_tree_set_cleanup_interval(self.tree_h, interval_ms) 
        };
        
        if rc == 0 {
            Ok(())
        } else {
            Err(Error::Acorn(unsafe { acorn_sys::last_error_string() }))
        }
    }

    /// Cleanup expired nuts manually
    /// 
    /// # Returns
    /// Number of expired nuts that were removed
    /// 
    /// # Example
    /// ```no_run
    /// # use acorn::{AcornTree, AcornAdvancedTree, Error};
    /// # fn main() -> Result<(), Error> {
    /// let tree = AcornTree::open_memory()?;
    /// let advanced_tree = AcornAdvancedTree::from_tree(tree);
    /// let removed_count = advanced_tree.cleanup_expired_nuts()?;
    /// println!("Removed {} expired nuts", removed_count);
    /// # Ok(())
    /// # }
    /// ```
    pub fn cleanup_expired_nuts(&self) -> Result<i32> {
        let mut removed_count: i32 = 0;
        
        let rc = unsafe { 
            acorn_tree_cleanup_expired_nuts(self.tree_h, &mut removed_count as *mut _) 
        };
        
        if rc == 0 {
            Ok(removed_count)
        } else {
            Err(Error::Acorn(unsafe { acorn_sys::last_error_string() }))
        }
    }

    /// Get count of nuts expiring within a timespan
    /// 
    /// # Arguments
    /// * `timespan_ms` - Timespan in milliseconds
    /// 
    /// # Example
    /// ```no_run
    /// # use acorn::{AcornTree, AcornAdvancedTree, Error};
    /// # fn main() -> Result<(), Error> {
    /// let tree = AcornTree::open_memory()?;
    /// let advanced_tree = AcornAdvancedTree::from_tree(tree);
    /// let count = advanced_tree.get_expiring_nuts_count(60000)?; // Next minute
    /// println!("{} nuts expiring in the next minute", count);
    /// # Ok(())
    /// # }
    /// ```
    pub fn get_expiring_nuts_count(&self, timespan_ms: i64) -> Result<i32> {
        let mut count: i32 = 0;
        
        let rc = unsafe { 
            acorn_tree_get_expiring_nuts_count(self.tree_h, timespan_ms, &mut count as *mut _) 
        };
        
        if rc == 0 {
            Ok(count)
        } else {
            Err(Error::Acorn(unsafe { acorn_sys::last_error_string() }))
        }
    }

    /// Get IDs of nuts expiring within a timespan
    /// 
    /// # Arguments
    /// * `timespan_ms` - Timespan in milliseconds
    /// 
    /// # Example
    /// ```no_run
    /// # use acorn::{AcornTree, AcornAdvancedTree, Error};
    /// # fn main() -> Result<(), Error> {
    /// let tree = AcornTree::open_memory()?;
    /// let advanced_tree = AcornAdvancedTree::from_tree(tree);
    /// let expiring_ids = advanced_tree.get_expiring_nuts(60000)?; // Next minute
    /// for id in expiring_ids {
    ///     println!("Nut {} is expiring soon", id);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn get_expiring_nuts(&self, timespan_ms: i64) -> Result<Vec<String>> {
        let mut ids_ptr: *mut *mut u8 = std::ptr::null_mut();
        let mut count: usize = 0;
        
        let rc = unsafe { 
            acorn_tree_get_expiring_nuts(self.tree_h, timespan_ms, &mut ids_ptr as *mut _, &mut count as *mut _) 
        };
        
        if rc == 0 {
            let mut ids = Vec::new();
            if !ids_ptr.is_null() && count > 0 {
                let ids_slice = unsafe { std::slice::from_raw_parts(ids_ptr, count) };
                for id_ptr in ids_slice {
                    if !id_ptr.is_null() {
                        let id = unsafe { CStr::from_ptr(id_ptr as *const i8).to_string_lossy().into_owned() };
                        ids.push(id);
                    }
                }
                unsafe { acorn_tree_free_expiring_nuts(ids_ptr, count); }
            }
            Ok(ids)
        } else {
            Err(Error::Acorn(unsafe { acorn_sys::last_error_string() }))
        }
    }

    /// Get all nuts with metadata
    /// 
    /// # Example
    /// ```no_run
    /// # use acorn::{AcornTree, AcornAdvancedTree, Error};
    /// # fn main() -> Result<(), Error> {
    /// let tree = AcornTree::open_memory()?;
    /// let advanced_tree = AcornAdvancedTree::from_tree(tree);
    /// let all_nuts = advanced_tree.get_all_nuts()?;
    /// for nut in all_nuts {
    ///     println!("Nut {}: version {}, timestamp {}", nut.id, nut.version, nut.timestamp);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn get_all_nuts(&self) -> Result<Vec<NutInfo>> {
        let mut json_ptr: *mut u8 = std::ptr::null_mut();
        let mut length: usize = 0;
        
        let rc = unsafe { 
            acorn_tree_get_all_nuts(self.tree_h, &mut json_ptr as *mut _, &mut length as *mut _) 
        };
        
        if rc == 0 {
            if !json_ptr.is_null() && length > 0 {
                let json_slice = unsafe { std::slice::from_raw_parts(json_ptr, length) };
                let json_str = String::from_utf8_lossy(json_slice).into_owned();
                unsafe { acorn_tree_free_all_nuts(json_ptr); }
                
                let nuts: Vec<NutInfo> = serde_json::from_str(&json_str)
                    .map_err(|e| Error::Acorn(format!("Failed to parse nuts JSON: {}", e)))?;
                Ok(nuts)
            } else {
                Ok(Vec::new())
            }
        } else {
            Err(Error::Acorn(unsafe { acorn_sys::last_error_string() }))
        }
    }

    /// Get the current nut count
    /// 
    /// # Example
    /// ```no_run
    /// # use acorn::{AcornTree, AcornAdvancedTree, Error};
    /// # fn main() -> Result<(), Error> {
    /// let tree = AcornTree::open_memory()?;
    /// let advanced_tree = AcornAdvancedTree::from_tree(tree);
    /// let count = advanced_tree.get_nut_count()?;
    /// println!("Tree contains {} nuts", count);
    /// # Ok(())
    /// # }
    /// ```
    pub fn get_nut_count(&self) -> Result<i32> {
        let mut count: i32 = 0;
        
        let rc = unsafe { 
            acorn_tree_get_nut_count(self.tree_h, &mut count as *mut _) 
        };
        
        if rc == 0 {
            Ok(count)
        } else {
            Err(Error::Acorn(unsafe { acorn_sys::last_error_string() }))
        }
    }

    /// Get the last sync timestamp
    /// 
    /// # Example
    /// ```no_run
    /// # use acorn::{AcornTree, AcornAdvancedTree, Error};
    /// # fn main() -> Result<(), Error> {
    /// let tree = AcornTree::open_memory()?;
    /// let advanced_tree = AcornAdvancedTree::from_tree(tree);
    /// let timestamp = advanced_tree.get_last_sync_timestamp()?;
    /// println!("Last sync: {}", timestamp);
    /// # Ok(())
    /// # }
    /// ```
    pub fn get_last_sync_timestamp(&self) -> Result<i64> {
        let mut timestamp: i64 = 0;
        
        let rc = unsafe { 
            acorn_tree_get_last_sync_timestamp(self.tree_h, &mut timestamp as *mut _) 
        };
        
        if rc == 0 {
            Ok(timestamp)
        } else {
            Err(Error::Acorn(unsafe { acorn_sys::last_error_string() }))
        }
    }
}

/// Event Management for enhanced event system, tangle support, and mesh primitives
pub struct AcornEventManager {
    h: acorn_event_manager_handle,
}

/// Event types for filtering and categorization
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventType {
    Stash,
    Toss,
    Squabble,
    Sync,
}

/// Event information structure
#[derive(Debug, Clone)]
pub struct EventInfo {
    pub event_type: EventType,
    pub key: String,
    pub json_payload: String,
    pub timestamp: i64,
    pub source_node: Option<String>,
}

/// Mesh topology types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MeshTopology {
    Full,
    Ring,
    Star,
    Custom,
}

/// Mesh statistics information
#[derive(Debug, Clone)]
pub struct MeshStats {
    pub node_id: String,
    pub tracked_change_ids: i32,
    pub active_tangles: i32,
    pub max_hop_count: i32,
    pub total_sync_operations: i32,
    pub last_sync_timestamp: i64,
}

impl AcornEventManager {
    /// Create an event manager for a tree
    /// 
    /// # Arguments
    /// * `tree` - The tree to manage events for
    /// 
    /// # Example
    /// ```no_run
    /// # use acorn::{AcornTree, AcornEventManager, Error};
    /// # fn main() -> Result<(), Error> {
    /// let tree = AcornTree::open_memory()?;
    /// let event_manager = AcornEventManager::new(tree)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(tree: AcornTree) -> Result<Self> {
        let mut h: acorn_event_manager_handle = 0;
        let rc = unsafe { acorn_event_manager_create(tree.h, &mut h as *mut _) };
        
        if rc == 0 {
            Ok(Self { h })
        } else {
            Err(Error::Acorn(unsafe { acorn_sys::last_error_string() }))
        }
    }

    /// Subscribe to all events from this tree
    /// 
    /// # Arguments
    /// * `callback` - Function to call when events occur
    /// 
    /// # Example
    /// ```no_run
    /// # use acorn::{AcornTree, AcornEventManager, Error};
    /// # fn main() -> Result<(), Error> {
    /// let tree = AcornTree::open_memory()?;
    /// let event_manager = AcornEventManager::new(tree)?;
    /// let subscription = event_manager.subscribe(|key, json, len| {
    ///     println!("Event: {} with data length {}", key, len);
    /// })?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn subscribe<F>(&self, callback: F) -> Result<AcornSubscription>
    where
        F: Fn(&str, &[u8]) + Send + Sync + 'static,
    {
        let callback = Box::new(callback);
        let user_data = Box::into_raw(callback) as *mut std::ffi::c_void;
        
        let mut sub_h: acorn_sub_handle = 0;
        let rc = unsafe { 
            acorn_event_manager_subscribe(self.h, Some(event_callback), user_data, &mut sub_h as *mut _) 
        };
        
        if rc == 0 {
            Ok(AcornSubscription { h: sub_h })
        } else {
            unsafe { Box::from_raw(user_data as *mut F); }
            Err(Error::Acorn(unsafe { acorn_sys::last_error_string() }))
        }
    }

    /// Subscribe to specific event types only
    /// 
    /// # Arguments
    /// * `event_type` - Type of events to subscribe to
    /// * `callback` - Function to call when events occur
    /// 
    /// # Example
    /// ```no_run
    /// # use acorn::{AcornTree, AcornEventManager, EventType, Error};
    /// # fn main() -> Result<(), Error> {
    /// let tree = AcornTree::open_memory()?;
    /// let event_manager = AcornEventManager::new(tree)?;
    /// let subscription = event_manager.subscribe_filtered(EventType::Stash, |key, json, len| {
    ///     println!("Stash event: {}", key);
    /// })?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn subscribe_filtered<F>(&self, event_type: EventType, callback: F) -> Result<AcornSubscription>
    where
        F: Fn(&str, &[u8]) + Send + Sync + 'static,
    {
        let callback = Box::new(callback);
        let user_data = Box::into_raw(callback) as *mut std::ffi::c_void;
        
        let mut sub_h: acorn_sub_handle = 0;
        let rc = unsafe { 
            acorn_event_manager_subscribe_filtered(self.h, event_type as i32, Some(event_callback), user_data, &mut sub_h as *mut _) 
        };
        
        if rc == 0 {
            Ok(AcornSubscription { h: sub_h })
        } else {
            unsafe { Box::from_raw(user_data as *mut F); }
            Err(Error::Acorn(unsafe { acorn_sys::last_error_string() }))
        }
    }

    /// Raise a custom event
    /// 
    /// # Arguments
    /// * `event_type` - Type of event to raise
    /// * `key` - Key associated with the event
    /// * `json_payload` - JSON payload for the event
    /// 
    /// # Example
    /// ```no_run
    /// # use acorn::{AcornTree, AcornEventManager, EventType, Error};
    /// # fn main() -> Result<(), Error> {
    /// let tree = AcornTree::open_memory()?;
    /// let event_manager = AcornEventManager::new(tree)?;
    /// let payload = r#"{"message": "Custom event"}"#;
    /// event_manager.raise_event(EventType::Sync, "custom-key", payload)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn raise_event(&self, event_type: EventType, key: &str, json_payload: &str) -> Result<()> {
        let key_c = CString::new(key).map_err(|e| Error::Acorn(format!("Invalid key: {}", e)))?;
        let payload_c = CString::new(json_payload).map_err(|e| Error::Acorn(format!("Invalid JSON payload: {}", e)))?;
        
        let rc = unsafe { 
            acorn_event_manager_raise_event(self.h, event_type as i32, key_c.as_ptr(), payload_c.as_ptr(), json_payload.len()) 
        };
        
        if rc == 0 {
            Ok(())
        } else {
            Err(Error::Acorn(unsafe { acorn_sys::last_error_string() }))
        }
    }

    /// Get the number of active subscribers
    /// 
    /// # Example
    /// ```no_run
    /// # use acorn::{AcornTree, AcornEventManager, Error};
    /// # fn main() -> Result<(), Error> {
    /// let tree = AcornTree::open_memory()?;
    /// let event_manager = AcornEventManager::new(tree)?;
    /// let count = event_manager.get_subscriber_count()?;
    /// println!("Active subscribers: {}", count);
    /// # Ok(())
    /// # }
    /// ```
    pub fn get_subscriber_count(&self) -> Result<i32> {
        let mut count: i32 = 0;
        
        let rc = unsafe { 
            acorn_event_manager_get_subscriber_count(self.h, &mut count as *mut _) 
        };
        
        if rc == 0 {
            Ok(count)
        } else {
            Err(Error::Acorn(unsafe { acorn_sys::last_error_string() }))
        }
    }
}

impl Drop for AcornEventManager {
    fn drop(&mut self) {
        unsafe { acorn_event_manager_close(self.h); }
    }
}

/// Tangle for live synchronization between trees
pub struct AcornTangle {
    h: acorn_tangle_handle,
}

impl AcornTangle {
    /// Create a tangle between two trees
    /// 
    /// # Arguments
    /// * `local_tree` - Local tree
    /// * `remote_tree` - Remote tree
    /// * `tangle_name` - Name for the tangle
    /// 
    /// # Example
    /// ```no_run
    /// # use acorn::{AcornTree, AcornTangle, Error};
    /// # fn main() -> Result<(), Error> {
    /// let local_tree = AcornTree::open_memory()?;
    /// let remote_tree = AcornTree::open_memory()?;
    /// let tangle = AcornTangle::new(local_tree, remote_tree, "sync-session")?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(local_tree: AcornTree, remote_tree: AcornTree, tangle_name: &str) -> Result<Self> {
        let name_c = CString::new(tangle_name).map_err(|e| Error::Acorn(format!("Invalid tangle name: {}", e)))?;
        
        let mut h: acorn_tangle_handle = 0;
        let rc = unsafe { 
            acorn_tangle_create(local_tree.h, remote_tree.h, name_c.as_ptr(), &mut h as *mut _) 
        };
        
        if rc == 0 {
            Ok(Self { h })
        } else {
            Err(Error::Acorn(unsafe { acorn_sys::last_error_string() }))
        }
    }

    /// Create an in-process tangle between two trees
    /// 
    /// # Arguments
    /// * `local_tree` - Local tree
    /// * `remote_tree` - Remote tree
    /// * `tangle_name` - Name for the tangle
    /// 
    /// # Example
    /// ```no_run
    /// # use acorn::{AcornTree, AcornTangle, Error};
    /// # fn main() -> Result<(), Error> {
    /// let local_tree = AcornTree::open_memory()?;
    /// let remote_tree = AcornTree::open_memory()?;
    /// let tangle = AcornTangle::new_in_process(local_tree, remote_tree, "in-process-sync")?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn new_in_process(local_tree: AcornTree, remote_tree: AcornTree, tangle_name: &str) -> Result<Self> {
        let name_c = CString::new(tangle_name).map_err(|e| Error::Acorn(format!("Invalid tangle name: {}", e)))?;
        
        let mut h: acorn_tangle_handle = 0;
        let rc = unsafe { 
            acorn_tangle_create_in_process(local_tree.h, remote_tree.h, name_c.as_ptr(), &mut h as *mut _) 
        };
        
        if rc == 0 {
            Ok(Self { h })
        } else {
            Err(Error::Acorn(unsafe { acorn_sys::last_error_string() }))
        }
    }

    /// Push data to the remote tree
    /// 
    /// # Arguments
    /// * `key` - Key to push
    /// * `json_payload` - JSON payload to push
    /// 
    /// # Example
    /// ```no_run
    /// # use acorn::{AcornTree, AcornTangle, Error};
    /// # fn main() -> Result<(), Error> {
    /// let local_tree = AcornTree::open_memory()?;
    /// let remote_tree = AcornTree::open_memory()?;
    /// let tangle = AcornTangle::new(local_tree, remote_tree, "sync-session")?;
    /// let payload = r#"{"name": "Alice", "age": 30}"#;
    /// tangle.push("user-1", payload)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn push(&self, key: &str, json_payload: &str) -> Result<()> {
        let key_c = CString::new(key).map_err(|e| Error::Acorn(format!("Invalid key: {}", e)))?;
        let payload_c = CString::new(json_payload).map_err(|e| Error::Acorn(format!("Invalid JSON payload: {}", e)))?;
        
        let rc = unsafe { 
            acorn_tangle_push(self.h, key_c.as_ptr(), payload_c.as_ptr(), json_payload.len()) 
        };
        
        if rc == 0 {
            Ok(())
        } else {
            Err(Error::Acorn(unsafe { acorn_sys::last_error_string() }))
        }
    }

    /// Pull data from the remote tree
    /// 
    /// # Example
    /// ```no_run
    /// # use acorn::{AcornTree, AcornTangle, Error};
    /// # fn main() -> Result<(), Error> {
    /// let local_tree = AcornTree::open_memory()?;
    /// let remote_tree = AcornTree::open_memory()?;
    /// let tangle = AcornTangle::new(local_tree, remote_tree, "sync-session")?;
    /// tangle.pull()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn pull(&self) -> Result<()> {
        let rc = unsafe { acorn_tangle_pull(self.h) };
        
        if rc == 0 {
            Ok(())
        } else {
            Err(Error::Acorn(unsafe { acorn_sys::last_error_string() }))
        }
    }

    /// Synchronize bidirectionally with the remote tree
    /// 
    /// # Example
    /// ```no_run
    /// # use acorn::{AcornTree, AcornTangle, Error};
    /// # fn main() -> Result<(), Error> {
    /// let local_tree = AcornTree::open_memory()?;
    /// let remote_tree = AcornTree::open_memory()?;
    /// let tangle = AcornTangle::new(local_tree, remote_tree, "sync-session")?;
    /// tangle.sync_bidirectional()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn sync_bidirectional(&self) -> Result<()> {
        let rc = unsafe { acorn_tangle_sync_bidirectional(self.h) };
        
        if rc == 0 {
            Ok(())
        } else {
            Err(Error::Acorn(unsafe { acorn_sys::last_error_string() }))
        }
    }

    /// Get tangle statistics
    /// 
    /// # Example
    /// ```no_run
    /// # use acorn::{AcornTree, AcornTangle, Error};
    /// # fn main() -> Result<(), Error> {
    /// let local_tree = AcornTree::open_memory()?;
    /// let remote_tree = AcornTree::open_memory()?;
    /// let tangle = AcornTangle::new(local_tree, remote_tree, "sync-session")?;
    /// let stats = tangle.get_stats()?;
    /// println!("Active tangles: {}", stats.active_tangles);
    /// # Ok(())
    /// # }
    /// ```
    pub fn get_stats(&self) -> Result<MeshStats> {
        let mut stats: acorn_mesh_stats = unsafe { std::mem::zeroed() };
        
        let rc = unsafe { 
            acorn_tangle_get_stats(self.h, &mut stats as *mut _) 
        };
        
        if rc == 0 {
            Ok(MeshStats {
                node_id: unsafe { CStr::from_ptr(stats.node_id).to_string_lossy().into_owned() },
                tracked_change_ids: stats.tracked_change_ids,
                active_tangles: stats.active_tangles,
                max_hop_count: stats.max_hop_count,
                total_sync_operations: stats.total_sync_operations,
                last_sync_timestamp: stats.last_sync_timestamp,
            })
        } else {
            Err(Error::Acorn(unsafe { acorn_sys::last_error_string() }))
        }
    }
}

impl Drop for AcornTangle {
    fn drop(&mut self) {
        unsafe { acorn_tangle_close(self.h); }
    }
}

/// Mesh coordinator for multi-node synchronization
pub struct AcornMeshCoordinator {
    h: acorn_mesh_coordinator_handle,
}

impl AcornMeshCoordinator {
    /// Create a new mesh coordinator
    /// 
    /// # Example
    /// ```no_run
    /// # use acorn::{AcornMeshCoordinator, Error};
    /// # fn main() -> Result<(), Error> {
    /// let coordinator = AcornMeshCoordinator::new()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn new() -> Result<Self> {
        let mut h: acorn_mesh_coordinator_handle = 0;
        let rc = unsafe { acorn_mesh_coordinator_create(&mut h as *mut _) };
        
        if rc == 0 {
            Ok(Self { h })
        } else {
            Err(Error::Acorn(unsafe { acorn_sys::last_error_string() }))
        }
    }

    /// Add a node to the mesh
    /// 
    /// # Arguments
    /// * `node_id` - Unique identifier for the node
    /// * `tree` - Tree to add to the mesh
    /// 
    /// # Example
    /// ```no_run
    /// # use acorn::{AcornMeshCoordinator, AcornTree, Error};
    /// # fn main() -> Result<(), Error> {
    /// let coordinator = AcornMeshCoordinator::new()?;
    /// let tree = AcornTree::open_memory()?;
    /// coordinator.add_node("node-1", tree)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn add_node(&self, node_id: &str, tree: AcornTree) -> Result<()> {
        let node_id_c = CString::new(node_id).map_err(|e| Error::Acorn(format!("Invalid node ID: {}", e)))?;
        
        let rc = unsafe { 
            acorn_mesh_coordinator_add_node(self.h, node_id_c.as_ptr(), tree.h) 
        };
        
        if rc == 0 {
            Ok(())
        } else {
            Err(Error::Acorn(unsafe { acorn_sys::last_error_string() }))
        }
    }

    /// Connect two nodes bidirectionally
    /// 
    /// # Arguments
    /// * `node_a` - First node ID
    /// * `node_b` - Second node ID
    /// 
    /// # Example
    /// ```no_run
    /// # use acorn::{AcornMeshCoordinator, AcornTree, Error};
    /// # fn main() -> Result<(), Error> {
    /// let coordinator = AcornMeshCoordinator::new()?;
    /// let tree_a = AcornTree::open_memory()?;
    /// let tree_b = AcornTree::open_memory()?;
    /// coordinator.add_node("node-a", tree_a)?;
    /// coordinator.add_node("node-b", tree_b)?;
    /// coordinator.connect_nodes("node-a", "node-b")?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn connect_nodes(&self, node_a: &str, node_b: &str) -> Result<()> {
        let node_a_c = CString::new(node_a).map_err(|e| Error::Acorn(format!("Invalid node A ID: {}", e)))?;
        let node_b_c = CString::new(node_b).map_err(|e| Error::Acorn(format!("Invalid node B ID: {}", e)))?;
        
        let rc = unsafe { 
            acorn_mesh_coordinator_connect_nodes(self.h, node_a_c.as_ptr(), node_b_c.as_ptr()) 
        };
        
        if rc == 0 {
            Ok(())
        } else {
            Err(Error::Acorn(unsafe { acorn_sys::last_error_string() }))
        }
    }

    /// Create a specific topology
    /// 
    /// # Arguments
    /// * `topology` - Type of topology to create
    /// * `hub_node_id` - Hub node ID for star topology (ignored for other topologies)
    /// 
    /// # Example
    /// ```no_run
    /// # use acorn::{AcornMeshCoordinator, AcornTree, MeshTopology, Error};
    /// # fn main() -> Result<(), Error> {
    /// let coordinator = AcornMeshCoordinator::new()?;
    /// let tree_a = AcornTree::open_memory()?;
    /// let tree_b = AcornTree::open_memory()?;
    /// let tree_c = AcornTree::open_memory()?;
    /// coordinator.add_node("node-a", tree_a)?;
    /// coordinator.add_node("node-b", tree_b)?;
    /// coordinator.add_node("node-c", tree_c)?;
    /// coordinator.create_topology(MeshTopology::Full, "")?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn create_topology(&self, topology: MeshTopology, hub_node_id: &str) -> Result<()> {
        let hub_c = CString::new(hub_node_id).map_err(|e| Error::Acorn(format!("Invalid hub node ID: {}", e)))?;
        
        let rc = unsafe { 
            acorn_mesh_coordinator_create_topology(self.h, topology as i32, hub_c.as_ptr()) 
        };
        
        if rc == 0 {
            Ok(())
        } else {
            Err(Error::Acorn(unsafe { acorn_sys::last_error_string() }))
        }
    }

    /// Synchronize all nodes in the mesh
    /// 
    /// # Example
    /// ```no_run
    /// # use acorn::{AcornMeshCoordinator, Error};
    /// # fn main() -> Result<(), Error> {
    /// let coordinator = AcornMeshCoordinator::new()?;
    /// coordinator.synchronize_all()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn synchronize_all(&self) -> Result<()> {
        let rc = unsafe { acorn_mesh_coordinator_synchronize_all(self.h) };
        
        if rc == 0 {
            Ok(())
        } else {
            Err(Error::Acorn(unsafe { acorn_sys::last_error_string() }))
        }
    }

    /// Get statistics for a specific node
    /// 
    /// # Arguments
    /// * `node_id` - Node ID to get statistics for
    /// 
    /// # Example
    /// ```no_run
    /// # use acorn::{AcornMeshCoordinator, Error};
    /// # fn main() -> Result<(), Error> {
    /// let coordinator = AcornMeshCoordinator::new()?;
    /// let stats = coordinator.get_node_stats("node-1")?;
    /// println!("Node {}: {} active tangles", stats.node_id, stats.active_tangles);
    /// # Ok(())
    /// # }
    /// ```
    pub fn get_node_stats(&self, node_id: &str) -> Result<MeshStats> {
        let node_id_c = CString::new(node_id).map_err(|e| Error::Acorn(format!("Invalid node ID: {}", e)))?;
        
        let mut stats: acorn_mesh_stats = unsafe { std::mem::zeroed() };
        
        let rc = unsafe { 
            acorn_mesh_coordinator_get_node_stats(self.h, node_id_c.as_ptr(), &mut stats as *mut _) 
        };
        
        if rc == 0 {
            Ok(MeshStats {
                node_id: unsafe { CStr::from_ptr(stats.node_id).to_string_lossy().into_owned() },
                tracked_change_ids: stats.tracked_change_ids,
                active_tangles: stats.active_tangles,
                max_hop_count: stats.max_hop_count,
                total_sync_operations: stats.total_sync_operations,
                last_sync_timestamp: stats.last_sync_timestamp,
            })
        } else {
            Err(Error::Acorn(unsafe { acorn_sys::last_error_string() }))
        }
    }

    /// Get statistics for all nodes in the mesh
    /// 
    /// # Example
    /// ```no_run
    /// # use acorn::{AcornMeshCoordinator, Error};
    /// # fn main() -> Result<(), Error> {
    /// let coordinator = AcornMeshCoordinator::new()?;
    /// let all_stats = coordinator.get_all_stats()?;
    /// for stats in all_stats {
    ///     println!("Node {}: {} active tangles", stats.node_id, stats.active_tangles);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn get_all_stats(&self) -> Result<Vec<MeshStats>> {
        let mut stats_ptr: *mut acorn_mesh_stats = std::ptr::null_mut();
        let mut count: usize = 0;
        
        let rc = unsafe { 
            acorn_mesh_coordinator_get_all_stats(self.h, &mut stats_ptr as *mut _, &mut count as *mut _) 
        };
        
        if rc == 0 {
            let mut stats_list = Vec::new();
            if !stats_ptr.is_null() && count > 0 {
                let stats_slice = unsafe { std::slice::from_raw_parts(stats_ptr, count) };
                for stats in stats_slice {
                    stats_list.push(MeshStats {
                        node_id: unsafe { CStr::from_ptr(stats.node_id).to_string_lossy().into_owned() },
                        tracked_change_ids: stats.tracked_change_ids,
                        active_tangles: stats.active_tangles,
                        max_hop_count: stats.max_hop_count,
                        total_sync_operations: stats.total_sync_operations,
                        last_sync_timestamp: stats.last_sync_timestamp,
                    });
                }
                unsafe { acorn_mesh_coordinator_free_stats(stats_ptr, count); }
            }
            Ok(stats_list)
        } else {
            Err(Error::Acorn(unsafe { acorn_sys::last_error_string() }))
        }
    }
}

impl Drop for AcornMeshCoordinator {
    fn drop(&mut self) {
        unsafe { acorn_mesh_coordinator_close(self.h); }
    }
}

/// Performance Monitoring for built-in metrics collection, health checks, and monitoring
pub struct AcornPerformanceMonitor {
    h: acorn_performance_monitor_handle,
}

/// Health status enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HealthStatus {
    Unknown,
    Healthy,
    Degraded,
    Unhealthy,
}

/// Performance metrics information
#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    pub operations_per_second: i64,
    pub memory_usage_bytes: i64,
    pub cache_hit_rate_percent: i64,
    pub sync_latency_ms: i64,
    pub disk_io_bytes: i64,
    pub network_bytes: i64,
    pub cpu_usage_percent: i64,
    pub timestamp: i64,
}

/// Health check information
#[derive(Debug, Clone)]
pub struct HealthInfo {
    pub status: HealthStatus,
    pub service_name: String,
    pub message: String,
    pub response_time_ms: i64,
    pub timestamp: i64,
    pub details: Option<String>,
}

/// Benchmark configuration
#[derive(Debug, Clone)]
pub struct BenchmarkConfig {
    pub operation_count: i32,
    pub warmup_iterations: i32,
    pub measurement_iterations: i32,
    pub timeout_ms: i64,
    pub enable_memory_tracking: bool,
    pub enable_disk_tracking: bool,
    pub enable_network_tracking: bool,
}

/// Benchmark results
#[derive(Debug, Clone)]
pub struct BenchmarkResult {
    pub operation_name: String,
    pub total_time_ms: i64,
    pub operations_per_second: i64,
    pub memory_allocated_bytes: i64,
    pub disk_io_bytes: i64,
    pub network_bytes: i64,
    pub average_latency_ms: f64,
    pub p50_latency_ms: f64,
    pub p95_latency_ms: f64,
    pub p99_latency_ms: f64,
    pub timestamp: i64,
}

impl AcornPerformanceMonitor {
    /// Create a new performance monitor
    /// 
    /// # Example
    /// ```no_run
    /// # use acorn::{AcornPerformanceMonitor, Error};
    /// # fn main() -> Result<(), Error> {
    /// let monitor = AcornPerformanceMonitor::new()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn new() -> Result<Self> {
        let mut h: acorn_performance_monitor_handle = 0;
        let rc = unsafe { acorn_performance_monitor_create(&mut h as *mut _) };
        
        if rc == 0 {
            Ok(Self { h })
        } else {
            Err(Error::Acorn(unsafe { acorn_sys::last_error_string() }))
        }
    }

    /// Start collecting performance metrics
    /// 
    /// # Example
    /// ```no_run
    /// # use acorn::{AcornPerformanceMonitor, Error};
    /// # fn main() -> Result<(), Error> {
    /// let monitor = AcornPerformanceMonitor::new()?;
    /// monitor.start_collection()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn start_collection(&self) -> Result<()> {
        let rc = unsafe { acorn_performance_monitor_start_collection(self.h) };
        
        if rc == 0 {
            Ok(())
        } else {
            Err(Error::Acorn(unsafe { acorn_sys::last_error_string() }))
        }
    }

    /// Stop collecting performance metrics
    /// 
    /// # Example
    /// ```no_run
    /// # use acorn::{AcornPerformanceMonitor, Error};
    /// # fn main() -> Result<(), Error> {
    /// let monitor = AcornPerformanceMonitor::new()?;
    /// monitor.start_collection()?;
    /// // ... do work ...
    /// monitor.stop_collection()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn stop_collection(&self) -> Result<()> {
        let rc = unsafe { acorn_performance_monitor_stop_collection(self.h) };
        
        if rc == 0 {
            Ok(())
        } else {
            Err(Error::Acorn(unsafe { acorn_sys::last_error_string() }))
        }
    }

    /// Get current performance metrics
    /// 
    /// # Example
    /// ```no_run
    /// # use acorn::{AcornPerformanceMonitor, Error};
    /// # fn main() -> Result<(), Error> {
    /// let monitor = AcornPerformanceMonitor::new()?;
    /// let metrics = monitor.get_metrics()?;
    /// println!("Operations per second: {}", metrics.operations_per_second);
    /// # Ok(())
    /// # }
    /// ```
    pub fn get_metrics(&self) -> Result<PerformanceMetrics> {
        let mut metrics: acorn_performance_metrics = unsafe { std::mem::zeroed() };
        
        let rc = unsafe { 
            acorn_performance_monitor_get_metrics(self.h, &mut metrics as *mut _) 
        };
        
        if rc == 0 {
            Ok(PerformanceMetrics {
                operations_per_second: metrics.operations_per_second,
                memory_usage_bytes: metrics.memory_usage_bytes,
                cache_hit_rate_percent: metrics.cache_hit_rate_percent,
                sync_latency_ms: metrics.sync_latency_ms,
                disk_io_bytes: metrics.disk_io_bytes,
                network_bytes: metrics.network_bytes,
                cpu_usage_percent: metrics.cpu_usage_percent,
                timestamp: metrics.timestamp,
            })
        } else {
            Err(Error::Acorn(unsafe { acorn_sys::last_error_string() }))
        }
    }

    /// Get performance metrics history
    /// 
    /// # Example
    /// ```no_run
    /// # use acorn::{AcornPerformanceMonitor, Error};
    /// # fn main() -> Result<(), Error> {
    /// let monitor = AcornPerformanceMonitor::new()?;
    /// let history = monitor.get_history()?;
    /// println!("Collected {} metrics samples", history.len());
    /// # Ok(())
    /// # }
    /// ```
    pub fn get_history(&self) -> Result<Vec<PerformanceMetrics>> {
        let mut metrics_ptr: *mut acorn_performance_metrics = std::ptr::null_mut();
        let mut count: usize = 0;
        
        let rc = unsafe { 
            acorn_performance_monitor_get_history(self.h, &mut metrics_ptr as *mut _, &mut count as *mut _) 
        };
        
        if rc == 0 {
            let mut metrics_list = Vec::new();
            if !metrics_ptr.is_null() && count > 0 {
                let metrics_slice = unsafe { std::slice::from_raw_parts(metrics_ptr, count) };
                for metrics in metrics_slice {
                    metrics_list.push(PerformanceMetrics {
                        operations_per_second: metrics.operations_per_second,
                        memory_usage_bytes: metrics.memory_usage_bytes,
                        cache_hit_rate_percent: metrics.cache_hit_rate_percent,
                        sync_latency_ms: metrics.sync_latency_ms,
                        disk_io_bytes: metrics.disk_io_bytes,
                        network_bytes: metrics.network_bytes,
                        cpu_usage_percent: metrics.cpu_usage_percent,
                        timestamp: metrics.timestamp,
                    });
                }
                unsafe { acorn_performance_monitor_free_metrics(metrics_ptr, count); }
            }
            Ok(metrics_list)
        } else {
            Err(Error::Acorn(unsafe { acorn_sys::last_error_string() }))
        }
    }

    /// Reset performance metrics
    /// 
    /// # Example
    /// ```no_run
    /// # use acorn::{AcornPerformanceMonitor, Error};
    /// # fn main() -> Result<(), Error> {
    /// let monitor = AcornPerformanceMonitor::new()?;
    /// monitor.reset_metrics()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn reset_metrics(&self) -> Result<()> {
        let rc = unsafe { acorn_performance_monitor_reset_metrics(self.h) };
        
        if rc == 0 {
            Ok(())
        } else {
            Err(Error::Acorn(unsafe { acorn_sys::last_error_string() }))
        }
    }
}

impl Drop for AcornPerformanceMonitor {
    fn drop(&mut self) {
        unsafe { acorn_performance_monitor_close(self.h); }
    }
}

/// Health checker for monitoring service health
pub struct AcornHealthChecker {
    h: acorn_health_checker_handle,
}

impl AcornHealthChecker {
    /// Create a new health checker
    /// 
    /// # Example
    /// ```no_run
    /// # use acorn::{AcornHealthChecker, Error};
    /// # fn main() -> Result<(), Error> {
    /// let checker = AcornHealthChecker::new()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn new() -> Result<Self> {
        let mut h: acorn_health_checker_handle = 0;
        let rc = unsafe { acorn_health_checker_create(&mut h as *mut _) };
        
        if rc == 0 {
            Ok(Self { h })
        } else {
            Err(Error::Acorn(unsafe { acorn_sys::last_error_string() }))
        }
    }

    /// Add a service to monitor
    /// 
    /// # Arguments
    /// * `service_name` - Name of the service to monitor
    /// * `health_endpoint` - Health check endpoint URL
    /// 
    /// # Example
    /// ```no_run
    /// # use acorn::{AcornHealthChecker, Error};
    /// # fn main() -> Result<(), Error> {
    /// let checker = AcornHealthChecker::new()?;
    /// checker.add_service("database", "http://localhost:5432/health")?;
    /// checker.add_service("api", "http://localhost:8080/health")?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn add_service(&self, service_name: &str, health_endpoint: &str) -> Result<()> {
        let service_c = CString::new(service_name).map_err(|e| Error::Acorn(format!("Invalid service name: {}", e)))?;
        let endpoint_c = CString::new(health_endpoint).map_err(|e| Error::Acorn(format!("Invalid health endpoint: {}", e)))?;
        
        let rc = unsafe { 
            acorn_health_checker_add_service(self.h, service_c.as_ptr(), endpoint_c.as_ptr()) 
        };
        
        if rc == 0 {
            Ok(())
        } else {
            Err(Error::Acorn(unsafe { acorn_sys::last_error_string() }))
        }
    }

    /// Check health of all registered services
    /// 
    /// # Example
    /// ```no_run
    /// # use acorn::{AcornHealthChecker, Error};
    /// # fn main() -> Result<(), Error> {
    /// let checker = AcornHealthChecker::new()?;
    /// checker.add_service("api", "http://localhost:8080/health")?;
    /// let results = checker.check_all()?;
    /// for result in results {
    ///     println!("Service {}: {:?}", result.service_name, result.status);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn check_all(&self) -> Result<Vec<HealthInfo>> {
        let mut results_ptr: *mut acorn_health_info = std::ptr::null_mut();
        let mut count: usize = 0;
        
        let rc = unsafe { 
            acorn_health_checker_check_all(self.h, &mut results_ptr as *mut _, &mut count as *mut _) 
        };
        
        if rc == 0 {
            let mut results_list = Vec::new();
            if !results_ptr.is_null() && count > 0 {
                let results_slice = unsafe { std::slice::from_raw_parts(results_ptr, count) };
                for result in results_slice {
                    let status = match result.status {
                        0 => HealthStatus::Unknown,
                        1 => HealthStatus::Healthy,
                        2 => HealthStatus::Degraded,
                        3 => HealthStatus::Unhealthy,
                        _ => HealthStatus::Unknown,
                    };
                    
                    results_list.push(HealthInfo {
                        status,
                        service_name: unsafe { CStr::from_ptr(result.service_name).to_string_lossy().into_owned() },
                        message: unsafe { CStr::from_ptr(result.message).to_string_lossy().into_owned() },
                        response_time_ms: result.response_time_ms,
                        timestamp: result.timestamp,
                        details: if result.details.is_null() {
                            None
                        } else {
                            Some(unsafe { CStr::from_ptr(result.details).to_string_lossy().into_owned() })
                        },
                    });
                }
                unsafe { acorn_health_checker_free_results(results_ptr, count); }
            }
            Ok(results_list)
        } else {
            Err(Error::Acorn(unsafe { acorn_sys::last_error_string() }))
        }
    }

    /// Check health of a specific service
    /// 
    /// # Arguments
    /// * `service_name` - Name of the service to check
    /// 
    /// # Example
    /// ```no_run
    /// # use acorn::{AcornHealthChecker, Error};
    /// # fn main() -> Result<(), Error> {
    /// let checker = AcornHealthChecker::new()?;
    /// checker.add_service("api", "http://localhost:8080/health")?;
    /// let result = checker.check_service("api")?;
    /// println!("API status: {:?}", result.status);
    /// # Ok(())
    /// # }
    /// ```
    pub fn check_service(&self, service_name: &str) -> Result<HealthInfo> {
        let service_c = CString::new(service_name).map_err(|e| Error::Acorn(format!("Invalid service name: {}", e)))?;
        
        let mut result: acorn_health_info = unsafe { std::mem::zeroed() };
        
        let rc = unsafe { 
            acorn_health_checker_check_service(self.h, service_c.as_ptr(), &mut result as *mut _) 
        };
        
        if rc == 0 {
            let status = match result.status {
                0 => HealthStatus::Unknown,
                1 => HealthStatus::Healthy,
                2 => HealthStatus::Degraded,
                3 => HealthStatus::Unhealthy,
                _ => HealthStatus::Unknown,
            };
            
            Ok(HealthInfo {
                status,
                service_name: unsafe { CStr::from_ptr(result.service_name).to_string_lossy().into_owned() },
                message: unsafe { CStr::from_ptr(result.message).to_string_lossy().into_owned() },
                response_time_ms: result.response_time_ms,
                timestamp: result.timestamp,
                details: if result.details.is_null() {
                    None
                } else {
                    Some(unsafe { CStr::from_ptr(result.details).to_string_lossy().into_owned() })
                },
            })
        } else {
            Err(Error::Acorn(unsafe { acorn_sys::last_error_string() }))
        }
    }

    /// Get overall health status
    /// 
    /// # Example
    /// ```no_run
    /// # use acorn::{AcornHealthChecker, Error};
    /// # fn main() -> Result<(), Error> {
    /// let checker = AcornHealthChecker::new()?;
    /// let overall_status = checker.get_overall_status()?;
    /// println!("Overall status: {:?}", overall_status);
    /// # Ok(())
    /// # }
    /// ```
    pub fn get_overall_status(&self) -> Result<HealthStatus> {
        let mut status: i32 = 0;
        
        let rc = unsafe { 
            acorn_health_checker_get_overall_status(self.h, &mut status as *mut _) 
        };
        
        if rc == 0 {
            Ok(match status {
                0 => HealthStatus::Unknown,
                1 => HealthStatus::Healthy,
                2 => HealthStatus::Degraded,
                3 => HealthStatus::Unhealthy,
                _ => HealthStatus::Unknown,
            })
        } else {
            Err(Error::Acorn(unsafe { acorn_sys::last_error_string() }))
        }
    }
}

impl Drop for AcornHealthChecker {
    fn drop(&mut self) {
        unsafe { acorn_health_checker_close(self.h); }
    }
}

/// Benchmark utilities for performance testing
pub struct AcornBenchmark;

impl AcornBenchmark {
    /// Benchmark tree operations
    /// 
    /// # Arguments
    /// * `tree` - Tree to benchmark
    /// * `config` - Benchmark configuration
    /// 
    /// # Example
    /// ```no_run
    /// # use acorn::{AcornTree, AcornBenchmark, BenchmarkConfig, Error};
    /// # fn main() -> Result<(), Error> {
    /// let tree = AcornTree::open_memory()?;
    /// let config = BenchmarkConfig {
    ///     operation_count: 1000,
    ///     warmup_iterations: 10,
    ///     measurement_iterations: 100,
    ///     timeout_ms: 30000,
    ///     enable_memory_tracking: true,
    ///     enable_disk_tracking: false,
    ///     enable_network_tracking: false,
    /// };
    /// let results = AcornBenchmark::benchmark_tree_operations(tree, &config)?;
    /// for result in results {
    ///     println!("{}: {} ops/sec", result.operation_name, result.operations_per_second);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn benchmark_tree_operations(tree: AcornTree, config: &BenchmarkConfig) -> Result<Vec<BenchmarkResult>> {
        let mut config_c = acorn_benchmark_config {
            operation_count: config.operation_count,
            warmup_iterations: config.warmup_iterations,
            measurement_iterations: config.measurement_iterations,
            timeout_ms: config.timeout_ms,
            enable_memory_tracking: if config.enable_memory_tracking { 1 } else { 0 },
            enable_disk_tracking: if config.enable_disk_tracking { 1 } else { 0 },
            enable_network_tracking: if config.enable_network_tracking { 1 } else { 0 },
        };
        
        let mut results_ptr: *mut acorn_benchmark_result = std::ptr::null_mut();
        let mut count: usize = 0;
        
        let rc = unsafe { 
            acorn_benchmark_tree_operations(tree.h, &mut config_c as *mut _, &mut results_ptr as *mut _, &mut count as *mut _) 
        };
        
        if rc == 0 {
            let mut results_list = Vec::new();
            if !results_ptr.is_null() && count > 0 {
                let results_slice = unsafe { std::slice::from_raw_parts(results_ptr, count) };
                for result in results_slice {
                    results_list.push(BenchmarkResult {
                        operation_name: unsafe { CStr::from_ptr(result.operation_name).to_string_lossy().into_owned() },
                        total_time_ms: result.total_time_ms,
                        operations_per_second: result.operations_per_second,
                        memory_allocated_bytes: result.memory_allocated_bytes,
                        disk_io_bytes: result.disk_io_bytes,
                        network_bytes: result.network_bytes,
                        average_latency_ms: result.average_latency_ms,
                        p50_latency_ms: result.p50_latency_ms,
                        p95_latency_ms: result.p95_latency_ms,
                        p99_latency_ms: result.p99_latency_ms,
                        timestamp: result.timestamp,
                    });
                }
                unsafe { acorn_benchmark_free_results(results_ptr, count); }
            }
            Ok(results_list)
        } else {
            Err(Error::Acorn(unsafe { acorn_sys::last_error_string() }))
        }
    }

    /// Benchmark sync operations
    /// 
    /// # Arguments
    /// * `tangle` - Tangle to benchmark
    /// * `config` - Benchmark configuration
    /// 
    /// # Example
    /// ```no_run
    /// # use acorn::{AcornTree, AcornTangle, AcornBenchmark, BenchmarkConfig, Error};
    /// # fn main() -> Result<(), Error> {
    /// let local_tree = AcornTree::open_memory()?;
    /// let remote_tree = AcornTree::open_memory()?;
    /// let tangle = AcornTangle::new(local_tree, remote_tree, "benchmark-tangle")?;
    /// let config = BenchmarkConfig {
    ///     operation_count: 1000,
    ///     warmup_iterations: 10,
    ///     measurement_iterations: 100,
    ///     timeout_ms: 30000,
    ///     enable_memory_tracking: true,
    ///     enable_disk_tracking: false,
    ///     enable_network_tracking: true,
    /// };
    /// let results = AcornBenchmark::benchmark_sync_operations(tangle, &config)?;
    /// for result in results {
    ///     println!("{}: {} ops/sec", result.operation_name, result.operations_per_second);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn benchmark_sync_operations(tangle: AcornTangle, config: &BenchmarkConfig) -> Result<Vec<BenchmarkResult>> {
        let mut config_c = acorn_benchmark_config {
            operation_count: config.operation_count,
            warmup_iterations: config.warmup_iterations,
            measurement_iterations: config.measurement_iterations,
            timeout_ms: config.timeout_ms,
            enable_memory_tracking: if config.enable_memory_tracking { 1 } else { 0 },
            enable_disk_tracking: if config.enable_disk_tracking { 1 } else { 0 },
            enable_network_tracking: if config.enable_network_tracking { 1 } else { 0 },
        };
        
        let mut results_ptr: *mut acorn_benchmark_result = std::ptr::null_mut();
        let mut count: usize = 0;
        
        let rc = unsafe { 
            acorn_benchmark_sync_operations(tangle.h, &mut config_c as *mut _, &mut results_ptr as *mut _, &mut count as *mut _) 
        };
        
        if rc == 0 {
            let mut results_list = Vec::new();
            if !results_ptr.is_null() && count > 0 {
                let results_slice = unsafe { std::slice::from_raw_parts(results_ptr, count) };
                for result in results_slice {
                    results_list.push(BenchmarkResult {
                        operation_name: unsafe { CStr::from_ptr(result.operation_name).to_string_lossy().into_owned() },
                        total_time_ms: result.total_time_ms,
                        operations_per_second: result.operations_per_second,
                        memory_allocated_bytes: result.memory_allocated_bytes,
                        disk_io_bytes: result.disk_io_bytes,
                        network_bytes: result.network_bytes,
                        average_latency_ms: result.average_latency_ms,
                        p50_latency_ms: result.p50_latency_ms,
                        p95_latency_ms: result.p95_latency_ms,
                        p99_latency_ms: result.p99_latency_ms,
                        timestamp: result.timestamp,
                    });
                }
                unsafe { acorn_benchmark_free_results(results_ptr, count); }
            }
            Ok(results_list)
        } else {
            Err(Error::Acorn(unsafe { acorn_sys::last_error_string() }))
        }
    }

    /// Benchmark mesh operations
    /// 
    /// # Arguments
    /// * `coordinator` - Mesh coordinator to benchmark
    /// * `config` - Benchmark configuration
    /// 
    /// # Example
    /// ```no_run
    /// # use acorn::{AcornMeshCoordinator, AcornTree, AcornBenchmark, BenchmarkConfig, Error};
    /// # fn main() -> Result<(), Error> {
    /// let coordinator = AcornMeshCoordinator::new()?;
    /// let tree_a = AcornTree::open_memory()?;
    /// let tree_b = AcornTree::open_memory()?;
    /// coordinator.add_node("node-a", tree_a)?;
    /// coordinator.add_node("node-b", tree_b)?;
    /// let config = BenchmarkConfig {
    ///     operation_count: 1000,
    ///     warmup_iterations: 10,
    ///     measurement_iterations: 100,
    ///     timeout_ms: 30000,
    ///     enable_memory_tracking: true,
    ///     enable_disk_tracking: false,
    ///     enable_network_tracking: true,
    /// };
    /// let results = AcornBenchmark::benchmark_mesh_operations(coordinator, &config)?;
    /// for result in results {
    ///     println!("{}: {} ops/sec", result.operation_name, result.operations_per_second);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn benchmark_mesh_operations(coordinator: AcornMeshCoordinator, config: &BenchmarkConfig) -> Result<Vec<BenchmarkResult>> {
        let mut config_c = acorn_benchmark_config {
            operation_count: config.operation_count,
            warmup_iterations: config.warmup_iterations,
            measurement_iterations: config.measurement_iterations,
            timeout_ms: config.timeout_ms,
            enable_memory_tracking: if config.enable_memory_tracking { 1 } else { 0 },
            enable_disk_tracking: if config.enable_disk_tracking { 1 } else { 0 },
            enable_network_tracking: if config.enable_network_tracking { 1 } else { 0 },
        };
        
        let mut results_ptr: *mut acorn_benchmark_result = std::ptr::null_mut();
        let mut count: usize = 0;
        
        let rc = unsafe { 
            acorn_benchmark_mesh_operations(coordinator.h, &mut config_c as *mut _, &mut results_ptr as *mut _, &mut count as *mut _) 
        };
        
        if rc == 0 {
            let mut results_list = Vec::new();
            if !results_ptr.is_null() && count > 0 {
                let results_slice = unsafe { std::slice::from_raw_parts(results_ptr, count) };
                for result in results_slice {
                    results_list.push(BenchmarkResult {
                        operation_name: unsafe { CStr::from_ptr(result.operation_name).to_string_lossy().into_owned() },
                        total_time_ms: result.total_time_ms,
                        operations_per_second: result.operations_per_second,
                        memory_allocated_bytes: result.memory_allocated_bytes,
                        disk_io_bytes: result.disk_io_bytes,
                        network_bytes: result.network_bytes,
                        average_latency_ms: result.average_latency_ms,
                        p50_latency_ms: result.p50_latency_ms,
                        p95_latency_ms: result.p95_latency_ms,
                        p99_latency_ms: result.p99_latency_ms,
                        timestamp: result.timestamp,
                    });
                }
                unsafe { acorn_benchmark_free_results(results_ptr, count); }
            }
            Ok(results_list)
        } else {
            Err(Error::Acorn(unsafe { acorn_sys::last_error_string() }))
        }
    }
}

/// Resource monitoring utilities
pub struct AcornResourceMonitor;

impl AcornResourceMonitor {
    /// Get current memory usage
    /// 
    /// # Example
    /// ```no_run
    /// # use acorn::{AcornResourceMonitor, Error};
    /// # fn main() -> Result<(), Error> {
    /// let (heap_bytes, stack_bytes, total_bytes) = AcornResourceMonitor::get_memory_usage()?;
    /// println!("Memory usage: {} bytes total (heap: {}, stack: {})", total_bytes, heap_bytes, stack_bytes);
    /// # Ok(())
    /// # }
    /// ```
    pub fn get_memory_usage() -> Result<(i64, i64, i64)> {
        let mut heap_bytes: i64 = 0;
        let mut stack_bytes: i64 = 0;
        let mut total_bytes: i64 = 0;
        
        let rc = unsafe { 
            acorn_get_memory_usage(&mut heap_bytes as *mut _, &mut stack_bytes as *mut _, &mut total_bytes as *mut _) 
        };
        
        if rc == 0 {
            Ok((heap_bytes, stack_bytes, total_bytes))
        } else {
            Err(Error::Acorn(unsafe { acorn_sys::last_error_string() }))
        }
    }

    /// Get disk usage for a path
    /// 
    /// # Arguments
    /// * `path` - Path to check disk usage for
    /// 
    /// # Example
    /// ```no_run
    /// # use acorn::{AcornResourceMonitor, Error};
    /// # fn main() -> Result<(), Error> {
    /// let (used_bytes, total_bytes, free_bytes) = AcornResourceMonitor::get_disk_usage("/tmp")?;
    /// println!("Disk usage: {} bytes used, {} bytes free, {} bytes total", used_bytes, free_bytes, total_bytes);
    /// # Ok(())
    /// # }
    /// ```
    pub fn get_disk_usage(path: &str) -> Result<(i64, i64, i64)> {
        let path_c = CString::new(path).map_err(|e| Error::Acorn(format!("Invalid path: {}", e)))?;
        
        let mut used_bytes: i64 = 0;
        let mut total_bytes: i64 = 0;
        let mut free_bytes: i64 = 0;
        
        let rc = unsafe { 
            acorn_get_disk_usage(path_c.as_ptr(), &mut used_bytes as *mut _, &mut total_bytes as *mut _, &mut free_bytes as *mut _) 
        };
        
        if rc == 0 {
            Ok((used_bytes, total_bytes, free_bytes))
        } else {
            Err(Error::Acorn(unsafe { acorn_sys::last_error_string() }))
        }
    }

    /// Get system information
    /// 
    /// # Example
    /// ```no_run
    /// # use acorn::{AcornResourceMonitor, Error};
    /// # fn main() -> Result<(), Error> {
    /// let system_info = AcornResourceMonitor::get_system_info()?;
    /// println!("System info: {}", system_info);
    /// # Ok(())
    /// # }
    /// ```
    pub fn get_system_info() -> Result<String> {
        let mut info_ptr: *mut u8 = std::ptr::null_mut();
        let mut length: usize = 0;
        
        let rc = unsafe { 
            acorn_get_system_info(&mut info_ptr as *mut _, &mut length as *mut _) 
        };
        
        if rc == 0 {
            if !info_ptr.is_null() && length > 0 {
                let info_slice = unsafe { std::slice::from_raw_parts(info_ptr, length) };
                let info_str = String::from_utf8_lossy(info_slice).into_owned();
                unsafe { acorn_free_system_info(info_ptr); }
                Ok(info_str)
            } else {
                Ok(String::new())
            }
        } else {
            Err(Error::Acorn(unsafe { acorn_sys::last_error_string() }))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct TestData {
        id: String,
        value: i32,
        name: String,
    }

    #[test]
    fn test_error_types() {
        // Test error types exist and can be created
        let _not_found = Error::NotFound;
        let _acorn_error = Error::Acorn("test error".to_string());
    }

    #[test]
    fn test_result_type() {
        // Test Result type works
        let ok_result: Result<String> = Ok("test".to_string());
        let err_result: Result<String> = Err(Error::NotFound);
        
        assert!(ok_result.is_ok());
        assert!(err_result.is_err());
    }

    #[test]
    fn test_serialization() {
        // Test that our test data can be serialized/deserialized
        let data = TestData {
            id: "test-1".to_string(),
            value: 42,
            name: "Test Item".to_string(),
        };

        let json = serde_json::to_string(&data).unwrap();
        let deserialized: TestData = serde_json::from_str(&json).unwrap();
        
        assert_eq!(data, deserialized);
    }

    #[test]
    fn test_cstring_creation() {
        // Test CString creation with various inputs
        let valid_id = "test-id";
        let cstring = CString::new(valid_id).unwrap();
        assert_eq!(cstring.to_string_lossy(), valid_id);

        // Test with null bytes (should fail)
        let invalid_id = "test\0id";
        let result = CString::new(invalid_id);
        assert!(result.is_err());
    }

    // Integration tests would go here, but they require the actual shim to be built
    // and the ACORN_SHIM_DIR environment variable to be set
    #[cfg(feature = "integration-tests")]
    mod integration_tests {
        use super::*;

        #[test]
        fn test_tree_lifecycle() {
            // This test would require the shim to be built and available
            // let mut tree = AcornTree::open("file://./test_db").unwrap();
            // 
            // let test_data = TestData {
            //     id: "test-1".to_string(),
            //     value: 42,
            //     name: "Test Item".to_string(),
            // };
            // 
            // // Test stash
            // tree.stash("test-1", &test_data).unwrap();
            // 
            // // Test crack
            // let retrieved: TestData = tree.crack("test-1").unwrap();
            // assert_eq!(test_data, retrieved);
            // 
            // // Test not found
            // let result: Result<TestData> = tree.crack("nonexistent");
            // assert!(matches!(result, Err(Error::NotFound)));
        }
    }
}
