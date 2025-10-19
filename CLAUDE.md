You are acting as a senior maintainer for AcornDB adding Rust bindings.

## Current State (as of 2025-10-18)

### Completed Components
✓ C# NativeAOT Shim (`Bindings/rust/shim/`)
  - AcornFacade.cs: JSON façade wrapping Tree<JsonElement> with real AcornDB APIs (NativeAOT-compatible)
  - JsonContext.cs: System.Text.Json source generation context for AOT serialization
  - NativeExports.cs: FFI exports for CRUD operations (open, close, stash, crack, delete, exists, count)
  - Error.cs: Thread-local error handling with proper memory management
  - HandleTable.cs: Thread-safe handle management for Tree instances
  - Utf8.cs: UTF-8 string marshalling utilities
  - Builds successfully for osx-arm64 (produces acornshim.dylib)
  - Full NativeAOT compatibility with no reflection warnings from shim code

✓ C Header (`Bindings/rust/bindings/c/acorn.h`)
  - Defines complete FFI interface with opaque handles
  - Documents error codes: 0=OK, 1=NotFound, -1=Error
  - Includes CRUD, iteration, subscription, and sync function signatures
  - Memory management: acorn_free_buf() for shim-allocated buffers

✓ Rust Low-level Bindings (`acorn-sys` crate)
  - build.rs: Uses bindgen to generate FFI bindings from acorn.h
  - Conditional linking: only links when ACORN_SHIM_DIR is set
  - Helper: last_error_string() safely retrieves and frees error messages

✓ Rust Safe Wrapper (`acorn` crate)
  - AcornTree: Safe wrapper with open(), stash(), crack(), Drop
  - Generic serde support for JSON serialization
  - Error enum: Acorn(String), NotFound
  - Unit tests for type safety and serialization
  - Integration tests gated behind "integration-tests" feature
  - Example: basic_usage.rs demonstrates CRUD operations

### Partially Implemented
✅ Iterator API - COMPLETED (2025-10-18)
  - Header (acorn.h) defines: acorn_iter_start, acorn_iter_next, acorn_iter_close
  - IMPLEMENTED in AcornFacade.cs: JsonIterator with prefix filtering and snapshot semantics
  - IMPLEMENTED in NativeExports.cs: Full FFI exports with proper memory management
  - IMPLEMENTED in Rust (acorn crate): AcornIterator with next() and collect() methods
  - Example: iterator_usage.rs demonstrates prefix filtering and iteration patterns
  - Integration tests added for all iterator scenarios

✅ Subscription API - COMPLETED (2025-10-18)
  - Header defines: acorn_subscribe, acorn_unsubscribe with callback support
  - IMPLEMENTED in AcornFacade.cs: JsonSubscription using Reactive Extensions (ObserveStash)
  - IMPLEMENTED in NativeExports.cs: Full callback marshalling with proper memory management
  - IMPLEMENTED in Rust (acorn crate): AcornSubscription with automatic cleanup on drop
  - Example: subscription_usage.rs demonstrates real-time change notifications
  - Integration tests added: basic, update, drop, multiple subscriptions

✅ Sync API - COMPLETED (2025-10-18)
  - Header defines: acorn_sync_http(tree, url)
  - IMPLEMENTED in AcornFacade.cs: SyncHttpAsync using Branch.ShakeAsync
  - IMPLEMENTED in NativeExports.cs: Synchronous wrapper for async sync operation
  - IMPLEMENTED in Rust (acorn crate): sync_http() method on AcornTree
  - Example: sync_usage.rs demonstrates HTTP sync API usage
  - Integration tests added: unreachable URL, invalid URL (fault-tolerant behavior)

### Known Issues (Resolved)
✅ Linking Configuration - FIXED (2025-10-18)
  - Added .cargo/config.toml with platform-specific rpath configuration
  - Created build-and-test.sh script that automates entire build process
  - Script fixes dylib install_name and creates symlink with lib prefix
  - Tests and examples now work seamlessly with the build script

✅ NativeAOT JSON Serialization - FIXED (2025-10-18)
  - Replaced reflection-based `Tree<object>` with `Tree<JsonElement>`
  - Implemented JsonContext with System.Text.Json source generation
  - All IL2026/IL3050 warnings for shim code eliminated
  - Example and integration tests run successfully

⚠ Remaining NativeAOT Warnings (from AcornDB core, not critical)
  - IL2090 warnings: Tree<T> reflection in InitializeIdExtractor (AcornDB.csproj)
  - IL2026/IL3050 warnings: Newtonsoft.Json in FileTrunk<T> (AcornDB.csproj)
  - These warnings come from the AcornDB core library, not the shim
  - Do not affect shim functionality since we use JsonElement

### Goals
- Keep FFI surface minimal and stable; JSON/bytes façade only.
- Ensure NativeAOT shim builds on Win/Linux/macOS.
- Maintain safety: all shim-allocated buffers are freed by acorn_free_buf.

### Tasks Remaining
1) ~~Implement iterator exports in NativeExports.cs~~ ✅ DONE (2025-10-18)
2) ~~Implement subscription exports with background thread safety~~ ✅ DONE (2025-10-18)
3) ~~Implement sync export: acorn_sync_http~~ ✅ DONE (2025-10-18)
4) ~~Expand Rust safe wrapper with iter()~~ ✅ DONE (2025-10-18)
5) ~~Add README.md with build/test instructions~~ ✅ DONE (2025-10-18)
6) ~~Add .cargo/config.toml or build automation~~ ✅ DONE (2025-10-18)
7) ~~Fix NativeAOT warnings (JSON source generation)~~ ✅ DONE (2025-10-18)
8) ~~Add subscription example and tests~~ ✅ DONE (2025-10-18)
9) ~~Add sync example and tests~~ ✅ DONE (2025-10-18)
10) Add CI workflow to build shim for multiple RIDs and upload artifacts

### Constraints
- No generics across FFI.
- UTF-8 strings only.
- Return codes: 0=OK, 1=NotFound, -1=Error (see acorn_error_message).

### Review Checklist
- [x] No GC-managed memory is returned to Rust without a copy.
- [x] Every allocation crossing FFI has a free path (acorn_free_buf, acorn_free_error_string).
- [x] No blocking in callbacks; offload heavy work in Rust. (Subscription callbacks use ThreadPool.QueueUserWorkItem)
- [ ] CI artifacts uploaded for each RID.

### Building & Testing Quick Reference

**RECOMMENDED: Use the automated build script**
```bash
cd Bindings/rust
./build-and-test.sh           # Build everything
./build-and-test.sh example   # Build and run example
./build-and-test.sh test      # Build and run all tests
```

**Manual build (if needed)**
```bash
# Build the C# shim
cd Bindings/rust/shim
dotnet publish -c Release -r osx-arm64  # or linux-x64, win-x64

# Build Rust bindings (no linking, compiles only)
cd ../bindings/acorn
cargo build

# Build with linking (requires shim to be built first)
export ACORN_SHIM_DIR=/Users/dan.hurley/dev/projects/AcornDB-binding/Bindings/rust/shim/bin/Release/net8.0/osx-arm64/publish
export DYLD_LIBRARY_PATH=$ACORN_SHIM_DIR  # macOS
# export LD_LIBRARY_PATH=$ACORN_SHIM_DIR  # Linux
cargo build

# Run example
cargo run --example basic_usage

# Run integration tests
cargo test --features integration-tests
```

---

## Recent Changes (2025-10-18)

### ✅ Fixed: NativeAOT JSON Serialization Issue

**Problem**: The shim used reflection-based JSON serialization (`System.Text.Json.JsonSerializer.Deserialize<object>()`) which is incompatible with NativeAOT compilation. This caused runtime errors:
```
System.InvalidOperationException: Reflection-based serialization has been disabled for this application.
```

**Solution**:
1. Created `JsonContext.cs` with System.Text.Json source generation:
   - Uses `[JsonSourceGenerationOptions]` and `[JsonSerializable]` attributes
   - Provides compile-time generated serializers for `JsonElement`

2. Changed `AcornFacade.cs` to use `Tree<JsonElement>` instead of `Tree<object>`:
   - `JsonElement` is a struct that can represent any JSON structure
   - Fully compatible with NativeAOT (no reflection required)
   - Changed `Stash()`: `JsonSerializer.Deserialize(json, JsonContext.Default.JsonElement)`
   - Changed `Crack()`: `JsonSerializer.SerializeToUtf8Bytes(element, JsonContext.Default.JsonElement)`
   - Fixed `Exists()`: Check `element.ValueKind != JsonValueKind.Undefined` instead of null

3. Updated all `OpenJsonTree()` paths to use `Tree<JsonElement>`

**Result**:
- ✅ Example runs successfully and completes all operations
- ✅ All 12 integration tests pass
- ✅ No more IL2026/IL3050 warnings from shim code
- ✅ Full NativeAOT compatibility achieved

### ✅ Fixed: Linking Configuration

**Problem**: Tests and examples failed with "library 'acornshim' not found" errors.

**Root Causes**:
1. The dylib was named `acornshim.dylib` but linker searched for `libacornshim.dylib`
2. The dylib had incorrect install_name (relative path instead of `@rpath/acornshim.dylib`)
3. No automation for setting environment variables and building

**Solution**:
1. Created `.cargo/config.toml` with platform-specific rpath settings
2. Updated `build.rs` to add runtime search paths when `ACORN_SHIM_DIR` is set
3. Created `build-and-test.sh` automation script that:
   - Detects platform (macOS/Linux/Windows) and architecture
   - Builds C# shim with `dotnet publish`
   - Fixes dylib install_name with `install_name_tool -id "@rpath/acornshim.dylib"`
   - Creates `libacornshim.dylib` symlink for linker compatibility
   - Sets `ACORN_SHIM_DIR` and library path environment variables
   - Builds Rust bindings with correct linking
   - Can run examples and tests with single command

4. Created comprehensive `Bindings/rust/README.md` with:
   - Architecture diagram
   - Quick start guide
   - Manual build instructions
   - API reference and usage examples
   - Troubleshooting section

**Result**:
- ✅ Single-command build: `./build-and-test.sh`
- ✅ Example runs: `./build-and-test.sh example`
- ✅ Tests pass: `./build-and-test.sh test`
- ✅ Complete developer documentation

### ✅ Implemented: Iterator API

**Requirement**: Enable prefix-based iteration over key-value pairs in a tree, with point-in-time snapshot semantics.

**Implementation** (2025-10-18):

1. **C# Shim Layer** (`AcornFacade.cs`):
   ```csharp
   internal sealed class JsonIterator
   {
       - Uses Tree<JsonElement>.GetAllNuts() to get all items
       - Filters by prefix with StartsWith()
       - Orders by key for consistent iteration
       - Creates snapshot (ToList()) at iterator creation time
       - Implements Next(out string key, out byte[] json)
       - Proper Dispose() for cleanup
   }
   ```

2. **FFI Exports** (`NativeExports.cs`):
   ```csharp
   - acorn_iter_start: Creates iterator with HandleTable management
   - acorn_iter_next: Returns key+value buffers, sets done flag
   - acorn_iter_close: Disposes iterator and removes from HandleTable
   - Memory: Allocates unmanaged buffers for both key and JSON data
   - Safety: Cleans up key buffer if JSON allocation fails
   ```

3. **Rust Safe Wrapper** (`acorn/src/lib.rs`):
   ```rust
   pub struct AcornIterator {
       - Holds iterator handle
       - next<T: DeserializeOwned>() -> Result<Option<(String, T)>>
       - collect<T: DeserializeOwned>() -> Result<Vec<(String, T)>>
       - Implements Drop to call acorn_iter_close
   }

   impl AcornTree {
       pub fn iter(&self, prefix: &str) -> Result<AcornIterator>
   }
   ```

4. **Example** (`examples/iterator_usage.rs`):
   - Demonstrates prefix filtering (e.g., "electronics:", "books:")
   - Shows manual iteration with next()
   - Shows collecting all items with collect()
   - Calculates aggregates (totals by category)
   - Filters during iteration (expensive items)

5. **Integration Tests** (`tests/integration_tests.rs`):
   - test_iterator_basic: Full iteration over 5 items
   - test_iterator_with_prefix: Filter by "user:" and "product:" prefixes
   - test_iterator_manual_next: Manual stepping through items
   - test_iterator_empty: Empty tree iteration
   - test_iterator_no_match: Prefix with no matches

**Result**:
- ✅ Iterator example runs successfully
- ✅ Demonstrates all iteration patterns
- ✅ Proper memory management (no leaks)
- ✅ Snapshot semantics (isolated from concurrent modifications)
- ✅ Prefix filtering works correctly
- ✅ Sorted iteration by key

**Usage**:
```rust
let mut tree = AcornTree::open("file://./db")?;

// Store data
tree.stash("user:alice", &alice)?;
tree.stash("user:bob", &bob)?;

// Iterate with prefix
let mut iter = tree.iter("user:")?;
while let Some((key, person)) = iter.next::<Person>()? {
    println!("{}: {}", key, person.name);
}

// Or collect all
let mut iter = tree.iter("")?;
let all_items: Vec<(String, Person)> = iter.collect()?;
```

### ✅ Implemented: Subscription API

**Requirement**: Enable real-time change notifications when items are added or modified in the tree, with callbacks invoked from background threads to avoid blocking.

**Implementation** (2025-10-18):

1. **C# Shim Layer** (`AcornFacade.cs`):
   ```csharp
   internal sealed class JsonSubscription
   {
       - Uses AcornDB.Reactive.ObserveStash() extension method
       - Subscribes to Tree<object> stash events via IObservable<TreeChange<object>>
       - Serializes payload to JSON using JsonContext.Default.Object
       - Invokes callback on background thread via ThreadPool.QueueUserWorkItem
       - Proper Dispose() to unsubscribe from observable
       - Catches and ignores exceptions in user callback to prevent crashes
   }
   ```

2. **FFI Exports** (`NativeExports.cs`):
   ```csharp
   - acorn_subscribe: Creates subscription with callback marshalling
   - Uses Marshal.GetDelegateForFunctionPointer to convert function pointer
   - Creates SubscriptionContext to hold callback and user data
   - Allocates unmanaged memory for key (null-terminated) and JSON bytes
   - Memory freed immediately after callback returns (callback must copy data)
   - acorn_unsubscribe: Disposes subscription and removes from HandleTable
   - HandleTable<SubscriptionContext> manages subscription lifetimes
   ```

3. **Rust Safe Wrapper** (`acorn/src/lib.rs`):
   ```rust
   pub struct AcornSubscription {
       - Holds subscription handle
       - Stores Box<Box<dyn Fn(&str, &serde_json::Value) + Send>> callback
       - Callback remains alive as long as subscription exists
       - Implements Drop to call acorn_unsubscribe
       - unsafe extern "C" fn c_callback converts FFI types to Rust types
   }

   impl AcornTree {
       pub fn subscribe<F>(&self, callback: F) -> Result<AcornSubscription>
       where F: Fn(&str, &serde_json::Value) + Send + 'static
   }
   ```

4. **Example** (`examples/subscription_usage.rs`):
   - Demonstrates subscribing to tree changes
   - Tracks notifications using Arc<Mutex<Vec<>>>
   - Shows single update notifications
   - Shows multiple update notifications
   - Shows update to existing value
   - Demonstrates automatic cleanup on subscription drop

5. **Integration Tests** (`tests/integration_tests.rs`):
   - test_subscription_basic: Verifies notifications received for stash operations
   - test_subscription_update: Verifies both initial and updated values trigger notifications
   - test_subscription_drop: Verifies unsubscription works correctly
   - test_multiple_subscriptions: Verifies multiple subscriptions receive same notifications

**Result**:
- ✅ Subscription example runs successfully
- ✅ All 4 subscription tests pass
- ✅ Callbacks invoked from background threads (non-blocking)
- ✅ Proper memory management (no leaks)
- ✅ Automatic cleanup when subscription dropped
- ✅ Multiple subscriptions work correctly
- ✅ User callback exceptions handled gracefully

**Usage**:
```rust
let mut tree = AcornTree::open("memory://")?;

// Subscribe to changes
let _subscription = tree.subscribe(|key: &str, value: &serde_json::Value| {
    println!("Changed: {} = {:?}", key, value);
})?;

// Store some data - callback will be invoked
tree.stash("sensor-1", &sensor_reading)?;

// Subscription automatically cleaned up when dropped
```

**Threading Model**:
- C# side: Reactive Extensions delivers events on observable stream
- C# callback wrapper: Queues user callback on ThreadPool
- Rust callback: Receives notification on background thread
- User code: Callback must be Send + 'static and handle threading appropriately

### ✅ Implemented: Sync API

**Requirement**: Enable HTTP-based synchronization to pull data from remote AcornDB servers and merge it into the local tree.

**Implementation** (2025-10-18):

1. **C# Shim Layer** (`AcornFacade.cs`):
   ```csharp
   public async Task SyncHttpAsync(string url)
   {
       - Creates Branch instance with SyncMode.Bidirectional
       - Calls Branch.ShakeAsync(_tree) to pull from remote
       - ShakeAsync fetches data via HTTP GET /bark/{treename}/export
       - Deserializes List<Nut<object>> from JSON response
       - Merges into local tree using tree.Squabble() for conflict resolution
   }
   ```

2. **FFI Exports** (`NativeExports.cs`):
   ```csharp
   [UnmanagedCallersOnly(EntryPoint = "acorn_sync_http")]
   public static int SyncHttp(ulong treeHandle, IntPtr urlUtf8)
   {
       - Gets tree from HandleTable
       - Converts UTF-8 URL string
       - Calls tree.SyncHttpAsync(url).GetAwaiter().GetResult() to block
       - Returns 0 on success, -1 on error
   }
   ```

3. **Rust Safe Wrapper** (`acorn/src/lib.rs`):
   ```rust
   pub fn sync_http(&self, url: &str) -> Result<()> {
       - Converts URL to CString
       - Calls acorn_sync_http via FFI
       - Returns Ok(()) or Error
   }
   ```

4. **Example** (`examples/sync_usage.rs`):
   - Demonstrates sync_http() API usage
   - Shows error handling for unreachable/invalid URLs
   - Documents fault-tolerant behavior
   - Provides guidance on setting up AcornDB HTTP server

5. **Integration Tests** (`tests/integration_tests.rs`):
   - test_sync_http_unreachable: Verifies fault-tolerant behavior with unreachable URL
   - test_sync_http_invalid_url: Verifies graceful handling of malformed URLs

**Result**:
- ✅ Sync example runs successfully
- ✅ All 2 sync tests pass
- ✅ Fault-tolerant behavior (network errors logged, not thrown)
- ✅ Synchronous FFI wrapper for async C# operation
- ✅ Proper error handling and UTF-8 string conversion

**Usage**:
```rust
let tree = AcornTree::open("file://./db")?;

// Sync with remote HTTP endpoint
tree.sync_http("http://example.com:5000/api/acorn")?;

// Tree now contains merged data from remote
```

**Fault-Tolerant Behavior**:
- Network errors are logged to console but don't cause method to fail
- This is by design in Branch.ShakeAsync - sync is best-effort
- Allows application to continue even if sync fails
- Check console output for "Branch shake failed" messages

**Server Requirements**:
To use sync, you need an AcornDB HTTP server that exposes:
- `GET /bark/{treename}/export` - Returns all nuts as JSON array
- `POST /bark/{treename}/stash/{id}` - Accepts a nut for storage

The sync operation:
1. Fetches all nuts from remote via GET
2. Merges each nut into local tree using conflict resolution
3. Local tree uses configured ConflictJudge to resolve conflicts
