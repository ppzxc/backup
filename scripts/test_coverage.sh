#!/usr/bin/env bash
set -euo pipefail

echo "=========================================="
echo " 1. Running cargo test..."
echo "=========================================="
cargo test

echo ""
echo "=========================================="
echo " 2. Measuring Code Coverage (cargo-llvm-cov)..."
echo "=========================================="
if command -v cargo-llvm-cov &> /dev/null; then
    cargo llvm-cov --summary-only
    echo ""
    echo "Generating HTML coverage report..."
    cargo llvm-cov --html --output-dir target/coverage/html
    echo "Coverage HTML report generated at: target/coverage/html/index.html"
else
    echo "cargo-llvm-cov is not installed. Install with: cargo install cargo-llvm-cov"
fi

echo ""
echo "=========================================="
echo " 3. Running Mutation Testing (cargo-mutants)..."
echo "=========================================="
if command -v cargo-mutants &> /dev/null; then
    cargo mutants --file src/config/model.rs --file src/config/legacy_import.rs --file src/runner/executor.rs || true
else
    echo "cargo-mutants is not installed. Install with: cargo install cargo-mutants"
fi

echo ""
echo "=========================================="
echo " Coverage & Quality Verification Complete!"
echo "=========================================="
