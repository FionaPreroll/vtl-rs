#!/bin/bash -e

# Get the first argument
if [ "$1" == "install" ]; then
    # Link this script into the .git/hooks/pre-commit location
    ln -sf ../../pre-commit.sh .git/hooks/pre-commit
    echo "Pre-commit hook installed."
    exit 0
fi

# Check formatting with cargo fmt
echo "Running cargo fmt..."
cargo fmt -- --check

# Run clippy for linting
echo "Running cargo clippy..."
cargo clippy -- -D warnings

# Optionally run other checks or tests
echo "Running cargo test..."
cargo test

