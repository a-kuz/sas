#!/bin/bash

set -e

echo "Building standalone executable (self-contained)..."

cargo build --release --bin sas

STANDALONE_DIR="standalone_build"
rm -rf "$STANDALONE_DIR"
mkdir -p "$STANDALONE_DIR"

cp target/release/sas "$STANDALONE_DIR/"

cd "$STANDALONE_DIR"

mkdir -p assets/fonts
cp -r ../assets/fonts/* assets/fonts/

mkdir -p maps
cp -r ../maps/* maps/

mkdir -p q3-resources
cp -r ../q3-resources/* q3-resources/

cd ..

echo "Creating archive..."
if command -v tar &> /dev/null; then
    tar -czf sas_release.tar.gz -C "$STANDALONE_DIR" .
    echo "Created sas_release.tar.gz"
fi

if command -v zip &> /dev/null; then
    cd "$STANDALONE_DIR"
    zip -r ../sas_release.zip .
    cd ..
    echo "Created sas_release.zip"
fi

echo "Standalone build complete!"
echo "Directory: $STANDALONE_DIR/"
du -sh "$STANDALONE_DIR"




