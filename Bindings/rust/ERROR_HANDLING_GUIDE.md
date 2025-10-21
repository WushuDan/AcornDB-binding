# 🔍 AcornDB Rust Bindings Error Handling Guide

This guide provides comprehensive information about error handling in the AcornDB Rust bindings, including error types, context, recovery strategies, and best practices.

## 📋 Error Types Overview

The AcornDB Rust bindings provide a comprehensive error handling system with detailed error types, context information, and recovery strategies.

### **Core Error Types**

| Error Type | Description | Severity | Recoverable |
|------------|-------------|----------|-------------|
| `Acorn` | Internal AcornDB errors with context | Error | Depends |
| `NotFound` | Item not found in tree | Info | Yes |
| `Serialization` | JSON serialization/deserialization errors | Error | Yes |
| `InvalidInput` | Invalid input parameters | Error | Yes |
| `FileSystem` | File system related errors | Error | Depends |
| `Network` | Network related errors | Warning | Yes |
| `Encryption` | Encryption/decryption errors | Error | Depends |
| `Compression` | Compression/decompression errors | Error | Depends |
| `Cache` | Cache related errors | Error | Yes |
| `Query` | Query related errors | Error | Yes |
| `Transaction` | Transaction related errors | Error | Depends |
| `Sync` | Synchronization errors | Error | Yes |
| `Configuration` | Configuration errors | Error | Yes |
| `ResourceExhausted` | Resource exhaustion errors | Error | Yes |
| `Timeout` | Operation timeout errors | Warning | Yes |
| `ConcurrentAccess` | Concurrent access errors | Warning | Yes |

## 🎯 Error Creation and Context

### **Creating Errors with Context**

```rust
use acorn::{Error, Result};

// Basic error creation
let error = Error::acorn("Database connection failed");

// Error with context and operation
let error = Error::acorn_with_context(
    "Failed to open database file",
    "File does not exist or insufficient permissions",
    "open_tree"
);

// Specific error types
let not_found = Error::not_found("user-123", "crack");
let serialization = Error::serialization(
    "Invalid JSON format",
    "User",
    "deserialize"
);
let invalid_input = Error::invalid_input(
    "Key cannot be empty",
    "key",
    "non-empty string"
);
```

### **Error Context Helper Trait**

```rust
use acorn::{ErrorContext, Result};

// Add context to any Result
let result: Result<String> = some_operation()
    .with_context(|| "Failed to process user data".to_string());

// Add operation context
let result: Result<String> = some_operation()
    .with_operation("user_processing");
```

## 🔧 Error Handling Patterns

### **1. Basic Error Handling**

```rust
use acorn::{AcornTree, Error, Result};

fn basic_error_handling() -> Result<()> {
    let tree = AcornTree::open_memory()?;
    
    // Handle specific error types
    match tree.crack("nonexistent-key") {
        Ok(value) => println!("Found: {}", value),
        Err(Error::NotFound { key, operation }) => {
            println!("Key '{}' not found during {}", key, operation);
            // Handle not found case
        },
        Err(e) => {
            println!("Other error: {}", e);
            return Err(e);
        }
    }
    
    Ok(())
}
```

### **2. Error Recovery Strategies**

```rust
use acorn::{AcornTree, Error, Result};
use std::time::Duration;
use std::thread;

fn resilient_operation() -> Result<String> {
    let tree = AcornTree::open_memory()?;
    let mut attempts = 0;
    let max_attempts = 3;
    
    loop {
        match tree.crack("important-key") {
            Ok(value) => return Ok(value),
            Err(Error::NotFound { .. }) => {
                // Not found is not recoverable
                return Err(Error::not_found("important-key", "crack"));
            },
            Err(Error::Timeout { operation, duration_ms, .. }) => {
                attempts += 1;
                if attempts >= max_attempts {
                    return Err(Error::timeout(
                        "Max retry attempts exceeded",
                        operation,
                        duration_ms
                    ));
                }
                
                // Wait and retry
                thread::sleep(Duration::from_millis(100 * attempts));
                continue;
            },
            Err(Error::Network { .. }) => {
                attempts += 1;
                if attempts >= max_attempts {
                    return Err(Error::network(
                        "Network retry attempts exceeded",
                        "sync",
                        "sync_operation"
                    ));
                }
                
                // Wait and retry
                thread::sleep(Duration::from_millis(500 * attempts));
                continue;
            },
            Err(e) => return Err(e),
        }
    }
}
```

### **3. Error Propagation with Context**

```rust
use acorn::{AcornTree, Error, Result, ErrorContext};

fn process_user_data(user_id: &str) -> Result<User> {
    let tree = AcornTree::open_memory()?;
    
    // Get user data with context
    let user_json = tree.crack(user_id)
        .with_context(|| format!("Failed to retrieve user data for ID: {}", user_id))?;
    
    // Deserialize with context
    let user: User = serde_json::from_str(&user_json)
        .map_err(|e| Error::serialization(
            e.to_string(),
            "User",
            "deserialize"
        ))?;
    
    // Validate user data
    if user.email.is_empty() {
        return Err(Error::invalid_input(
            "User email cannot be empty",
            "email",
            "non-empty string"
        ));
    }
    
    Ok(user)
}
```

### **4. Batch Error Handling**

```rust
use acorn::{AcornTree, AcornBatch, Error, Result};

fn batch_operation_with_error_handling() -> Result<()> {
    let tree = AcornTree::open_memory()?;
    let mut batch = AcornBatch::new(tree)?;
    
    let items = vec![
        ("key1", r#"{"id": "1", "name": "Item 1"}"#),
        ("key2", r#"{"id": "2", "name": "Item 2"}"#),
        ("key3", r#"{"id": "3", "name": "Item 3"}"#),
    ];
    
    // Collect errors instead of failing fast
    let mut errors = Vec::new();
    
    for (key, value) in items {
        match batch.stash(key, value) {
            Ok(_) => {},
            Err(e) => {
                errors.push((key, e));
            }
        }
    }
    
    if !errors.is_empty() {
        // Log errors but continue
        for (key, error) in &errors {
            eprintln!("Failed to stash {}: {}", key, error);
        }
        
        // Decide whether to commit partial batch or fail
        if errors.len() == items.len() {
            return Err(Error::transaction(
                "All batch operations failed",
                "batch_stash"
            ));
        }
    }
    
    // Commit the batch
    batch.commit()
        .with_context(|| "Failed to commit batch operations")?;
    
    Ok(())
}
```

## 🚨 Error Severity and Logging

### **Error Severity Levels**

```rust
use acorn::{Error, ErrorSeverity};

fn handle_error_by_severity(error: &Error) {
    match error.severity() {
        ErrorSeverity::Info => {
            // Log as info (e.g., NotFound)
            println!("INFO: {}", error);
        },
        ErrorSeverity::Warning => {
            // Log as warning (e.g., Timeout, Network)
            eprintln!("WARNING: {}", error);
        },
        ErrorSeverity::Error => {
            // Log as error (e.g., Serialization, InvalidInput)
            eprintln!("ERROR: {}", error);
        },
        ErrorSeverity::Critical => {
            // Log as critical and potentially panic
            eprintln!("CRITICAL: {}", error);
            panic!("Critical error occurred: {}", error);
        }
    }
}
```

### **Structured Error Logging**

```rust
use acorn::{Error, Result};
use serde_json;

fn log_error_structured(error: &Error) {
    let error_info = serde_json::json!({
        "error_type": format!("{:?}", error),
        "message": error.to_string(),
        "operation": error.operation(),
        "context": error.context(),
        "severity": error.severity().to_string(),
        "recoverable": error.is_recoverable(),
        "timestamp": chrono::Utc::now().to_rfc3339(),
    });
    
    println!("{}", serde_json::to_string_pretty(&error_info).unwrap());
}
```

## 🔄 Error Recovery Strategies

### **1. Retry with Exponential Backoff**

```rust
use acorn::{Error, Result};
use std::time::{Duration, Instant};

fn retry_with_backoff<F, T>(mut operation: F, max_retries: u32) -> Result<T>
where
    F: FnMut() -> Result<T>,
{
    let mut attempt = 0;
    let base_delay = Duration::from_millis(100);
    
    loop {
        match operation() {
            Ok(result) => return Ok(result),
            Err(e) if e.is_recoverable() && attempt < max_retries => {
                attempt += 1;
                let delay = base_delay * (2_u32.pow(attempt - 1));
                std::thread::sleep(delay);
                continue;
            },
            Err(e) => return Err(e),
        }
    }
}
```

### **2. Circuit Breaker Pattern**

```rust
use acorn::{Error, Result};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

struct CircuitBreaker {
    state: Arc<Mutex<CircuitState>>,
    failure_threshold: u32,
    timeout: Duration,
}

enum CircuitState {
    Closed,
    Open(Instant),
    HalfOpen,
}

impl CircuitBreaker {
    fn new(failure_threshold: u32, timeout: Duration) -> Self {
        Self {
            state: Arc::new(Mutex::new(CircuitState::Closed)),
            failure_threshold,
            timeout,
        }
    }
    
    fn call<F, T>(&self, operation: F) -> Result<T>
    where
        F: FnOnce() -> Result<T>,
    {
        let mut state = self.state.lock().unwrap();
        
        match *state {
            CircuitState::Open(open_time) => {
                if Instant::now() - open_time > self.timeout {
                    *state = CircuitState::HalfOpen;
                } else {
                    return Err(Error::timeout(
                        "Circuit breaker is open",
                        "circuit_breaker",
                        0
                    ));
                }
            },
            CircuitState::HalfOpen => {
                // Allow one request to test
            },
            CircuitState::Closed => {
                // Normal operation
            }
        }
        
        match operation() {
            Ok(result) => {
                *state = CircuitState::Closed;
                Ok(result)
            },
            Err(e) => {
                *state = CircuitState::Open(Instant::now());
                Err(e)
            }
        }
    }
}
```

### **3. Graceful Degradation**

```rust
use acorn::{AcornTree, Error, Result};

fn graceful_degradation_example() -> Result<()> {
    let tree = AcornTree::open_memory()?;
    
    // Try to get cached data first
    match tree.crack("cached-data") {
        Ok(data) => {
            println!("Using cached data: {}", data);
            return Ok(());
        },
        Err(Error::NotFound { .. }) => {
            // Cache miss, continue to fallback
        },
        Err(e) => {
            eprintln!("Cache error: {}", e);
            // Continue to fallback
        }
    }
    
    // Fallback to default data
    println!("Using default data");
    Ok(())
}
```

## 📊 Error Monitoring and Metrics

### **Error Metrics Collection**

```rust
use acorn::{Error, Result};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

struct ErrorMetrics {
    counts: Arc<Mutex<HashMap<String, u64>>>,
    severities: Arc<Mutex<HashMap<String, u64>>>,
}

impl ErrorMetrics {
    fn new() -> Self {
        Self {
            counts: Arc::new(Mutex::new(HashMap::new())),
            severities: Arc::new(Mutex::new(HashMap::new())),
        }
    }
    
    fn record_error(&self, error: &Error) {
        let error_type = format!("{:?}", error);
        let severity = error.severity().to_string();
        
        // Increment error count
        {
            let mut counts = self.counts.lock().unwrap();
            *counts.entry(error_type.clone()).or_insert(0) += 1;
        }
        
        // Increment severity count
        {
            let mut severities = self.severities.lock().unwrap();
            *severities.entry(severity).or_insert(0) += 1;
        }
    }
    
    fn get_stats(&self) -> HashMap<String, u64> {
        self.counts.lock().unwrap().clone()
    }
}

// Usage
fn monitored_operation(metrics: &ErrorMetrics) -> Result<()> {
    let result = some_operation();
    
    if let Err(ref error) = result {
        metrics.record_error(error);
    }
    
    result
}
```

## 🛠️ Best Practices

### **1. Error Handling Guidelines**

```rust
// ✅ Good: Specific error handling
match tree.crack("key") {
    Ok(value) => process_value(value),
    Err(Error::NotFound { key, .. }) => handle_not_found(key),
    Err(Error::Serialization { .. }) => handle_serialization_error(),
    Err(e) => handle_other_error(e),
}

// ❌ Bad: Generic error handling
match tree.crack("key") {
    Ok(value) => process_value(value),
    Err(e) => {
        println!("Error: {}", e);
        // No specific handling
    }
}
```

### **2. Context Preservation**

```rust
// ✅ Good: Preserve context through error chain
fn process_data() -> Result<ProcessedData> {
    let raw_data = get_raw_data()
        .with_context(|| "Failed to retrieve raw data")?;
    
    let parsed_data = parse_data(&raw_data)
        .with_context(|| "Failed to parse raw data")?;
    
    let processed_data = transform_data(parsed_data)
        .with_context(|| "Failed to transform parsed data")?;
    
    Ok(processed_data)
}

// ❌ Bad: Lose context
fn process_data_bad() -> Result<ProcessedData> {
    let raw_data = get_raw_data()?;
    let parsed_data = parse_data(&raw_data)?;
    let processed_data = transform_data(parsed_data)?;
    Ok(processed_data)
}
```

### **3. Error Recovery**

```rust
// ✅ Good: Implement recovery strategies
fn resilient_sync() -> Result<()> {
    let mut retries = 0;
    let max_retries = 3;
    
    loop {
        match sync_operation() {
            Ok(_) => return Ok(()),
            Err(Error::Network { .. }) if retries < max_retries => {
                retries += 1;
                std::thread::sleep(Duration::from_millis(1000 * retries));
                continue;
            },
            Err(e) => return Err(e),
        }
    }
}

// ❌ Bad: No recovery
fn fragile_sync() -> Result<()> {
    sync_operation()?;
    Ok(())
}
```

## 📚 Error Reference

### **Common Error Scenarios**

| Scenario | Error Type | Recovery Strategy |
|----------|------------|-------------------|
| Key not found | `NotFound` | Check if key exists, use default value |
| Invalid JSON | `Serialization` | Validate JSON format, provide schema |
| Network timeout | `Timeout` | Retry with exponential backoff |
| File not found | `FileSystem` | Check file path, create if needed |
| Permission denied | `FileSystem` | Check permissions, request elevation |
| Memory exhausted | `ResourceExhausted` | Reduce batch size, free memory |
| Concurrent access | `ConcurrentAccess` | Implement locking, retry |
| Invalid parameters | `InvalidInput` | Validate input, provide examples |

### **Error Codes and Messages**

| Error Code | Message | Context | Solution |
|------------|---------|---------|----------|
| `ACORN_001` | Database file not found | File system | Check file path |
| `ACORN_002` | Invalid JSON format | Serialization | Validate JSON |
| `ACORN_003` | Network connection failed | Network | Check connectivity |
| `ACORN_004` | Encryption key invalid | Encryption | Verify key format |
| `ACORN_005` | Cache size exceeded | Cache | Increase cache size |
| `ACORN_006` | Query timeout | Query | Optimize query |
| `ACORN_007` | Transaction conflict | Transaction | Retry transaction |
| `ACORN_008` | Sync authentication failed | Sync | Check credentials |

## 🔧 Debugging Tools

### **Error Debugging Helper**

```rust
use acorn::{Error, Result};

fn debug_error(error: &Error) {
    println!("=== Error Debug Information ===");
    println!("Type: {:?}", error);
    println!("Message: {}", error);
    println!("Operation: {:?}", error.operation());
    println!("Context: {:?}", error.context());
    println!("Severity: {}", error.severity());
    println!("Recoverable: {}", error.is_recoverable());
    
    if let Some(source) = error.source() {
        println!("Source: {}", source);
    }
}
```

### **Error Tracing**

```rust
use acorn::{Error, Result};
use std::backtrace::Backtrace;

fn trace_error(error: &Error) {
    println!("Error: {}", error);
    println!("Backtrace: {}", Backtrace::capture());
}
```

---

*This guide will be updated as we implement additional error handling features and gather feedback from users.*
