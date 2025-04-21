#!/bin/bash

set -e

# Get the directory where the script is located
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
PROJECT_ROOT="$( cd "$SCRIPT_DIR/.." && pwd )"

# Function to perform formatting
format() {
    echo "Running cargo fmt..."
    cargo fmt

    echo "Running cargo clippy --fix..."
    cargo clippy --fix --allow-dirty

    echo "Running stdlib make fmt..."
    cd "$PROJECT_ROOT/stdlib"
    make fmt
    cd "$PROJECT_ROOT"
}

# Function to perform format checking
check() {
    echo "Running cargo fmt --check..."
    cargo fmt --all -- --check

    echo "Running cargo clippy check..."
    cargo clippy --all-targets --all-features -- -D warnings

    echo "Running stdlib make fmt_check..."
    cd "$PROJECT_ROOT/stdlib"
    make fmt_check
    cd "$PROJECT_ROOT"
}

# Main script logic
case "$1" in
    "format")
        format
        ;;
    "check")
        check
        ;;
    *)
        echo "Usage: $0 {format|check}"
        echo "  format: Run formatting tools"
        echo "  check:  Run format checking tools"
        exit 1
        ;;
esac
