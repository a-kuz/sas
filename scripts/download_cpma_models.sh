#!/bin/bash

set -e

MODELS_DIR="q3-resources/models/players"
TEMP_DIR="/tmp/cpma_models"

echo "=== Downloading CPMA Player Models ==="

mkdir -p "$TEMP_DIR"
mkdir -p "$MODELS_DIR"

CPMA_MODELS=(
    "pm"
    "pm/bright"
    "pm/dark"
)

echo "Note: CPMA models require pak files from cpma-mappack-full.pk3"
echo "You need to extract models from CPMA distribution or pak files"
echo ""
echo "Common CPMA model locations:"
echo "  - models/players/pm/ (ProMode)"
echo "  - models/players/sarge_pm/ (CPMA Sarge)"
echo "  - models/players/keel_pm/ (CPMA Keel)"
echo ""
echo "To add CPMA models manually:"
echo "1. Download CPMA from playmorepromode.com"
echo "2. Extract .pk3 files from cpma/pak*.pk3"
echo "3. Copy models/players/* to $MODELS_DIR/"
echo ""

read -p "Do you have CPMA pak files to extract? (y/n) " -n 1 -r
echo

if [[ $REPLY =~ ^[Yy]$ ]]; then
    read -p "Enter path to CPMA pak file (e.g., cpma-mappack-full.pk3): " PAK_PATH
    
    if [ -f "$PAK_PATH" ]; then
        echo "Extracting models from $PAK_PATH..."
        
        pushd "$TEMP_DIR" > /dev/null
        unzip -q "$PAK_PATH" "models/players/*" 2>/dev/null || echo "Extraction complete (some warnings are normal)"
        popd > /dev/null
        
        if [ -d "$TEMP_DIR/models/players" ]; then
            echo "Copying models to $MODELS_DIR..."
            cp -r "$TEMP_DIR/models/players/"* "$MODELS_DIR/" 2>/dev/null || true
            
            echo "✓ Models copied successfully!"
            echo ""
            echo "Available CPMA models:"
            ls -1 "$MODELS_DIR" | grep -E "(pm|_pm)" || echo "No CPMA-specific models found"
        else
            echo "✗ No models found in pak file"
        fi
    else
        echo "✗ File not found: $PAK_PATH"
        exit 1
    fi
fi

echo ""
echo "=== Setup Complete ==="
echo "Your current models:"
ls -1 "$MODELS_DIR" | head -20

rm -rf "$TEMP_DIR"

