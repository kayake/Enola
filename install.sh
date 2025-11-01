#!/bin/bash
set -e

BIN_NAME="enola"
INSTALL_DIR="/usr/bin"      
DATA_DIR="$HOME/.enola"             
PROJECT_ROOT="$(pwd)"

cargo build --release

cargo build --release
echo "Installing $BIN_NAME to $INSTALL_DIR (requires sudo)..."
sudo cp "$PROJECT_ROOT/target/release/$BIN_NAME" "$INSTALL_DIR/$BIN_NAME"
sudo chmod +x "$INSTALL_DIR/$BIN_NAME"

echo "Installing data to $DATA_DIR (requires sudo)..."
sudo mkdir -p "$DATA_DIR"
sudo cp -r "$PROJECT_ROOT/src/utils/"* "$DATA_DIR/"

echo "$BIN_NAME installation completed successfully!"
echo "You can run it using the command: $BIN_NAME"
echo "Data directory is located at: $DATA_DIR"
echo "Utils have been copied to: $DATA_DIR"
echo "Enjoy using $BIN_NAME!"
