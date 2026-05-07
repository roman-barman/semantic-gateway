#!/usr/bin/env bash
# Make this script executable before running: chmod +x tools/coverage.sh
set -euo pipefail

if ! cargo llvm-cov --version &>/dev/null; then
    echo "Error: cargo-llvm-cov is not installed." >&2
    echo "Install it with: cargo install cargo-llvm-cov --locked" >&2
    exit 1
fi

if ! cargo nextest --version &>/dev/null; then
    echo "Error: cargo-nextest is not installed." >&2
    echo "Install it with: cargo install cargo-nextest --locked" >&2
    exit 1
fi

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "${REPO_ROOT}"

echo "Running coverage analysis..." >&2

TOTAL_LINE=$(cargo llvm-cov nextest --workspace --summary-only 2>/dev/null | grep '^TOTAL')

if [[ -z "${TOTAL_LINE}" ]]; then
    echo "Error: No TOTAL line found. Run 'cargo build' to check for compile errors." >&2
    exit 1
fi

PERCENTAGE=$(echo "${TOTAL_LINE}" | awk '{print $10}')

if [[ -z "${PERCENTAGE}" ]]; then
    echo "Error: Could not parse percentage from: ${TOTAL_LINE}" >&2
    exit 1
fi

echo "Coverage: ${PERCENTAGE}"
