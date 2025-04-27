#!/bin/bash
set -e

# Check if lib directory exists
if [ ! -d "lib" ]; then
    echo "lib directory not found, downloading dependencies..."
    ./download_deps.sh
fi

# Build based on platform
platform=$(uname)
if [ "$platform" = "Linux" ]; then
    echo "Building on Linux..."
    make -f dev.makefile release
elif [ "$platform" = "Darwin" ]; then
    echo "Building on macOS..."
    make -f dev.makefile release
else
    echo "Unsupported platform: $platform"
    exit 1
fi

git config --global --add safe.directory $(pwd)

# Get git commit hash and branch name
git_commit=$(git rev-parse --short HEAD)
git_branch=$(git rev-parse --abbrev-ref HEAD)

# Replace forward slashes with underscores in git_branch
git_branch=$(echo "$git_branch" | tr '/' '_')


# Create package directory
package_name="DTVM_SolSDK-${git_commit}-${platform}"
package_dir="/tmp/${package_name}"
mkdir -p "$package_dir"

# Copy required files to package directory
cp -r target/release/lib "$package_dir"
cp target/release/yul2wasm "$package_dir" 2>/dev/null || echo "Warning: yul2wasm not found"
cp docs/release_docs/* "$package_dir" 2>/dev/null || echo "Warning: release_docs not found"

# Create tarball in target/release directory
mkdir -p "target/release"
output_file="target/release/${package_name}.tar.gz"

# Remove existing file if it exists
if [ -f "$output_file" ]; then
    echo "Removing existing package file: $output_file"
    rm -f "$output_file"
fi

# Create the compressed package
tar -czf "$output_file" -C /tmp "$package_name"

cp -f $output_file target/release/DTVM_SolSDK-${platform}-nightly.tar.gz

echo "Package created: $output_file and target/release/DTVM_SolSDK-${platform}-${git_commit}.tar.gz"

# Clean up temporary directory
rm -rf "$package_dir"
