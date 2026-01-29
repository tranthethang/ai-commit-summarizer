#!/bin/bash

# Load environment variables from .env if it exists
if [ -f .env ]; then
    export $(grep -v '^#' .env | xargs)
fi

# --- CONFIGURATION ---
APP_NAME=${APP_NAME:-"asum"}
INSTALL_DIR=${INSTALL_DIR:-"/usr/local/bin"}

echo "--------------------------------------------------"
echo "🗑️  Uninstalling $APP_NAME..."
echo "--------------------------------------------------"

# 1. Check if the file exists
if [ ! -f "$INSTALL_DIR/$APP_NAME" ]; then
    echo "❓ $APP_NAME not found in $INSTALL_DIR. Nothing to do."
else
    # 2. Remove binary with sudo permissions
    echo "[1/2] Removing binary from $INSTALL_DIR..."
    sudo rm "$INSTALL_DIR/$APP_NAME"
    
    if [ $? -eq 0 ]; then
        echo "✅ Binary removed successfully."
    else
        echo "❌ Error: Could not remove binary."
        exit 1
    fi
fi

# 3. Suggest shell configuration cleanup
echo "[2/2] Checking for aliases in ~/.zshrc..."
if grep -q "$APP_NAME" ~/.zshrc; then
    echo "--------------------------------------------------"
    echo "💡 Note: You still have references to '$APP_NAME' in your ~/.zshrc."
    echo "   Please run 'nano ~/.zshrc' and remove any aliases or functions"
    echo "   related to this tool, then run 'source ~/.zshrc'."
else
    echo "✅ No aliases found in ~/.zshrc."
fi

echo "--------------------------------------------------"
echo "✨ $APP_NAME has been uninstalled."
echo "--------------------------------------------------"
