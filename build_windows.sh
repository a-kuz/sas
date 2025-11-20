#!/bin/bash

set -e

export PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:$PATH"

echo "Building for Windows (x86_64-pc-windows-gnu)..."

export CARGO_TARGET_X86_64_PC_WINDOWS_GNU_LINKER=x86_64-w64-mingw32-gcc

cargo build --release --target x86_64-pc-windows-gnu --bin sas

WINDOWS_DIR="windows_release"
rm -rf "$WINDOWS_DIR"
mkdir -p "$WINDOWS_DIR"

cp target/x86_64-pc-windows-gnu/release/sas.exe "$WINDOWS_DIR/"

cp -r assets "$WINDOWS_DIR/"
cp -r maps "$WINDOWS_DIR/"
cp -r q3-resources "$WINDOWS_DIR/"

echo "Creating Windows archive..."
cd "$WINDOWS_DIR"
zip -r ../sas_windows.zip .
cd ..

echo "Windows build complete!"
echo "Directory: $WINDOWS_DIR/"
echo "Archive: sas_windows.zip"
du -sh "$WINDOWS_DIR"
ls -lh sas_windows.zip

