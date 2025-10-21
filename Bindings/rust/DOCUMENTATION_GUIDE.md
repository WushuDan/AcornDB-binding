# 📚 AcornDB Rust Bindings Documentation Guide

This guide provides comprehensive documentation for the AcornDB Rust bindings, including API reference, examples, tutorials, and best practices.

## 📖 Documentation Structure

### **1. Library Documentation (rustdoc)**
- **Main crate documentation**: Comprehensive overview and examples
- **API reference**: Complete function and type documentation
- **Examples**: Working code examples for all features
- **Error reference**: Detailed error types and handling

### **2. User Guides**
- **Getting Started**: Quick start guide and basic usage
- **Performance Guide**: Optimization strategies and benchmarking
- **Error Handling Guide**: Comprehensive error handling patterns
- **Migration Guide**: Upgrading between versions

### **3. Developer Documentation**
- **Architecture Overview**: System design and components
- **Contributing Guide**: How to contribute to the project
- **Testing Guide**: Testing strategies and examples
- **Release Guide**: Release process and versioning

## 🚀 Quick Start Documentation

### **Installation**

```toml
[dependencies]
acorn = "0.1.0"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

### **Basic Usage**

```rust
use acorn::{AcornTree, Error};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
struct User {
    id: String,
    name: String,
    email: String,
}

fn main() -> Result<(), Error> {
    // Open a tree (database)
    let tree = AcornTree::open_memory()?;
    
    // Create and store a user
    let user = User {
        id: "user-1".to_string(),
        name: "Alice".to_string(),
        email: "alice@example.com".to_string(),
    };
    
    let user_json = serde_json::to_string(&user)?;
    tree.stash(&user.id, &user_json)?;
    
    // Retrieve the user
    if let Some(user_json) = tree.crack(&user.id)? {
        let retrieved_user: User = serde_json::from_str(&user_json)?;
        println!("Retrieved user: {:?}", retrieved_user);
    }
    
    Ok(())
}
```

## 📋 API Reference

### **Core Types**

#### **AcornTree**
The primary data structure for storing and retrieving data.

```rust
pub struct AcornTree {
    h: acorn_tree_handle,
}
```

**Methods:**
- `open(uri: &str) -> Result<Self>` - Open a tree with the specified URI
- `open_memory() -> Result<Self>` - Open an in-memory tree
- `open_file(path: &str) -> Result<Self>` - Open a file-based tree
- `stash(key: &str, value: &str) -> Result<()>` - Store a key-value pair
- `crack(key: &str) -> Result<Option<String>>` - Retrieve a value by key
- `toss(key: &str) -> Result<()>` - Delete a value by key
- `iter() -> Result<AcornIterator>` - Create an iterator over all items
- `iter_from(prefix: &str) -> Result<AcornIterator>` - Create a prefix-based iterator

#### **AcornBatch**
Efficient batch operations for multiple changes.

```rust
pub struct AcornBatch {
    tree: AcornTree,
    h: acorn_batch_handle,
}
```

**Methods:**
- `new(tree: AcornTree) -> Result<Self>` - Create a new batch
- `stash(key: &str, value: &str) -> Result<()>` - Add a stash operation
- `toss(key: &str) -> Result<()>` - Add a toss operation
- `commit() -> Result<()>` - Commit all operations atomically

#### **AcornQuery**
LINQ-style queries with filtering and sorting.

```rust
pub struct AcornQuery {
    tree: AcornTree,
    h: acorn_query_handle,
}
```

**Methods:**
- `new(tree: AcornTree) -> Result<Self>` - Create a new query
- `where_condition<F>(predicate: F) -> Self` - Filter items with a predicate
- `order_by<F>(key_selector: F) -> Self` - Sort items by a key
- `order_by_descending<F>(key_selector: F) -> Self` - Sort items in descending order
- `take(count: usize) -> Self` - Limit the number of results
- `skip(count: usize) -> Self` - Skip a number of results
- `collect() -> Result<Vec<String>>` - Execute the query and collect results
- `first() -> Result<Option<String>>` - Get the first result
- `count() -> Result<usize>` - Count the number of results

### **Security & Encryption**

#### **AcornEncryption**
AES-256 encryption with PBKDF2 key derivation.

```rust
pub struct AcornEncryption {
    h: acorn_encryption_handle,
}
```

**Methods:**
- `from_password(password: &str, salt: &str) -> Result<Self>` - Create from password
- `from_key(key_base64: &str, iv_base64: &str) -> Result<Self>` - Create from key
- `encrypt(plaintext: &str) -> Result<String>` - Encrypt data
- `decrypt(ciphertext: &str) -> Result<String>` - Decrypt data
- `export_key() -> Result<String>` - Export encryption key
- `export_iv() -> Result<String>` - Export initialization vector

### **Compression**

#### **AcornCompression**
Data compression with multiple algorithms.

```rust
pub struct AcornCompression {
    h: acorn_compression_handle,
}
```

**Methods:**
- `new_gzip(level: CompressionLevel) -> Result<Self>` - Create Gzip compressor
- `new_brotli(level: CompressionLevel) -> Result<Self>` - Create Brotli compressor
- `compress(data: &str) -> Result<String>` - Compress data
- `decompress(data: &str) -> Result<String>` - Decompress data
- `get_stats() -> Result<CompressionStats>` - Get compression statistics

### **Caching**

#### **AcornCache**
LRU caching with configurable limits.

```rust
pub struct AcornCache {
    h: acorn_cache_handle,
}
```

**Methods:**
- `new_lru(max_size: usize) -> Result<Self>` - Create LRU cache
- `new_no_eviction() -> Result<Self>` - Create no-eviction cache
- `get_stats() -> Result<CacheStats>` - Get cache statistics
- `clear() -> Result<()>` - Clear the cache

### **Synchronization**

#### **AcornSync**
HTTP-based synchronization with remote servers.

```rust
pub struct AcornSync {
    tree: AcornTree,
    h: acorn_sync_handle,
}
```

**Methods:**
- `new(tree: AcornTree, server_url: &str) -> Result<Self>` - Create sync client
- `push() -> Result<()>` - Push local changes to server
- `pull() -> Result<()>` - Pull changes from server
- `sync_bidirectional() -> Result<()>` - Sync in both directions
- `get_stats() -> Result<SyncStats>` - Get synchronization statistics

### **Advanced Features**

#### **AcornTransaction**
ACID transactions for atomic operations.

```rust
pub struct AcornTransaction {
    tree: AcornTree,
    h: acorn_transaction_handle,
}
```

**Methods:**
- `new(tree: AcornTree) -> Result<Self>` - Create a new transaction
- `stash(key: &str, value: &str) -> Result<()>` - Add a stash operation
- `toss(key: &str) -> Result<()>` - Add a toss operation
- `commit() -> Result<()>` - Commit the transaction
- `rollback() -> Result<()>` - Rollback the transaction

#### **AcornSubscription**
Reactive programming with change notifications.

```rust
pub struct AcornSubscription {
    tree: AcornTree,
    h: acorn_subscription_handle,
}
```

**Methods:**
- `new(tree: AcornTree) -> Result<Self>` - Create a new subscription
- `subscribe<F>(callback: F) -> Result<()>` - Subscribe to all changes
- `subscribe_stash<F>(callback: F) -> Result<()>` - Subscribe to stash events
- `subscribe_toss<F>(callback: F) -> Result<()>` - Subscribe to toss events
- `subscribe_where<F>(predicate: F, callback: F) -> Result<()>` - Subscribe with filter

## 📚 Tutorials

### **Tutorial 1: Building a User Management System**

```rust
use acorn::{AcornTree, Error};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug, Clone)]
struct User {
    id: String,
    name: String,
    email: String,
    created_at: String,
    is_active: bool,
}

struct UserManager {
    tree: AcornTree,
}

impl UserManager {
    fn new() -> Result<Self, Error> {
        let tree = AcornTree::open_memory()?;
        Ok(Self { tree })
    }
    
    fn create_user(&self, name: String, email: String) -> Result<User, Error> {
        let id = format!("user-{}", uuid::Uuid::new_v4());
        let user = User {
            id: id.clone(),
            name,
            email,
            created_at: chrono::Utc::now().to_rfc3339(),
            is_active: true,
        };
        
        let user_json = serde_json::to_string(&user)?;
        self.tree.stash(&id, &user_json)?;
        
        Ok(user)
    }
    
    fn get_user(&self, id: &str) -> Result<Option<User>, Error> {
        if let Some(user_json) = self.tree.crack(id)? {
            let user: User = serde_json::from_str(&user_json)?;
            Ok(Some(user))
        } else {
            Ok(None)
        }
    }
    
    fn update_user(&self, id: &str, updates: HashMap<String, serde_json::Value>) -> Result<Option<User>, Error> {
        if let Some(user_json) = self.tree.crack(id)? {
            let mut user: User = serde_json::from_str(&user_json)?;
            
            // Apply updates
            for (key, value) in updates {
                match key.as_str() {
                    "name" => user.name = value.as_str().unwrap_or(&user.name).to_string(),
                    "email" => user.email = value.as_str().unwrap_or(&user.email).to_string(),
                    "is_active" => user.is_active = value.as_bool().unwrap_or(user.is_active),
                    _ => {}
                }
            }
            
            let updated_json = serde_json::to_string(&user)?;
            self.tree.stash(&id, &updated_json)?;
            
            Ok(Some(user))
        } else {
            Ok(None)
        }
    }
    
    fn delete_user(&self, id: &str) -> Result<bool, Error> {
        if self.tree.crack(id)?.is_some() {
            self.tree.toss(id)?;
            Ok(true)
        } else {
            Ok(false)
        }
    }
    
    fn list_users(&self) -> Result<Vec<User>, Error> {
        let mut users = Vec::new();
        
        for item in self.tree.iter()? {
            if let Ok(user) = serde_json::from_str::<User>(&item) {
                users.push(user);
            }
        }
        
        Ok(users)
    }
    
    fn search_users(&self, query: &str) -> Result<Vec<User>, Error> {
        let mut results = Vec::new();
        
        for item in self.tree.iter()? {
            if let Ok(user) = serde_json::from_str::<User>(&item) {
                if user.name.to_lowercase().contains(&query.to_lowercase()) ||
                   user.email.to_lowercase().contains(&query.to_lowercase()) {
                    results.push(user);
                }
            }
        }
        
        Ok(results)
    }
}

fn main() -> Result<(), Error> {
    let user_manager = UserManager::new()?;
    
    // Create users
    let alice = user_manager.create_user("Alice Smith".to_string(), "alice@example.com".to_string())?;
    let bob = user_manager.create_user("Bob Johnson".to_string(), "bob@example.com".to_string())?;
    
    println!("Created users: {:?}, {:?}", alice, bob);
    
    // Get user
    if let Some(user) = user_manager.get_user(&alice.id)? {
        println!("Retrieved user: {:?}", user);
    }
    
    // Update user
    let mut updates = HashMap::new();
    updates.insert("name".to_string(), serde_json::Value::String("Alice Brown".to_string()));
    updates.insert("is_active".to_string(), serde_json::Value::Bool(false));
    
    if let Some(updated_user) = user_manager.update_user(&alice.id, updates)? {
        println!("Updated user: {:?}", updated_user);
    }
    
    // Search users
    let search_results = user_manager.search_users("alice")?;
    println!("Search results: {:?}", search_results);
    
    // List all users
    let all_users = user_manager.list_users()?;
    println!("All users: {:?}", all_users);
    
    Ok(())
}
```

### **Tutorial 2: Building a Task Management System**

```rust
use acorn::{AcornTree, AcornBatch, Error};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
enum TaskStatus {
    Todo,
    InProgress,
    Done,
    Cancelled,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Task {
    id: String,
    title: String,
    description: String,
    status: TaskStatus,
    priority: u8, // 1-5, 5 being highest
    created_at: String,
    updated_at: String,
    due_date: Option<String>,
    tags: Vec<String>,
}

struct TaskManager {
    tree: AcornTree,
}

impl TaskManager {
    fn new() -> Result<Self, Error> {
        let tree = AcornTree::open_memory()?;
        Ok(Self { tree })
    }
    
    fn create_task(&self, title: String, description: String, priority: u8) -> Result<Task, Error> {
        let id = format!("task-{}", uuid::Uuid::new_v4());
        let now = chrono::Utc::now().to_rfc3339();
        
        let task = Task {
            id: id.clone(),
            title,
            description,
            status: TaskStatus::Todo,
            priority: priority.min(5).max(1),
            created_at: now.clone(),
            updated_at: now,
            due_date: None,
            tags: Vec::new(),
        };
        
        let task_json = serde_json::to_string(&task)?;
        self.tree.stash(&id, &task_json)?;
        
        Ok(task)
    }
    
    fn update_task_status(&self, id: &str, status: TaskStatus) -> Result<Option<Task>, Error> {
        if let Some(task_json) = self.tree.crack(id)? {
            let mut task: Task = serde_json::from_str(&task_json)?;
            task.status = status;
            task.updated_at = chrono::Utc::now().to_rfc3339();
            
            let updated_json = serde_json::to_string(&task)?;
            self.tree.stash(&id, &updated_json)?;
            
            Ok(Some(task))
        } else {
            Ok(None)
        }
    }
    
    fn add_tag(&self, id: &str, tag: String) -> Result<Option<Task>, Error> {
        if let Some(task_json) = self.tree.crack(id)? {
            let mut task: Task = serde_json::from_str(&task_json)?;
            
            if !task.tags.contains(&tag) {
                task.tags.push(tag);
                task.updated_at = chrono::Utc::now().to_rfc3339();
                
                let updated_json = serde_json::to_string(&task)?;
                self.tree.stash(&id, &updated_json)?;
                
                Ok(Some(task))
            } else {
                Ok(Some(task))
            }
        } else {
            Ok(None)
        }
    }
    
    fn get_tasks_by_status(&self, status: TaskStatus) -> Result<Vec<Task>, Error> {
        let mut tasks = Vec::new();
        
        for item in self.tree.iter()? {
            if let Ok(task) = serde_json::from_str::<Task>(&item) {
                if task.status == status {
                    tasks.push(task);
                }
            }
        }
        
        // Sort by priority (highest first)
        tasks.sort_by(|a, b| b.priority.cmp(&a.priority));
        
        Ok(tasks)
    }
    
    fn get_tasks_by_tag(&self, tag: &str) -> Result<Vec<Task>, Error> {
        let mut tasks = Vec::new();
        
        for item in self.tree.iter()? {
            if let Ok(task) = serde_json::from_str::<Task>(&item) {
                if task.tags.contains(&tag.to_string()) {
                    tasks.push(task);
                }
            }
        }
        
        Ok(tasks)
    }
    
    fn bulk_update_status(&self, task_ids: Vec<String>, status: TaskStatus) -> Result<usize, Error> {
        let mut batch = AcornBatch::new(self.tree.clone())?;
        let mut updated_count = 0;
        
        for task_id in task_ids {
            if let Some(task_json) = self.tree.crack(&task_id)? {
                let mut task: Task = serde_json::from_str(&task_json)?;
                task.status = status.clone();
                task.updated_at = chrono::Utc::now().to_rfc3339();
                
                let updated_json = serde_json::to_string(&task)?;
                batch.stash(&task_id, &updated_json)?;
                updated_count += 1;
            }
        }
        
        batch.commit()?;
        Ok(updated_count)
    }
    
    fn get_task_statistics(&self) -> Result<HashMap<String, usize>, Error> {
        let mut stats = HashMap::new();
        stats.insert("total".to_string(), 0);
        stats.insert("todo".to_string(), 0);
        stats.insert("in_progress".to_string(), 0);
        stats.insert("done".to_string(), 0);
        stats.insert("cancelled".to_string(), 0);
        
        for item in self.tree.iter()? {
            if let Ok(task) = serde_json::from_str::<Task>(&item) {
                *stats.get_mut("total").unwrap() += 1;
                
                match task.status {
                    TaskStatus::Todo => *stats.get_mut("todo").unwrap() += 1,
                    TaskStatus::InProgress => *stats.get_mut("in_progress").unwrap() += 1,
                    TaskStatus::Done => *stats.get_mut("done").unwrap() += 1,
                    TaskStatus::Cancelled => *stats.get_mut("cancelled").unwrap() += 1,
                }
            }
        }
        
        Ok(stats)
    }
}

fn main() -> Result<(), Error> {
    let task_manager = TaskManager::new()?;
    
    // Create tasks
    let task1 = task_manager.create_task(
        "Implement user authentication".to_string(),
        "Add login and registration functionality".to_string(),
        4
    )?;
    
    let task2 = task_manager.create_task(
        "Write documentation".to_string(),
        "Create comprehensive API documentation".to_string(),
        3
    )?;
    
    let task3 = task_manager.create_task(
        "Fix bug in payment processing".to_string(),
        "Resolve issue with credit card validation".to_string(),
        5
    )?;
    
    println!("Created tasks: {:?}, {:?}, {:?}", task1, task2, task3);
    
    // Update task status
    task_manager.update_task_status(&task1.id, TaskStatus::InProgress)?;
    task_manager.update_task_status(&task3.id, TaskStatus::Done)?;
    
    // Add tags
    task_manager.add_tag(&task1.id, "backend".to_string())?;
    task_manager.add_tag(&task1.id, "security".to_string())?;
    task_manager.add_tag(&task2.id, "documentation".to_string())?;
    task_manager.add_tag(&task3.id, "bugfix".to_string())?;
    task_manager.add_tag(&task3.id, "critical".to_string())?;
    
    // Get tasks by status
    let todo_tasks = task_manager.get_tasks_by_status(TaskStatus::Todo)?;
    let in_progress_tasks = task_manager.get_tasks_by_status(TaskStatus::InProgress)?;
    let done_tasks = task_manager.get_tasks_by_status(TaskStatus::Done)?;
    
    println!("Todo tasks: {:?}", todo_tasks);
    println!("In progress tasks: {:?}", in_progress_tasks);
    println!("Done tasks: {:?}", done_tasks);
    
    // Get tasks by tag
    let backend_tasks = task_manager.get_tasks_by_tag("backend")?;
    let critical_tasks = task_manager.get_tasks_by_tag("critical")?;
    
    println!("Backend tasks: {:?}", backend_tasks);
    println!("Critical tasks: {:?}", critical_tasks);
    
    // Bulk update
    let task_ids = vec![task1.id.clone(), task2.id.clone()];
    let updated_count = task_manager.bulk_update_status(task_ids, TaskStatus::Done)?;
    println!("Bulk updated {} tasks to Done", updated_count);
    
    // Get statistics
    let stats = task_manager.get_task_statistics()?;
    println!("Task statistics: {:?}", stats);
    
    Ok(())
}
```

## 🎯 Best Practices

### **1. Error Handling**

```rust
use acorn::{AcornTree, Error, ErrorContext};

fn best_practice_error_handling() -> Result<(), Error> {
    let tree = AcornTree::open_memory()?;
    
    // Use specific error handling
    match tree.crack("key") {
        Ok(Some(value)) => {
            // Process value
            println!("Found: {}", value);
        },
        Ok(None) => {
            // Handle not found case
            println!("Key not found");
        },
        Err(Error::NotFound { key, operation }) => {
            // Handle specific not found error
            println!("Key '{}' not found during {}", key, operation);
        },
        Err(e) => {
            // Handle other errors with context
            return Err(e).with_context(|| "Failed to retrieve data".to_string());
        }
    }
    
    Ok(())
}
```

### **2. Performance Optimization**

```rust
use acorn::{AcornTree, AcornBatch, Error};

fn best_practice_performance() -> Result<(), Error> {
    let tree = AcornTree::open_memory()?;
    
    // Use batch operations for multiple changes
    let mut batch = AcornBatch::new(tree)?;
    
    for i in 0..1000 {
        let key = format!("item-{}", i);
        let value = format!("data-{}", i);
        batch.stash(&key, &value)?;
    }
    
    // Commit all at once
    batch.commit()?;
    
    Ok(())
}
```

### **3. Data Serialization**

```rust
use acorn::{AcornTree, Error};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
struct User {
    id: String,
    name: String,
    email: String,
}

fn best_practice_serialization() -> Result<(), Error> {
    let tree = AcornTree::open_memory()?;
    
    let user = User {
        id: "user-1".to_string(),
        name: "Alice".to_string(),
        email: "alice@example.com".to_string(),
    };
    
    // Serialize before storing
    let user_json = serde_json::to_string(&user)?;
    tree.stash(&user.id, &user_json)?;
    
    // Deserialize after retrieving
    if let Some(user_json) = tree.crack(&user.id)? {
        let retrieved_user: User = serde_json::from_str(&user_json)?;
        println!("Retrieved: {:?}", retrieved_user);
    }
    
    Ok(())
}
```

## 🔧 Development Tools

### **Documentation Generation**

```bash
# Generate documentation
cargo doc --open

# Generate documentation with private items
cargo doc --document-private-items

# Generate documentation for all dependencies
cargo doc --all-deps
```

### **Documentation Testing**

```bash
# Test documentation examples
cargo test --doc

# Test all examples
cargo test --examples
```

### **Documentation Validation**

```bash
# Check documentation
cargo doc --no-deps

# Validate links
cargo doc --no-deps --document-private-items
```

## 📊 Documentation Metrics

### **Coverage Targets**

| Component | Target Coverage | Current Status |
|-----------|----------------|----------------|
| Public API | 100% | ✅ Complete |
| Examples | 100% | ✅ Complete |
| Error Types | 100% | ✅ Complete |
| Tutorials | 100% | ✅ Complete |
| Best Practices | 100% | ✅ Complete |

### **Quality Metrics**

- **API Documentation**: All public functions and types documented
- **Examples**: Working examples for all major features
- **Error Handling**: Comprehensive error documentation
- **Performance**: Performance guides and optimization tips
- **Migration**: Upgrade guides and breaking changes

## 🚀 Future Documentation

### **Planned Additions**

1. **Video Tutorials**: Screen recordings of common tasks
2. **Interactive Examples**: Web-based interactive examples
3. **API Explorer**: Interactive API documentation
4. **Performance Benchmarks**: Live performance comparisons
5. **Community Examples**: User-contributed examples and use cases

### **Documentation Maintenance**

- **Regular Updates**: Keep documentation current with code changes
- **User Feedback**: Incorporate user feedback and suggestions
- **Translation**: Support for multiple languages
- **Accessibility**: Ensure documentation is accessible to all users

---

*This documentation guide will be updated as we add new features and gather feedback from users.*
