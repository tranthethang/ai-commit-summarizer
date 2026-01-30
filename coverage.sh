#!/bin/bash

# 1. Dọn dẹp dữ liệu cũ
rm -rf target/debug/deps/*.gcno target/debug/deps/*.gcda *.profraw

# 2. Chạy test với cờ đặc biệt để sinh dữ liệu coverage
export RUSTFLAGS="-Cinstrument-coverage"
export LLVM_PROFILE_FILE="asum-%p-%m.profraw"

cargo test

# 3. Tổng hợp báo cáo HTML vào thư mục ./coverage
grcov . -s . --binary-path ./target/debug/ \
    -t html --branch --ignore-not-existing \
    -o ./coverage/

# 4. Hiển thị thông báo
echo "Báo cáo HTML đã được tạo tại: ./coverage/index.html"

# (Tùy chọn) Kiểm tra coverage bằng lcov hoặc công cụ phân tích json nếu cần parse threshold 90%
