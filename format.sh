#!/bin/bash

# Run formatting and linting in parallel for efficiency
echo "Running format and clippy in parallel..."

# Run fmt in background
cargo fmt & 
fmt_pid=$!

# Run clippy in background (clippy includes all checks from 'cargo check')
cargo clippy --all-targets --all-features -- -D warnings &
clippy_pid=$!

# Wait for both processes
wait $fmt_pid
fmt_status=$?

wait $clippy_pid
clippy_status=$?

# Report status
if [ $fmt_status -ne 0 ] || [ $clippy_status -ne 0 ]; then
    echo "❌ Validation failed!"
    exit 1
fi

echo "✅ All checks passed!"
