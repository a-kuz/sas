#!/bin/bash

set -e

cd "$(dirname "$0")/.."

echo "Rebuilding application icons..."

if [ ! -f "assets/logo.png" ]; then
    echo "Error: assets/logo.png not found"
    exit 1
fi

echo "Step 1: Creating opaque logo..."
python3 tools/create_opaque_icon.py

echo "Step 2: Generating .icns and .ico files..."
python3 tools/generate_app_icons.py

echo "Step 3: Rebuilding macOS app bundle..."
cargo bundle --release

echo ""
echo "Done! App bundle created at:"
echo "  target/release/bundle/osx/SAS III.app"
echo ""
echo "To test the app, run:"
echo "  open \"target/release/bundle/osx/SAS III.app\""



