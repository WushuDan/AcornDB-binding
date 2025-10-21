# 🌐 AcornDB Additional Language Bindings Guide

This guide provides comprehensive information about creating additional language bindings for AcornDB using the same C# shim and FFI interface.

## 🎯 Overview

The AcornDB Rust bindings serve as a reference implementation for creating bindings in other languages. All bindings use the same underlying C# shim and FFI interface, ensuring consistency and maintainability.

## 🏗️ Architecture

### **Shared Components**

```
┌─────────────────────────────────────────────────────────────┐
│                    AcornDB C# Core                         │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐          │
│  │   Trees     │ │   Trunks    │ │   Sync      │          │
│  └─────────────┘ └─────────────┘ └─────────────┘          │
└─────────────────────────────────────────────────────────────┘
                                │
                                ▼
┌─────────────────────────────────────────────────────────────┐
│                    C# NativeAOT Shim                       │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐          │
│  │ FFI Layer   │ │ Error       │ │ Memory      │          │
│  │             │ │ Handling    │ │ Management  │          │
│  └─────────────┘ └─────────────┘ └─────────────┘          │
└─────────────────────────────────────────────────────────────┘
                                │
                                ▼
┌─────────────────────────────────────────────────────────────┐
│                    Language Bindings                        │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐          │
│  │    Rust     │ │   Python    │ │     Go      │          │
│  │  (Complete) │ │ (Planned)   │ │ (Planned)   │          │
│  └─────────────┘ └─────────────┘ └─────────────┘          │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐          │
│  │   Node.js   │ │     C++     │ │     Java    │          │
│  │ (Planned)   │ │ (Planned)   │ │ (Planned)   │          │
│  └─────────────┘ └─────────────┘ └─────────────┘          │
└─────────────────────────────────────────────────────────────┘
```

### **FFI Interface**

All language bindings use the same C header file (`acorn.h`) which defines:
- **Opaque handles**: `acorn_tree_handle`, `acorn_batch_handle`, etc.
- **Data structures**: `acorn_buf`, `acorn_iter_handle`, etc.
- **Function declarations**: All FFI functions with consistent signatures

## 🐍 Python Bindings

### **Design Goals**

- **Pythonic API**: Follow Python conventions and idioms
- **Type Safety**: Use type hints and mypy for static analysis
- **Async Support**: Native async/await support
- **Error Handling**: Pythonic exception handling
- **Performance**: Minimal overhead, efficient memory management

### **Implementation Plan**

#### **1. Core Structure**

```python
# acorn/__init__.py
from .tree import Tree
from .batch import Batch
from .query import Query
from .encryption import Encryption
from .compression import Compression
from .sync import Sync
from .errors import AcornError, NotFoundError, SerializationError

__version__ = "0.1.0"
__all__ = [
    "Tree", "Batch", "Query", "Encryption", "Compression", "Sync",
    "AcornError", "NotFoundError", "SerializationError"
]
```

#### **2. Tree Implementation**

```python
# acorn/tree.py
import json
from typing import Optional, Iterator, Dict, Any
from .ffi import ffi, lib
from .errors import AcornError, NotFoundError

class Tree:
    """AcornDB Tree - the primary data structure for storing key-value pairs."""
    
    def __init__(self, uri: str):
        """Open a tree with the specified URI.
        
        Args:
            uri: Storage URI (e.g., "memory://", "file://./db")
            
        Raises:
            AcornError: If the tree cannot be opened
        """
        self._handle = ffi.new("acorn_tree_handle *")
        rc = lib.acorn_open_tree(uri.encode('utf-8'), self._handle)
        if rc != 0:
            raise AcornError("Failed to open tree")
    
    @classmethod
    def open_memory(cls) -> 'Tree':
        """Open an in-memory tree.
        
        Returns:
            Tree: A new in-memory tree
        """
        return cls("memory://")
    
    @classmethod
    def open_file(cls, path: str) -> 'Tree':
        """Open a file-based tree.
        
        Args:
            path: Path to the database file
            
        Returns:
            Tree: A new file-based tree
        """
        return cls(f"file://{path}")
    
    def stash(self, key: str, value: str) -> None:
        """Store a key-value pair.
        
        Args:
            key: The key to store
            value: The value to store (JSON string)
            
        Raises:
            AcornError: If the operation fails
        """
        rc = lib.acorn_stash(self._handle[0], key.encode('utf-8'), value.encode('utf-8'))
        if rc != 0:
            raise AcornError("Failed to stash value")
    
    def crack(self, key: str) -> Optional[str]:
        """Retrieve a value by key.
        
        Args:
            key: The key to retrieve
            
        Returns:
            Optional[str]: The value if found, None otherwise
            
        Raises:
            AcornError: If the operation fails
        """
        result = ffi.new("char **")
        rc = lib.acorn_crack(self._handle[0], key.encode('utf-8'), result)
        if rc != 0:
            raise AcornError("Failed to crack value")
        
        if result[0] == ffi.NULL:
            return None
        
        value = ffi.string(result[0]).decode('utf-8')
        lib.acorn_free_string(result[0])
        return value
    
    def toss(self, key: str) -> None:
        """Delete a value by key.
        
        Args:
            key: The key to delete
            
        Raises:
            AcornError: If the operation fails
        """
        rc = lib.acorn_toss(self._handle[0], key.encode('utf-8'))
        if rc != 0:
            raise AcornError("Failed to toss value")
    
    def iter(self) -> Iterator[str]:
        """Iterate over all values in the tree.
        
        Yields:
            str: JSON values from the tree
        """
        iterator = ffi.new("acorn_iter_handle *")
        rc = lib.acorn_iter(self._handle[0], iterator)
        if rc != 0:
            raise AcornError("Failed to create iterator")
        
        try:
            while True:
                result = ffi.new("char **")
                rc = lib.acorn_iter_next(iterator[0], result)
                if rc != 0:
                    break
                
                if result[0] == ffi.NULL:
                    break
                
                value = ffi.string(result[0]).decode('utf-8')
                lib.acorn_free_string(result[0])
                yield value
        finally:
            lib.acorn_iter_close(iterator[0])
    
    def __enter__(self):
        return self
    
    def __exit__(self, exc_type, exc_val, exc_tb):
        lib.acorn_close_tree(self._handle[0])
```

#### **3. Error Handling**

```python
# acorn/errors.py
class AcornError(Exception):
    """Base exception for AcornDB operations."""
    
    def __init__(self, message: str, operation: Optional[str] = None, context: Optional[str] = None):
        super().__init__(message)
        self.operation = operation
        self.context = context

class NotFoundError(AcornError):
    """Raised when a key is not found."""
    
    def __init__(self, key: str, operation: str):
        super().__init__(f"Key '{key}' not found", operation)
        self.key = key

class SerializationError(AcornError):
    """Raised when serialization/deserialization fails."""
    
    def __init__(self, message: str, data_type: str):
        super().__init__(f"Serialization error for {data_type}: {message}")
        self.data_type = data_type
```

#### **4. Async Support**

```python
# acorn/async_tree.py
import asyncio
from typing import Optional, AsyncIterator
from .tree import Tree
from .errors import AcornError

class AsyncTree:
    """Async wrapper for AcornDB Tree."""
    
    def __init__(self, tree: Tree):
        self._tree = tree
    
    async def stash(self, key: str, value: str) -> None:
        """Async stash operation."""
        loop = asyncio.get_event_loop()
        await loop.run_in_executor(None, self._tree.stash, key, value)
    
    async def crack(self, key: str) -> Optional[str]:
        """Async crack operation."""
        loop = asyncio.get_event_loop()
        return await loop.run_in_executor(None, self._tree.crack, key)
    
    async def toss(self, key: str) -> None:
        """Async toss operation."""
        loop = asyncio.get_event_loop()
        await loop.run_in_executor(None, self._tree.toss, key)
    
    async def iter(self) -> AsyncIterator[str]:
        """Async iteration."""
        loop = asyncio.get_event_loop()
        for value in await loop.run_in_executor(None, list, self._tree.iter()):
            yield value
```

#### **5. Usage Examples**

```python
# examples/basic_usage.py
import asyncio
import json
from acorn import Tree, AsyncTree
from acorn.errors import AcornError, NotFoundError

def sync_example():
    """Synchronous usage example."""
    try:
        # Open a tree
        tree = Tree.open_memory()
        
        # Store data
        user = {"id": "user-1", "name": "Alice", "email": "alice@example.com"}
        tree.stash("user-1", json.dumps(user))
        
        # Retrieve data
        user_json = tree.crack("user-1")
        if user_json:
            retrieved_user = json.loads(user_json)
            print(f"Retrieved user: {retrieved_user}")
        
        # Iterate over all data
        for item in tree.iter():
            print(f"Item: {item}")
        
    except NotFoundError as e:
        print(f"Key not found: {e.key}")
    except AcornError as e:
        print(f"AcornDB error: {e}")

async def async_example():
    """Asynchronous usage example."""
    try:
        # Open a tree
        tree = Tree.open_memory()
        async_tree = AsyncTree(tree)
        
        # Store data
        user = {"id": "user-1", "name": "Alice", "email": "alice@example.com"}
        await async_tree.stash("user-1", json.dumps(user))
        
        # Retrieve data
        user_json = await async_tree.crack("user-1")
        if user_json:
            retrieved_user = json.loads(user_json)
            print(f"Retrieved user: {retrieved_user}")
        
        # Iterate over all data
        async for item in async_tree.iter():
            print(f"Item: {item}")
        
    except NotFoundError as e:
        print(f"Key not found: {e.key}")
    except AcornError as e:
        print(f"AcornDB error: {e}")

if __name__ == "__main__":
    print("Synchronous example:")
    sync_example()
    
    print("\nAsynchronous example:")
    asyncio.run(async_example())
```

## 🐹 Go Bindings

### **Design Goals**

- **Go Idioms**: Follow Go conventions and patterns
- **Interface-based**: Use Go interfaces for extensibility
- **Error Handling**: Use Go's error handling patterns
- **Concurrency**: Leverage Go's goroutines and channels
- **Performance**: Minimal overhead, efficient memory management

### **Implementation Plan**

#### **1. Core Structure**

```go
// acorn.go
package acorn

import (
    "C"
    "encoding/json"
    "errors"
    "unsafe"
)

// Tree represents an AcornDB tree
type Tree struct {
    handle C.acorn_tree_handle
}

// Batch represents a batch of operations
type Batch struct {
    tree   *Tree
    handle C.acorn_batch_handle
}

// Query represents a query builder
type Query struct {
    tree   *Tree
    handle C.acorn_query_handle
}

// Error types
var (
    ErrNotFound      = errors.New("key not found")
    ErrSerialization = errors.New("serialization error")
    ErrInvalidInput  = errors.New("invalid input")
    ErrAcorn         = errors.New("acorn error")
)
```

#### **2. Tree Implementation**

```go
// tree.go
package acorn

import (
    "C"
    "encoding/json"
    "unsafe"
)

// Open opens a tree with the specified URI
func Open(uri string) (*Tree, error) {
    cURI := C.CString(uri)
    defer C.free(unsafe.Pointer(cURI))
    
    var handle C.acorn_tree_handle
    rc := C.acorn_open_tree(cURI, &handle)
    if rc != 0 {
        return nil, ErrAcorn
    }
    
    return &Tree{handle: handle}, nil
}

// OpenMemory opens an in-memory tree
func OpenMemory() (*Tree, error) {
    return Open("memory://")
}

// OpenFile opens a file-based tree
func OpenFile(path string) (*Tree, error) {
    return Open("file://" + path)
}

// Stash stores a key-value pair
func (t *Tree) Stash(key, value string) error {
    cKey := C.CString(key)
    defer C.free(unsafe.Pointer(cKey))
    
    cValue := C.CString(value)
    defer C.free(unsafe.Pointer(cValue))
    
    rc := C.acorn_stash(t.handle, cKey, cValue)
    if rc != 0 {
        return ErrAcorn
    }
    
    return nil
}

// Crack retrieves a value by key
func (t *Tree) Crack(key string) (string, error) {
    cKey := C.CString(key)
    defer C.free(unsafe.Pointer(cKey))
    
    var result *C.char
    rc := C.acorn_crack(t.handle, cKey, &result)
    if rc != 0 {
        return "", ErrAcorn
    }
    
    if result == nil {
        return "", ErrNotFound
    }
    
    defer C.acorn_free_string(result)
    return C.GoString(result), nil
}

// Toss deletes a value by key
func (t *Tree) Toss(key string) error {
    cKey := C.CString(key)
    defer C.free(unsafe.Pointer(cKey))
    
    rc := C.acorn_toss(t.handle, cKey)
    if rc != 0 {
        return ErrAcorn
    }
    
    return nil
}

// Iter returns an iterator over all values
func (t *Tree) Iter() *Iterator {
    var handle C.acorn_iter_handle
    rc := C.acorn_iter(t.handle, &handle)
    if rc != 0 {
        return nil
    }
    
    return &Iterator{handle: handle}
}

// Close closes the tree
func (t *Tree) Close() error {
    rc := C.acorn_close_tree(t.handle)
    if rc != 0 {
        return ErrAcorn
    }
    return nil
}

// Iterator represents an iterator over tree values
type Iterator struct {
    handle C.acorn_iter_handle
}

// Next returns the next value in the iteration
func (it *Iterator) Next() (string, bool) {
    var result *C.char
    rc := C.acorn_iter_next(it.handle, &result)
    if rc != 0 || result == nil {
        return "", false
    }
    
    defer C.acorn_free_string(result)
    return C.GoString(result), true
}

// Close closes the iterator
func (it *Iterator) Close() error {
    rc := C.acorn_iter_close(it.handle)
    if rc != 0 {
        return ErrAcorn
    }
    return nil
}
```

#### **3. Usage Examples**

```go
// examples/basic_usage.go
package main

import (
    "encoding/json"
    "fmt"
    "log"
    
    "github.com/acorn-db/acorn-go"
)

type User struct {
    ID    string `json:"id"`
    Name  string `json:"name"`
    Email string `json:"email"`
}

func main() {
    // Open a tree
    tree, err := acorn.OpenMemory()
    if err != nil {
        log.Fatal(err)
    }
    defer tree.Close()
    
    // Create a user
    user := User{
        ID:    "user-1",
        Name:  "Alice",
        Email: "alice@example.com",
    }
    
    // Serialize and store
    userJSON, err := json.Marshal(user)
    if err != nil {
        log.Fatal(err)
    }
    
    err = tree.Stash(user.ID, string(userJSON))
    if err != nil {
        log.Fatal(err)
    }
    
    // Retrieve and deserialize
    userJSONStr, err := tree.Crack(user.ID)
    if err != nil {
        log.Fatal(err)
    }
    
    var retrievedUser User
    err = json.Unmarshal([]byte(userJSONStr), &retrievedUser)
    if err != nil {
        log.Fatal(err)
    }
    
    fmt.Printf("Retrieved user: %+v\n", retrievedUser)
    
    // Iterate over all data
    iter := tree.Iter()
    defer iter.Close()
    
    for {
        value, ok := iter.Next()
        if !ok {
            break
        }
        fmt.Printf("Item: %s\n", value)
    }
}
```

## 🟨 Node.js Bindings

### **Design Goals**

- **JavaScript/TypeScript**: Support both JavaScript and TypeScript
- **Async/Await**: Native async/await support
- **Streams**: Node.js stream support for large data
- **Error Handling**: JavaScript error handling patterns
- **Performance**: Minimal overhead, efficient memory management

### **Implementation Plan**

#### **1. Core Structure**

```typescript
// src/index.ts
export { Tree } from './tree';
export { Batch } from './batch';
export { Query } from './query';
export { Encryption } from './encryption';
export { Compression } from './compression';
export { Sync } from './sync';

export * from './errors';

export const version = '0.1.0';
```

#### **2. Tree Implementation**

```typescript
// src/tree.ts
import { NativeAddon } from './native';
import { AcornError, NotFoundError } from './errors';

export class Tree {
    private handle: number;
    
    constructor(uri: string) {
        this.handle = NativeAddon.openTree(uri);
        if (this.handle === 0) {
            throw new AcornError('Failed to open tree');
        }
    }
    
    static openMemory(): Tree {
        return new Tree('memory://');
    }
    
    static openFile(path: string): Tree {
        return new Tree(`file://${path}`);
    }
    
    async stash(key: string, value: string): Promise<void> {
        const rc = NativeAddon.stash(this.handle, key, value);
        if (rc !== 0) {
            throw new AcornError('Failed to stash value');
        }
    }
    
    async crack(key: string): Promise<string | null> {
        const result = NativeAddon.crack(this.handle, key);
        if (result === null) {
            return null;
        }
        return result;
    }
    
    async toss(key: string): Promise<void> {
        const rc = NativeAddon.toss(this.handle, key);
        if (rc !== 0) {
            throw new AcornError('Failed to toss value');
        }
    }
    
    async *iter(): AsyncGenerator<string> {
        const iterator = NativeAddon.iter(this.handle);
        if (iterator === 0) {
            throw new AcornError('Failed to create iterator');
        }
        
        try {
            while (true) {
                const value = NativeAddon.iterNext(iterator);
                if (value === null) {
                    break;
                }
                yield value;
            }
        } finally {
            NativeAddon.iterClose(iterator);
        }
    }
    
    close(): void {
        NativeAddon.closeTree(this.handle);
    }
}
```

#### **3. Error Handling**

```typescript
// src/errors.ts
export class AcornError extends Error {
    constructor(
        message: string,
        public operation?: string,
        public context?: string
    ) {
        super(message);
        this.name = 'AcornError';
    }
}

export class NotFoundError extends AcornError {
    constructor(public key: string, operation: string) {
        super(`Key '${key}' not found`, operation);
        this.name = 'NotFoundError';
    }
}

export class SerializationError extends AcornError {
    constructor(message: string, public dataType: string) {
        super(`Serialization error for ${dataType}: ${message}`);
        this.name = 'SerializationError';
    }
}
```

#### **4. Usage Examples**

```typescript
// examples/basic_usage.ts
import { Tree } from '../src/tree';
import { AcornError, NotFoundError } from '../src/errors';

interface User {
    id: string;
    name: string;
    email: string;
}

async function basicUsage() {
    try {
        // Open a tree
        const tree = Tree.openMemory();
        
        // Create a user
        const user: User = {
            id: 'user-1',
            name: 'Alice',
            email: 'alice@example.com'
        };
        
        // Store data
        await tree.stash(user.id, JSON.stringify(user));
        
        // Retrieve data
        const userJSON = await tree.crack(user.id);
        if (userJSON) {
            const retrievedUser: User = JSON.parse(userJSON);
            console.log('Retrieved user:', retrievedUser);
        }
        
        // Iterate over all data
        for await (const item of tree.iter()) {
            console.log('Item:', item);
        }
        
        // Close the tree
        tree.close();
        
    } catch (error) {
        if (error instanceof NotFoundError) {
            console.log(`Key not found: ${error.key}`);
        } else if (error instanceof AcornError) {
            console.log(`AcornDB error: ${error.message}`);
        } else {
            console.error('Unexpected error:', error);
        }
    }
}

// Run the example
basicUsage().catch(console.error);
```

## 🔧 Implementation Guidelines

### **1. FFI Integration**

All language bindings should:
- Use the same C header file (`acorn.h`)
- Implement consistent error handling
- Follow language-specific conventions
- Provide type safety where possible

### **2. Memory Management**

- **Rust**: Automatic memory management with RAII
- **Python**: Reference counting with proper cleanup
- **Go**: Garbage collection with explicit cleanup
- **Node.js**: V8 garbage collection with native cleanup

### **3. Error Handling**

- **Rust**: Result<T, Error> with detailed error types
- **Python**: Exceptions with context information
- **Go**: Error interface with wrapped errors
- **Node.js**: Error classes with stack traces

### **4. Concurrency**

- **Rust**: Thread-safe types with Arc/Mutex
- **Python**: Thread-safe with GIL, async support
- **Go**: Goroutines with channels
- **Node.js**: Event loop with async/await

## 📊 Implementation Status

| Language | Status | Features | Documentation | Tests |
|----------|--------|----------|---------------|-------|
| **Rust** | ✅ Complete | All features | Complete | Complete |
| **Python** | 🔄 Planned | Core features | Planned | Planned |
| **Go** | 🔄 Planned | Core features | Planned | Planned |
| **Node.js** | 🔄 Planned | Core features | Planned | Planned |
| **C++** | 🔄 Planned | Core features | Planned | Planned |
| **Java** | 🔄 Planned | Core features | Planned | Planned |

## 🚀 Getting Started

### **For Python Developers**

```bash
# Install Python bindings (when available)
pip install acorn-db

# Basic usage
python -c "
import acorn
tree = acorn.Tree.open_memory()
tree.stash('key', 'value')
print(tree.crack('key'))
"
```

### **For Go Developers**

```bash
# Install Go bindings (when available)
go get github.com/acorn-db/acorn-go

# Basic usage
go run -c "
package main
import 'github.com/acorn-db/acorn-go'
func main() {
    tree, _ := acorn.OpenMemory()
    tree.Stash('key', 'value')
    value, _ := tree.Crack('key')
    println(value)
}
"
```

### **For Node.js Developers**

```bash
# Install Node.js bindings (when available)
npm install acorn-db

# Basic usage
node -e "
const acorn = require('acorn-db');
const tree = acorn.Tree.openMemory();
tree.stash('key', 'value');
console.log(tree.crack('key'));
"
```

## 🔮 Future Plans

### **Phase 1: Core Bindings**
- Python bindings with async support
- Go bindings with goroutine support
- Node.js bindings with stream support

### **Phase 2: Advanced Features**
- C++ bindings for high-performance applications
- Java bindings for enterprise applications
- Swift bindings for iOS/macOS applications

### **Phase 3: Ecosystem**
- Language-specific package managers
- IDE integrations and tooling
- Community examples and tutorials

---

*This guide will be updated as we implement additional language bindings and gather feedback from developers.*
