#!/bin/bash

# Load environment variables from .env if it exists
if [ -f .env ]; then
    export $(grep -v '^#' .env | xargs)
fi

# --- CONFIGURATION ---
APP_NAME=${APP_NAME:-"asum"}
INSTALL_DIR=${INSTALL_DIR:-"/usr/local/bin"}
BINARY_PATH="target/release/$APP_NAME"

echo "--------------------------------------------------"
echo "🚀 Starting Release Build for $APP_NAME"
echo "--------------------------------------------------"

# 1. Check if Rust is installed
if ! command -v cargo &> /dev/null; then
    echo "❌ Error: Rust/Cargo is not installed."
    exit 1
fi

# 2. Compile Release version with high optimization
echo "[1/3] Compiling optimized binary..."
cargo build --release

# Check if build failed
if [ $? -ne 0 ]; then
    echo "❌ Build failed. Please check your Rust code."
    exit 1
fi

# 3. Compress binary file (Optional - makes file lighter)
# On Mac you can use 'strip' to remove extra symbols
echo "[2/3] Stripping debug symbols to reduce size..."
strip "$BINARY_PATH"

# 4. Install to system
echo "[3/3] Installing to $INSTALL_DIR..."
sudo cp "$BINARY_PATH" "$INSTALL_DIR/$APP_NAME"
sudo chmod +x "$INSTALL_DIR/$APP_NAME"

echo "--------------------------------------------------"
echo "✅ SUCCESS: $APP_NAME is now updated and ready!"
echo "Location: $(which $APP_NAME)"
echo "Version:  $($APP_NAME --version 2>/dev/null || echo '0.1.0')"
echo "--------------------------------------------------"
