#!/bin/bash
set -e

# Prerequisites:
# rustup component add llvm-tools-preview
# cargo install rustfilt
# cargo install grcov@0.5.1
# or download grcov from https://github.com/mozilla/grcov/releases

export RUSTC_BOOTSTRAP=1

# Create the directory for coverage reports
mkdir -p target/coverage

# Clean any previous coverage data
rm -rf target/coverage/*

# Clean previous build artifacts to ensure coverage instrumentation is applied
cargo clean
rm -f *.profraw

export RUSTFLAGS="-Cinstrument-coverage"

# Build the project with coverage instrumentation

cargo build --verbose

# Set environment variables for coverage
export CARGO_INCREMENTAL=0
export RUSTFLAGS="-C instrument-coverage -Zprofile -Ccodegen-units=1 -Copt-level=0 -Clink-dead-code -Coverflow-checks=off -Zpanic_abort_tests -Cpanic=abort"
export RUSTDOCFLAGS="-Cpanic=abort"
export LLVM_PROFILE_FILE="target/coverage/coverage-%p-%m.profraw"

# Run tests with coverage instrumentation
cargo test --no-fail-fast --verbose

# List all the profraw files to verify they were created
echo "Generated profraw files:"
# ls -la target/coverage/

# Generate coverage report
# target/coverage
grcov . --binary-path ./target/debug -s . -t html --branch --ignore-not-existing --ignore "/*" --ignore "target/*" --ignore "tests/*" --ignore "examples/*" -o target/coverage/html

# Generate a summary report
grcov . --binary-path ./target/debug -s . -t lcov --branch --ignore-not-existing --ignore "/*" --ignore "target/*" --ignore "tests/*" --ignore "examples/*" -o target/coverage/lcov.info

# Print a message with the location of the coverage report
echo "Coverage report generated at target/coverage/html/index.html"

ls -la *.profraw
rm -rf *.profraw

# Start a local web server to view the coverage report
cd target/coverage/html && python3 -m http.server 8080
# open http://localhost:8080/src/yul2ir/index.html
