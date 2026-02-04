#!/bin/bash

# Bug Bounty Engine - Build and Setup Script

set -e

echo "=================================="
echo "BB-Engine Build and Setup"
echo "=================================="

# Check if Rust is installed
if ! command -v cargo &> /dev/null; then
    echo "Error: Rust is not installed"
    echo "Install from: https://rustup.rs/"
    exit 1
fi

echo "✓ Rust toolchain found"

# Check Rust version
RUST_VERSION=$(rustc --version | awk '{print $2}')
echo "  Version: $RUST_VERSION"

# Build the project
echo ""
echo "Building bb-engine..."
cargo build --release

if [ $? -eq 0 ]; then
    echo "✓ Build successful"
    echo ""
    echo "Binary location: target/release/bb-engine"
else
    echo "✗ Build failed"
    exit 1
fi

# Run tests
echo ""
echo "Running tests..."
cargo test

if [ $? -eq 0 ]; then
    echo "✓ All tests passed"
else
    echo "✗ Some tests failed"
    exit 1
fi

# Create output directories
echo ""
echo "Setting up directories..."
mkdir -p output
mkdir -p wordlists
echo "✓ Directories created"

# Verify examples exist
echo ""
echo "Checking example files..."
for file in examples/config.yaml examples/patterns.yaml examples/workflow.yaml examples/wordlist.txt; do
    if [ -f "$file" ]; then
        echo "✓ $file"
    else
        echo "✗ $file missing"
    fi
done

echo ""
echo "=================================="
echo "Setup Complete!"
echo "=================================="
echo ""
echo "Quick start commands:"
echo ""
echo "  # Discover endpoints"
echo "  ./target/release/bb-engine discover --target https://example.com --wordlist examples/wordlist.txt"
echo ""
echo "  # Fuzz for SQLi"
echo "  ./target/release/bb-engine fuzz --target https://example.com/api/user?id=1 --mode sqli"
echo ""
echo "  # Run full workflow"
echo "  ./target/release/bb-engine pipeline --target https://example.com --workflow examples/workflow.yaml"
echo ""
echo "  # Export findings"
echo "  ./target/release/bb-engine export --database bb-engine.db --format json --output findings.json"
echo ""
echo "Read README.md for full documentation"