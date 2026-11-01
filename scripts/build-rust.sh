#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/../rust"
cargo build
cargo test