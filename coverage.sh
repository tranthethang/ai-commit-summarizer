#!/bin/bash

# 1. Dọn dẹp dữ liệu cũ
rm -rf target/debug/deps/*.gcno target/debug/deps/*.gcda target/*.profraw

# 2. Chạy test với cờ đặc biệt để sinh dữ liệu coverage
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

# 3. Tổng hợp báo cáo HTML vào thư mục ./coverage
grcov target/ -s . --binary-path ./target/debug/ \
    -t html --branch --ignore-not-existing \
    -o ./coverage/

# 4. Dọn dẹp file .profraw
rm -f target/*.profraw

# 5. Hiển thị thông báo
echo "Báo cáo HTML đã được tạo tại: ./coverage/index.html"

# (Tùy chọn) Kiểm tra coverage bằng lcov hoặc công cụ phân tích json nếu cần parse threshold 90%
