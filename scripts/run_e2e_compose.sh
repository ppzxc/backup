#!/usr/bin/env bash
set -euo pipefail

# Keep one local gate for the isolated container matrix.  The Rust harness owns resource naming,
# readiness deadlines, Setup Wizard input, invariant checks, and cleanup, so this entry point cannot
# accidentally drift into a second hand-written scenario.
cargo test --test e2e_isolated_container_test -- --nocapture
