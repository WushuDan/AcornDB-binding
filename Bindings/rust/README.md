# AcornDB Rust Bindings

Safe, idiomatic Rust bindings for AcornDB using a C# NativeAOT shim layer.

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│ Rust Application (Your Code)                               │
└─────────────────────────────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────┐
│ acorn crate (Safe Rust API)                                │
│ - AcornTree, Error types                                   │
│ - Generic serde JSON serialization                         │
└─────────────────────────────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────┐
│ acorn-sys crate (Raw FFI bindings)                         │
│ - Generated from acorn.h via bindgen                       │
└─────────────────────────────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────┐
│ C# NativeAOT Shim (acornshim.dylib/.so/.dll)               │
│ - FFI exports matching acorn.h                             │
│ - JSON façade over AcornDB Tree<object>                    │
└─────────────────────────────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────┐
│ AcornDB Core (C# Library)                                  │
│ - Tree, Nursery, Grove                                     │
│ - Storage, Sync, Conflict Resolution                       │
└─────────────────────────────────────────────────────────────┘
```

## Quick Start

### Prerequisites

- **Rust** (1.70+): Install from [rustup.rs](https://rustup.rs)
- **.NET 8 SDK**: Install from [dotnet.microsoft.com](https://dotnet.microsoft.com/download)
- **bindgen dependencies**:
  - macOS: `xcode-select --install`
  - Linux: `apt-get install llvm-dev libclang-dev clang`
  - Windows: Install LLVM from [llvm.org](https://llvm.org)

### Automated Build

The easiest way to build and test:

```bash
cd Bindings/rust
./build-and-test.sh           # Build shim and Rust bindings
./build-and-test.sh test      # Build and run tests
./build-and-test.sh example   # Build and run example
```

The script automatically:
- Detects your platform (macOS, Linux, Windows)
- Builds the C# shim for your architecture
- Sets up environment variables
- Builds the Rust bindings

### Manual Build

If you prefer manual control:

#### 1. Build the C# Shim

```bash
cd Bindings/rust/shim

# macOS ARM64 (M1/M2/M3)
dotnet publish -c Release -r osx-arm64

# macOS x64 (Intel)
dotnet publish -c Release -r osx-x64

# Linux x64
dotnet publish -c Release -r linux-x64

# Windows x64
dotnet publish -c Release -r win-x64
```

This produces a native library:
- macOS: `bin/Release/net8.0/{rid}/publish/acornshim.dylib`
- Linux: `bin/Release/net8.0/{rid}/publish/acornshim.so`
- Windows: `bin/Release/net8.0/{rid}/publish/acornshim.dll`

#### 2. Build Rust Bindings

```bash
cd Bindings/rust/bindings/acorn

# Set the shim directory (adjust path/RID for your platform)
export ACORN_SHIM_DIR="$(pwd)/../../shim/bin/Release/net8.0/osx-arm64/publish"

# Build the bindings
cargo build

# On macOS, also set library path for runtime
export DYLD_LIBRARY_PATH="$ACORN_SHIM_DIR"

# On Linux, use:
# export LD_LIBRARY_PATH="$ACORN_SHIM_DIR"

# On Windows, use:
# set PATH=%ACORN_SHIM_DIR%;%PATH%
```

#### 3. Run Example

```bash
cargo run --example basic_usage
```

#### 4. Run Tests

```bash
# Unit tests (no shim required)
cargo test --lib

# Integration tests (requires shim and env vars)
cargo test --features integration-tests
```

## Usage Example

```rust
use acorn::{AcornTree, Error};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct Person {
    name: String,
    age: u32,
}

fn main() -> Result<(), Error> {
    // Open a tree with file storage
    let mut tree = AcornTree::open("file://./my_db")?;

    // Store data
    let alice = Person {
        name: "Alice".to_string(),
        age: 30,
    };
    tree.stash("alice", &alice)?;

    // Retrieve data
    let retrieved: Person = tree.crack("alice")?;
    assert_eq!(alice, retrieved);

    // Handle not found
    match tree.crack::<Person>("bob") {
        Err(Error::NotFound) => println!("Bob not found"),
        Ok(person) => println!("Found: {:?}", person),
        Err(e) => println!("Error: {}", e),
    }

    // Iterate over all people
    let mut iter = tree.iter("")?;
    while let Some((key, person)) = iter.next::<Person>()? {
        println!("{}: {} (age {})", key, person.name, person.age);
    }

    // Or iterate with a prefix
    tree.stash("user:alice", &alice)?;
    tree.stash("user:bob", &Person { name: "Bob".to_string(), age: 25 })?;

    let mut user_iter = tree.iter("user:")?;
    let users: Vec<(String, Person)> = user_iter.collect()?;
    println!("Found {} users", users.len());

    // Subscribe to changes (callback runs on background thread)
    use std::sync::{Arc, Mutex};
    let notifications = Arc::new(Mutex::new(Vec::new()));
    let notifications_clone = notifications.clone();

    let _subscription = tree.subscribe(move |key: &str, value: &serde_json::Value| {
        println!("Changed: {} = {:?}", key, value);
        let mut n = notifications_clone.lock().unwrap();
        n.push(key.to_string());
    })?;

    // Any stash operations now trigger notifications
    tree.stash("user:charlie", &Person { name: "Charlie".to_string(), age: 35 })?;

    // Subscription automatically cleaned up when dropped

    // Sync with remote HTTP server (if available)
    // This pulls data from the remote and merges it into the local tree
    match tree.sync_http("http://localhost:5000/api/acorn") {
        Ok(()) => println!("Sync completed"),
        Err(e) => println!("Sync failed: {}", e), // Fault-tolerant
    }

    Ok(())
}
```

## API Reference

### `AcornTree`

The main interface for interacting with AcornDB.

#### Methods

- **`open(uri: &str) -> Result<AcornTree>`**
  Opens a tree at the given storage URI.
  - `"file://path/to/db"` - File-based storage
  - `"memory://"` - In-memory storage
  - Falls back to file storage if scheme is omitted

- **`stash<T: Serialize>(&mut self, id: &str, value: &T) -> Result<()>`**
  Stores a value with the given ID. The value is serialized to JSON.

- **`crack<T: DeserializeOwned>(&self, id: &str) -> Result<T>`**
  Retrieves a value by ID and deserializes it.
  - Returns `Err(Error::NotFound)` if the ID doesn't exist

- **`iter(&self, prefix: &str) -> Result<AcornIterator>`**
  Creates an iterator over key-value pairs with the given prefix.
  - Pass an empty string `""` to iterate over all keys
  - The iterator holds a snapshot of matching entries at creation time
  - Keys are returned in sorted order

- **`subscribe<F>(&self, callback: F) -> Result<AcornSubscription>`**
  Subscribe to real-time change notifications. The callback is invoked whenever an item is added or modified.
  - The callback receives `(key: &str, value: &serde_json::Value)`
  - Callback is invoked from a background thread (must be `Send + 'static`)
  - Subscription is automatically cleaned up when dropped
  - Multiple subscriptions can be active simultaneously

- **`sync_http(&self, url: &str) -> Result<()>`**
  Synchronize this tree with a remote AcornDB HTTP server.
  - Pulls all data from the remote endpoint and merges it into the local tree
  - Uses conflict resolution to handle concurrent modifications
  - Network errors are logged but don't cause the method to fail (fault-tolerant)
  - Requires a compatible AcornDB HTTP server at the URL

### `AcornIterator`

Iterator over key-value pairs, created by `AcornTree::iter()`.

#### Methods

- **`next<T: DeserializeOwned>(&mut self) -> Result<Option<(String, T)>>`**
  Get the next key-value pair.
  - Returns `Ok(Some((key, value)))` if there's an item
  - Returns `Ok(None)` when iteration is complete

- **`collect<T: DeserializeOwned>(&mut self) -> Result<Vec<(String, T)>>`**
  Collect all remaining items into a Vec. This consumes the iterator.

### `AcornSubscription`

Subscription to tree changes, created by `AcornTree::subscribe()`.

The subscription is automatically cleaned up when dropped, unsubscribing from the tree.

**Note**: The callback is invoked from a background thread, so it must be `Send + 'static`. Use appropriate synchronization (e.g., `Arc<Mutex<T>>`) if sharing state.

### `Error`

Error type for AcornDB operations.

```rust
pub enum Error {
    Acorn(String),  // General error from AcornDB or FFI layer
    NotFound,       // Key not found in tree
}
```

## Current Limitations

### Other Limitations

- **JSON only**: All values must be JSON-serializable (via serde)
- **String IDs**: IDs must be valid UTF-8 strings without null bytes
- **No transactions**: Operations are not atomic across multiple keys

## Project Structure

```
Bindings/rust/
├── shim/                      # C# NativeAOT shim
│   ├── AcornDB.Shim.csproj
│   ├── AcornFacade.cs        # Wrapper around AcornDB APIs
│   ├── NativeExports.cs      # FFI exports
│   ├── Error.cs              # Error handling
│   ├── HandleTable.cs        # Handle management
│   └── Utf8.cs               # UTF-8 marshalling
├── bindings/
│   ├── c/
│   │   └── acorn.h           # C header defining FFI interface
│   ├── acorn-sys/            # Low-level FFI bindings
│   │   ├── build.rs          # Bindgen configuration
│   │   └── src/lib.rs        # Generated bindings
│   └── acorn/                # Safe Rust wrapper
│       ├── src/lib.rs        # Public API
│       ├── examples/         # Usage examples
│       └── tests/            # Integration tests
├── build-and-test.sh         # Build automation
└── README.md                 # This file
```

## Troubleshooting

### "Undefined symbols" linker error

Make sure `ACORN_SHIM_DIR` is set and points to the correct publish directory:

```bash
export ACORN_SHIM_DIR="/full/path/to/Bindings/rust/shim/bin/Release/net8.0/osx-arm64/publish"
```

### "dyld: Library not loaded" (macOS) or "cannot open shared object file" (Linux)

Set the library path environment variable:

```bash
# macOS
export DYLD_LIBRARY_PATH="$ACORN_SHIM_DIR"

# Linux
export LD_LIBRARY_PATH="$ACORN_SHIM_DIR"
```

### Shim build warnings (IL2026, IL3050) from AcornDB core

You may see some NativeAOT warnings during the shim build. These come from the AcornDB core library (Newtonsoft.Json usage in FileTrunk, reflection in Tree<T>) and do not affect the shim functionality. The shim itself uses System.Text.Json source generation and is fully NativeAOT-compatible.

### Integration tests fail

Make sure you:
1. Built the shim for your platform
2. Set `ACORN_SHIM_DIR` environment variable
3. Set the library path (`DYLD_LIBRARY_PATH` or `LD_LIBRARY_PATH`)
4. Run tests with the feature flag: `cargo test --features integration-tests`

## Contributing

When adding new features:

1. **Update the header** (`bindings/c/acorn.h`) if adding new FFI functions
2. **Implement in shim** (`shim/NativeExports.cs` and `shim/AcornFacade.cs`)
3. **Add safe wrapper** in `bindings/acorn/src/lib.rs`
4. **Add tests** in `bindings/acorn/tests/integration_tests.rs`
5. **Update docs** in this README

Follow these safety rules:
- ✓ No GC-managed memory returned to Rust without copying
- ✓ Every FFI allocation has a corresponding free function
- ✓ No blocking operations in callbacks
- ✓ UTF-8 strings only, no null bytes in IDs
- ✓ Return codes: 0=success, 1=not found, -1=error

## License

Same as AcornDB core (see repository root LICENSE.md)
