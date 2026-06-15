#!/bin/bash

# asum uninstallation script

set -e

APP_NAME="asum"
INSTALL_DIR="/usr/local/bin"

echo "--------------------------------------------------"
echo "🗑️  Uninstalling $APP_NAME..."
echo "--------------------------------------------------"

if [ -f "$INSTALL_DIR/$APP_NAME" ]; then
    echo "Removing $APP_NAME from $INSTALL_DIR (requires sudo access)..."
    sudo rm -f "$INSTALL_DIR/$APP_NAME"
    echo "--------------------------------------------------"
    echo "✅ SUCCESS: $APP_NAME has been uninstalled successfully!"
    echo "--------------------------------------------------"
else
    echo "⚠️  $APP_NAME is not installed in $INSTALL_DIR or was not found."
    echo "--------------------------------------------------"
fi
