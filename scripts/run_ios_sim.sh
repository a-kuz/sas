#!/bin/bash

set -e

export PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:$PATH"

APP_NAME="SAS3"
BUNDLE_ID="com.kuznetsov.sas3"
APP_DIR="${APP_NAME}.app"

SIM_ID=$(xcrun simctl list devices available | grep "iPhone" | head -1 | grep -o '[A-F0-9-]\{36\}')
echo "Using simulator: $SIM_ID"

echo "Building for iOS simulator..."
cargo build --bin sas --target aarch64-apple-ios-sim --release

echo "Creating app bundle..."
rm -rf "$APP_DIR"
mkdir -p "$APP_DIR"

echo "Copying binary..."
cp "target/aarch64-apple-ios-sim/release/sas" "$APP_DIR/$APP_NAME"

echo "Creating Info.plist..."
cat > "$APP_DIR/Info.plist" << EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleExecutable</key>
    <string>$APP_NAME</string>
    <key>CFBundleIdentifier</key>
    <string>$BUNDLE_ID</string>
    <key>CFBundleName</key>
    <string>$APP_NAME</string>
    <key>CFBundleVersion</key>
    <string>1</string>
    <key>CFBundleShortVersionString</key>
    <string>0.62.0</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
</dict>
</plist>
EOF

echo "Copying resources..."
mkdir -p "$APP_DIR/q3-resources"
cp -r q3-resources/* "$APP_DIR/q3-resources/" 2>/dev/null || echo "Note: q3-resources not fully copied"

mkdir -p "$APP_DIR/maps"
cp maps/*.json "$APP_DIR/maps/" 2>/dev/null || echo "Note: maps not fully copied"

echo "Booting simulator..."
xcrun simctl boot "$SIM_ID" 2>/dev/null || echo "Simulator already booted"

echo "Opening simulator..."
open /Applications/Xcode.app/Contents/Developer/Applications/Simulator.app/

sleep 2

echo "Installing app..."
xcrun simctl install booted "$APP_DIR"

echo "Launching app..."
xcrun simctl launch --console booted "$BUNDLE_ID"

echo ""
echo "App launched on simulator!"
echo "Check the simulator window."


