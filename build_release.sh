#!/bin/bash

set -e

echo "Building release executable with resources..."

cargo build --release --bin sas

RELEASE_DIR="release_package"
rm -rf "$RELEASE_DIR"
mkdir -p "$RELEASE_DIR"

cp target/release/sas "$RELEASE_DIR/"

cp -r assets "$RELEASE_DIR/"
cp -r maps "$RELEASE_DIR/"
cp -r q3-resources "$RELEASE_DIR/"

echo "Package created in $RELEASE_DIR/"
echo "Contents:"
ls -lh "$RELEASE_DIR/"




