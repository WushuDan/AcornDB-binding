#!/usr/bin/env bash
set -euo pipefail

# Build script for AcornDB Rust bindings
# This script builds the C# shim and then builds/tests the Rust bindings

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BINDINGS_ROOT="$SCRIPT_DIR"
SHIM_DIR="$BINDINGS_ROOT/shim"
RUST_DIR="$BINDINGS_ROOT/bindings/acorn"

# Detect platform
if [[ "$OSTYPE" == "darwin"* ]]; then
    if [[ "$(uname -m)" == "arm64" ]]; then
        RID="osx-arm64"
        LIB_PATH_VAR="DYLD_LIBRARY_PATH"
    else
        RID="osx-x64"
        LIB_PATH_VAR="DYLD_LIBRARY_PATH"
    fi
elif [[ "$OSTYPE" == "linux-gnu"* ]]; then
    if [[ "$(uname -m)" == "x86_64" ]]; then
        RID="linux-x64"
        LIB_PATH_VAR="LD_LIBRARY_PATH"
    elif [[ "$(uname -m)" == "aarch64" ]]; then
        RID="linux-arm64"
        LIB_PATH_VAR="LD_LIBRARY_PATH"
    else
        echo "Unsupported Linux architecture: $(uname -m)"
        exit 1
    fi
elif [[ "$OSTYPE" == "msys" ]] || [[ "$OSTYPE" == "win32" ]]; then
    if [[ "$(uname -m)" == "x86_64" ]]; then
        RID="win-x64"
        LIB_PATH_VAR="PATH"
    else
        echo "Unsupported Windows architecture: $(uname -m)"
        exit 1
    fi
else
    echo "Unsupported OS: $OSTYPE"
    exit 1
fi

PUBLISH_DIR="$SHIM_DIR/bin/Release/net8.0/$RID/publish"

echo "==> Building AcornDB Rust Bindings"
echo "    Platform: $RID"
echo "    Shim directory: $SHIM_DIR"
echo "    Publish directory: $PUBLISH_DIR"
echo ""

# Step 1: Build the C# shim
echo "==> Step 1: Building C# NativeAOT shim..."
cd "$SHIM_DIR"
dotnet publish -c Release -r "$RID" --nologo

if [ ! -f "$PUBLISH_DIR/acornshim.dylib" ] && [ ! -f "$PUBLISH_DIR/acornshim.so" ] && [ ! -f "$PUBLISH_DIR/acornshim.dll" ]; then
    echo "ERROR: Shim library not found in $PUBLISH_DIR"
    exit 1
fi

# Fix dylib install_name on macOS
if [[ "$OSTYPE" == "darwin"* ]] && [ -f "$PUBLISH_DIR/acornshim.dylib" ]; then
    install_name_tool -id "@rpath/acornshim.dylib" "$PUBLISH_DIR/acornshim.dylib"
    # Create symlink with lib prefix for linker
    cd "$PUBLISH_DIR"
    ln -sf acornshim.dylib libacornshim.dylib
fi

echo "✓ Shim built successfully"
echo ""

# Step 2: Build Rust bindings
echo "==> Step 2: Building Rust bindings..."
cd "$RUST_DIR"
export ACORN_SHIM_DIR="$PUBLISH_DIR"
export "$LIB_PATH_VAR"="$PUBLISH_DIR:${!LIB_PATH_VAR:-}"

cargo build
echo "✓ Rust bindings built successfully"
echo ""

# Step 3: Run tests if requested
if [[ "${1:-}" == "test" ]]; then
    echo "==> Step 3: Running tests..."

    # Run unit tests (no shim required)
    echo "Running unit tests..."
    cargo test --lib

    # Run integration tests (requires shim)
    echo "Running integration tests..."
    cargo test --features integration-tests

    echo "✓ All tests passed"
    echo ""
fi

# Step 4: Run example if requested
if [[ "${1:-}" == "example" ]]; then
    echo "==> Step 3: Running example..."
    cargo run --example basic_usage
    echo ""
fi

echo "==> Build complete!"
echo ""
echo "To use the bindings in your own project, set these environment variables:"
echo "    export ACORN_SHIM_DIR=\"$PUBLISH_DIR\""
echo "    export $LIB_PATH_VAR=\"$PUBLISH_DIR:\$$LIB_PATH_VAR\""
echo ""
echo "Then run:"
echo "    cargo build"
echo "    cargo run --example basic_usage"
echo "    cargo test --features integration-tests"
