#!/bin/bash
# Validation script for Nakama long-term memory integration
# Checks system readiness and basic functionality

set -e

echo "🔍 Nakama Long-Term Memory Validation Script"
echo "=============================================="
echo ""

# Check 1: Ollama running
echo "[1/5] Checking Ollama service..."
if curl -s http://localhost:11434/api/tags > /dev/null; then
    echo "✓ Ollama is reachable on localhost:11434"
    OLLAMA_MODELS=$(curl -s http://localhost:11434/api/tags | jq -r '.models[].name' 2>/dev/null || echo "unknown")
    echo "  Available models: $OLLAMA_MODELS"
else
    echo "✗ Ollama is NOT reachable on localhost:11434"
    echo "  Fix: Install Ollama (https://ollama.ai) and run:"
    echo "       ollama run deepseek-r1:1.5b"
    exit 1
fi

# Check 2: LanceDB directory writable
echo ""
echo "[2/5] Checking LanceDB directory..."
LANCEDB_PATH="${HOME}/.lancedb"
if [ ! -d "$LANCEDB_PATH" ]; then
    echo "  Creating $LANCEDB_PATH..."
    mkdir -p "$LANCEDB_PATH"
fi

if [ -w "$LANCEDB_PATH" ]; then
    echo "✓ LanceDB directory is writable at $LANCEDB_PATH"
    SIZE=$(du -sh "$LANCEDB_PATH" 2>/dev/null | cut -f1)
    echo "  Current size: $SIZE"
else
    echo "✗ LanceDB directory is NOT writable!"
    echo "  Fix: Check permissions on $LANCEDB_PATH"
    exit 1
fi

# Check 3: Cargo dependencies compile
echo ""
echo "[3/5] Checking Rust compilation..."
cd "$(dirname "$0")/../src-tauri"

if cargo check -p nakama --features swiftide_integration > /tmp/cargo_check.log 2>&1; then
    echo "✓ Nakama crate compiles successfully with swiftide_integration"
else
    echo "✗ Cargo check failed!"
    echo "  Error log (last 20 lines):"
    tail -20 /tmp/cargo_check.log
    exit 1
fi

# Check 4: Test RAG functionality (if app running)
echo ""
echo "[4/5] Checking Tauri app runtime (optional)..."
if pgrep -f "nakama" > /dev/null; then
    echo "✓ Nakama app is running"
    echo "  Note: Full functional tests require UI interaction"
else
    echo "ℹ  Nakama app not running (expected for dev)"
    echo "  Build and run: pnpm tauri dev"
fi

# Check 5: File system permissions
echo ""
echo "[5/5] Checking file system permissions..."
TEST_FILE="/tmp/nakama_test_$$.txt"
if echo "test content" > "$TEST_FILE" && rm "$TEST_FILE"; then
    echo "✓ Temporary file I/O works"
else
    echo "✗ Cannot write to /tmp directory"
    exit 1
fi

# Summary
echo ""
echo "=============================================="
echo "✓ All checks passed!"
echo ""
echo "Next steps:"
echo "  1. Build: cargo build --features swiftide_integration --manifest-path src-tauri/Cargo.toml"
echo "  2. Run:   pnpm tauri dev"
echo "  3. Test:  Upload a file in the Index Manager (📚 button)"
echo "  4. Ask:   Query about the uploaded file content"
echo ""
echo "For detailed guide, see: LONG_TERM_MEMORY.md"
