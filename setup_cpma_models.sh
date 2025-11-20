#!/bin/bash

set -e

MODELS_DIR="q3-resources/models/players"
TEMP_DIR="/tmp/cpma_models_setup"
CPMA_VERSION="1.52"

echo "═══════════════════════════════════════════════════════════"
echo "       CPMA Player Models Setup Utility"
echo "═══════════════════════════════════════════════════════════"
echo ""

mkdir -p "$TEMP_DIR"
mkdir -p "$MODELS_DIR"

show_menu() {
    echo ""
    echo "Choose installation method:"
    echo ""
    echo "  1) Extract from local CPMA pak file"
    echo "  2) Download CPMA from official source (requires wget/curl)"
    echo "  3) Manual installation instructions"
    echo "  4) Test existing models"
    echo "  5) Exit"
    echo ""
    read -p "Enter choice [1-5]: " choice
    echo ""
}

extract_from_pak() {
    read -p "Enter path to CPMA pak file (e.g., /path/to/cpma-mappack-full.pk3): " PAK_PATH
    
    if [ ! -f "$PAK_PATH" ]; then
        echo "✗ File not found: $PAK_PATH"
        return 1
    fi
    
    echo "Extracting player models from $PAK_PATH..."
    
    pushd "$TEMP_DIR" > /dev/null
    
    unzip -q "$PAK_PATH" "models/players/*" 2>/dev/null || {
        echo "Note: Some extraction warnings are normal"
    }
    
    popd > /dev/null
    
    if [ -d "$TEMP_DIR/models/players" ]; then
        echo ""
        echo "Found models:"
        ls -1 "$TEMP_DIR/models/players"
        echo ""
        
        for model_dir in "$TEMP_DIR/models/players"/*; do
            if [ -d "$model_dir" ]; then
                model_name=$(basename "$model_dir")
                
                if [ -f "$model_dir/lower.md3" ] && [ -f "$model_dir/upper.md3" ] && [ -f "$model_dir/head.md3" ]; then
                    echo "✓ Copying $model_name..."
                    cp -r "$model_dir" "$MODELS_DIR/" 2>/dev/null || {
                        echo "⚠ Failed to copy $model_name (may already exist)"
                    }
                else
                    echo "⚠ Skipping $model_name (incomplete model)"
                fi
            fi
        done
        
        echo ""
        echo "✓✓✓ Extraction complete!"
        echo ""
        echo "Testing models..."
        cargo run --bin test_model list
    else
        echo "✗ No player models found in pak file"
        return 1
    fi
}

download_cpma() {
    echo "CPMA Download Options:"
    echo ""
    echo "Official sources:"
    echo "  • https://playmorepromode.com/"
    echo "  • https://github.com/defraged/cpma"
    echo ""
    
    CPMA_URL="https://cdn.playmorepromode.com/files/cpma-1.52-nomaps.zip"
    
    echo "Attempting to download CPMA ${CPMA_VERSION}..."
    echo "URL: $CPMA_URL"
    echo ""
    
    if command -v curl &> /dev/null; then
        curl -L -o "$TEMP_DIR/cpma.zip" "$CPMA_URL" || {
            echo "✗ Download failed. Please download manually from playmorepromode.com"
            return 1
        }
    elif command -v wget &> /dev/null; then
        wget -O "$TEMP_DIR/cpma.zip" "$CPMA_URL" || {
            echo "✗ Download failed. Please download manually from playmorepromode.com"
            return 1
        }
    else
        echo "✗ Neither curl nor wget found. Please install one or download manually."
        return 1
    fi
    
    echo "✓ Downloaded CPMA"
    echo "Extracting..."
    
    pushd "$TEMP_DIR" > /dev/null
    unzip -q cpma.zip 2>/dev/null || true
    popd > /dev/null
    
    echo "Looking for pak files..."
    find "$TEMP_DIR" -name "*.pk3" -type f
    
    echo ""
    read -p "Enter path to pak file from above (or press Enter to skip): " selected_pak
    
    if [ -n "$selected_pak" ] && [ -f "$selected_pak" ]; then
        PAK_PATH="$selected_pak"
        extract_from_pak
    else
        echo "Extraction skipped"
    fi
}

manual_instructions() {
    cat << 'EOF'
═══════════════════════════════════════════════════════════
                Manual Installation Guide
═══════════════════════════════════════════════════════════

CPMA Models Installation:

1. Download CPMA
   Visit: https://playmorepromode.com/
   Download: cpma-1.52-nomaps.zip or cpma-mappack-full.zip

2. Extract Models
   CPMA files are .pk3 files (ZIP archives)
   
   Method A - Command Line:
     unzip cpma/pak*.pk3 "models/players/*" -d temp/
   
   Method B - GUI:
     - Rename .pk3 to .zip
     - Extract using your archive manager
     - Navigate to models/players/

3. Copy Models
   Copy extracted models/players/* to:
   q3-resources/models/players/

4. Verify Installation
   cargo run --bin test_model list

Popular CPMA Models:
  • pm          - ProMode default model
  • sarge_pm    - CPMA Sarge variant
  • keel_pm     - CPMA Keel variant  
  • crash_pm    - CPMA Crash variant

Alternative Sources:
  • LvL World: https://lvlworld.com/ (search "cpma models")
  • GameBanana: https://gamebanana.com/games/24
  • Quake3World: https://www.quake3world.com/

Community Models:
  Many custom CPMA models are available from the community.
  They use the same MD3 format and directory structure.

Model Structure:
  models/players/MODEL_NAME/
    ├── lower.md3           (legs)
    ├── upper.md3           (torso)
    ├── head.md3            (head)
    ├── animation.cfg       (animations)
    ├── lower_*.skin        (skin mappings)
    ├── upper_*.skin
    ├── head_*.skin
    └── *.tga/*.png         (textures)

═══════════════════════════════════════════════════════════
EOF
}

test_models() {
    echo "Running model tests..."
    echo ""
    cargo run --bin test_model list
    echo ""
    read -p "Test specific model? (enter model name or press Enter to skip): " test_model
    
    if [ -n "$test_model" ]; then
        cargo run --bin test_model "$test_model"
    fi
}

main_loop() {
    while true; do
        show_menu
        
        case $choice in
            1)
                extract_from_pak
                ;;
            2)
                download_cpma
                ;;
            3)
                manual_instructions
                ;;
            4)
                test_models
                ;;
            5)
                echo "Exiting..."
                break
                ;;
            *)
                echo "Invalid choice. Please enter 1-5."
                ;;
        esac
        
        echo ""
        read -p "Press Enter to continue..."
    done
}

main_loop

rm -rf "$TEMP_DIR"

echo ""
echo "═══════════════════════════════════════════════════════════"
echo "Setup complete!"
echo ""
echo "To use a model:"
echo "  export NFK_PLAYER_MODEL=pm"
echo "  cargo run"
echo "═══════════════════════════════════════════════════════════"

