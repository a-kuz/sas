#!/bin/bash

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

cd "$PROJECT_ROOT"

VERSION=$(grep '^version = ' Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/')
RELEASE_TAG="v$VERSION"
RELEASE_DIR="release_builds"

echo "╔════════════════════════════════════════════════════════════╗"
echo "║         SAS Release Builder v$VERSION                        "
echo "╚════════════════════════════════════════════════════════════╝"
echo ""

rm -rf "$RELEASE_DIR"
mkdir -p "$RELEASE_DIR"

OS=$(uname -s)
ARCH=$(uname -m)

case "$ARCH" in
    x86_64)
        ARCH_NAME="x86_64"
        ;;
    arm64|aarch64)
        ARCH_NAME="arm64"
        ;;
    *)
        ARCH_NAME="$ARCH"
        ;;
esac

case "$OS" in
    Darwin)
        OS_NAME="macos"
        BINARY_NAME="sas"
        ;;
    Linux)
        OS_NAME="linux"
        BINARY_NAME="sas"
        ;;
    MINGW*|MSYS*|CYGWIN*)
        OS_NAME="windows"
        BINARY_NAME="sas.exe"
        ;;
    *)
        OS_NAME="unknown"
        BINARY_NAME="sas"
        ;;
esac

PACKAGE_NAME="sas-${VERSION}-${OS_NAME}-${ARCH_NAME}"
PACKAGE_DIR="$RELEASE_DIR/$PACKAGE_NAME"

echo "════════════════════════════════════════════════════════════"
echo "  Building for: $OS_NAME-$ARCH_NAME"
echo "════════════════════════════════════════════════════════════"
echo ""

echo "→ Building release binary..."
cargo build --release --bin sas

echo "→ Creating package directory..."
mkdir -p "$PACKAGE_DIR"

echo "→ Copying binary..."
cp "target/release/$BINARY_NAME" "$PACKAGE_DIR/"

echo "→ Copying assets..."
cp -r assets "$PACKAGE_DIR/"

echo "→ Copying maps..."
cp -r maps "$PACKAGE_DIR/"

echo "→ Copying config..."
if [ -f "sas_config.cfg" ]; then
    cp sas_config.cfg "$PACKAGE_DIR/"
fi

echo "→ Creating archive..."
cd "$RELEASE_DIR"
tar -czf "${PACKAGE_NAME}.tar.gz" "$PACKAGE_NAME"
rm -rf "$PACKAGE_NAME"

cd "$PROJECT_ROOT"

ARCHIVE_SIZE=$(du -h "$RELEASE_DIR/${PACKAGE_NAME}.tar.gz" | cut -f1)

echo ""
echo "✓ Release package created:"
echo "  File: $RELEASE_DIR/${PACKAGE_NAME}.tar.gz"
echo "  Size: $ARCHIVE_SIZE"
echo ""


echo "════════════════════════════════════════════════════════════"
echo "  Release Summary"
echo "════════════════════════════════════════════════════════════"
echo ""
echo "Version: $VERSION"
echo "Tag: $RELEASE_TAG"
echo ""
echo "Packages created:"
ls -lh "$RELEASE_DIR"/*.tar.gz | awk '{print "  " $9 " (" $5 ")"}'
echo ""
echo "To publish this release:"
echo "  ./scripts/publish_release.sh"
echo ""

