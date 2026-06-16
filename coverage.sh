#!/bin/bash

# 1. Clean up old coverage data
rm -rf target/debug/deps/*.gcno target/debug/deps/*.gcda target/*.profraw

# 2. Run tests with instrumented coverage flags
export RUSTFLAGS="-Cinstrument-coverage"
export LLVM_PROFILE_FILE="target/asum-%p-%m.profraw"

cargo test

# Ensure llvm-tools-preview is installed
rustup component add llvm-tools-preview

# Check if grcov is installed
if ! command -v grcov &> /dev/null; then
    echo "Error: grcov could not be found."
    echo "Please install it using: cargo install grcov"
    exit 1
fi

# 3. Generate LCOV and HTML coverage reports in the ./coverage directory
mkdir -p ./coverage
grcov target/ -s . --binary-path ./target/debug/ \
    -t lcov --branch --ignore-not-existing \
    -o ./coverage/lcov.info

grcov target/ -s . --binary-path ./target/debug/ \
    -t html --branch --ignore-not-existing \
    -o ./coverage/

# 4. Clean up raw profile data (.profraw)
rm -f target/*.profraw

# 5. Display completion message
echo "HTML report has been generated at: ./coverage/index.html"

# (Optional) Check coverage using lcov or a json parser tool if threshold validation (e.g. 90%) is needed
