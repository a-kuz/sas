#!/bin/bash

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
Q3_RESOURCES="$PROJECT_ROOT/q3-resources"

GITHUB_REPO="a-kuz/sas"
GAME_BINARY="sas"


declare -r pak0="https://github.com/nrempel/q3-server/raw/master/baseq3/pak0.pk3"
declare -r pak="https://github.com/diegoulloao/ioquake3-mac-install/raw/master/dependencies/baseq3/pak@.pk3"
declare -r hi_res="https://files.ioquake3.org/xcsv_hires.zip"
declare -r xpr="https://github.com/diegoulloao/ioquake3-mac-install/raw/master/extras/extra-pack-resolution.pk3"
declare -r q3_ls="https://github.com/diegoulloao/ioquake3-mac-install/raw/master/extras/quake3-live-sounds.pk3"
declare -r hd_weapons="https://github.com/diegoulloao/ioquake3-mac-install/raw/master/extras/hd-weapons.pk3"
declare -r zpack_weapons="https://github.com/diegoulloao/ioquake3-mac-install/raw/master/extras/zpack-weapons.pk3"
declare -r mappack="https://cdn.playmorepromode.com/files/cpma-mappack-full.zip"
declare -r cpma="https://cdn.playmorepromode.com/files/cpma/cpma-1.53-nomaps.zip"

echo "╔════════════════════════════════════════════════════════════╗"
echo "║         SAS (Shoot and Strafe) Installer                  ║"
echo "╚════════════════════════════════════════════════════════════╝"
echo ""
echo "This installer will:"
echo "  • Download Quake 3 Arena resources (baseq3)"
echo "  • Convert textures to PNG format"
echo "  • Download and build SAS game"
echo "  • Launch the game"
echo ""
echo "⚠  Note: You must own a legal copy of Quake 3 Arena."
echo ""

if [ ! -d "$Q3_RESOURCES" ]; then
    mkdir -p "$Q3_RESOURCES"
fi

cd "$Q3_RESOURCES"

if [ ! -d "baseq3" ]; then
    mkdir -p baseq3
fi

cd baseq3

echo "════════════════════════════════════════════════════════════"
echo "  Step 1/5: Downloading baseq3 resources..."
echo "════════════════════════════════════════════════════════════"
echo ""

if [ ! -f "pak0.pk3" ]; then
    echo "→ Downloading pak0.pk3..."
    curl -L --progress-bar $pak0 > pak0.pk3
    echo "✓ pak0.pk3 downloaded"
else
    echo "✓ pak0.pk3 already exists"
fi

for i in {1..8}; do
    if [ ! -f "pak$i.pk3" ]; then
        echo "→ Downloading pak$i.pk3..."
        curl -L --progress-bar "${pak/@/$i}" > "pak$i.pk3"
        echo "✓ pak$i.pk3 downloaded"
    else
        echo "✓ pak$i.pk3 already exists"
    fi
done

if [ ! -f "xcsv_hires.pk3" ]; then
    echo "→ Downloading High Resolution Pack..."
    curl -L --progress-bar $hi_res > xcsv_hires.zip
    unzip -q -o xcsv_hires.zip
    rm -f xcsv_hires.zip
    echo "✓ High Resolution Pack installed"
else
    echo "✓ High Resolution Pack already exists"
fi

if [ ! -f "pak9hqq37test20181106.pk3" ]; then
    echo "→ Downloading Extra Pack Resolutions..."
    curl -L --progress-bar $xpr > pak9hqq37test20181106.pk3
    echo "✓ Extra Pack Resolutions installed"
else
    echo "✓ Extra Pack Resolutions already exists"
fi

if [ ! -f "quake3-live-soundpack.pk3" ]; then
    echo "→ Downloading Quake3 Live Soundpack..."
    curl -L --progress-bar $q3_ls > quake3-live-soundpack.pk3
    echo "✓ Quake3 Live Soundpack installed"
else
    echo "✓ Quake3 Live Soundpack already exists"
fi

if [ ! -f "pakxy01Tv5.pk3" ]; then
    echo "→ Downloading HD Weapons..."
    curl -L --progress-bar $hd_weapons > pakxy01Tv5.pk3
    echo "✓ HD Weapons installed"
else
    echo "✓ HD Weapons already exists"
fi

if [ ! -f "zpack-weapons.pk3" ]; then
    echo "→ Downloading ZPack Weapons..."
    curl -L --progress-bar $zpack_weapons > zpack-weapons.pk3
    echo "✓ ZPack Weapons installed"
else
    echo "✓ ZPack Weapons already exists"
fi

if [ ! -f "cpma-mappack-full.pk3" ] && [ ! -d "maps" ]; then
    echo "→ Downloading CPMA Map-Pack..."
    curl -L --progress-bar $mappack > cpma-mappack-full.zip
    unzip -q -o -d . cpma-mappack-full.zip
    rm -f cpma-mappack-full.zip
    echo "✓ CPMA Map-Pack installed"
else
    echo "✓ CPMA Map-Pack already exists"
fi

cd "$Q3_RESOURCES"

if [ ! -d "cpma" ]; then
    echo "→ Downloading CPMA Mod..."
    curl -L --progress-bar $cpma > cpma.zip
    unzip -q -o cpma.zip
    rm -f cpma.zip
    echo "✓ CPMA Mod installed"
else
    echo "✓ CPMA Mod already exists"
fi

echo ""
echo "════════════════════════════════════════════════════════════"
echo "  Step 2/5: Converting textures to PNG..."
echo "════════════════════════════════════════════════════════════"
echo ""

cd "$PROJECT_ROOT"

if command -v sips &> /dev/null || command -v convert &> /dev/null; then
    count=0
    
    for ext in tga jpg jpeg; do
        while IFS= read -r -d '' file; do
            png_file="${file%.*}.png"
            
            if [ ! -f "$png_file" ]; then
                if command -v sips &> /dev/null; then
                    sips -s format png "$file" --out "$png_file" > /dev/null 2>&1
                elif command -v convert &> /dev/null; then
                    convert "$file" "$png_file" 2>/dev/null
                fi
                
                if [ $? -eq 0 ]; then
                    echo "✓ Converted: $(basename "$file") → $(basename "$png_file")"
                    ((count++))
                fi
            fi
        done < <(find "$Q3_RESOURCES" -name "*.$ext" -type f -print0)
    done
    
    echo ""
    echo "✓ Converted $count texture files to PNG"
else
    echo "⚠ Warning: Neither 'sips' nor 'convert' (ImageMagick) found"
    echo "  Textures will not be converted. Install ImageMagick to enable conversion."
fi

echo ""
echo "════════════════════════════════════════════════════════════"
echo "  Step 3/5: Downloading latest game release..."
echo "════════════════════════════════════════════════════════════"
echo ""

if command -v gh &> /dev/null; then
    echo "→ Fetching latest release from GitHub..."
    
    cd "$PROJECT_ROOT"
    
    LATEST_RELEASE=$(gh release view --repo "$GITHUB_REPO" --json tagName -q .tagName 2>/dev/null || echo "")
    
    if [ -n "$LATEST_RELEASE" ]; then
        echo "✓ Found release: $LATEST_RELEASE"
        
        OS=$(uname -s)
        ARCH=$(uname -m)
        
        case "$OS" in
            Darwin)
                ASSET_PATTERN="*macos*.tar.gz"
                ;;
            Linux)
                ASSET_PATTERN="*linux*.tar.gz"
                ;;
            *)
                echo "⚠ Unsupported OS: $OS"
                ASSET_PATTERN=""
                ;;
        esac
        
        if [ -n "$ASSET_PATTERN" ]; then
            echo "→ Downloading release for $OS..."
            gh release download "$LATEST_RELEASE" --repo "$GITHUB_REPO" --pattern "$ASSET_PATTERN" --clobber 2>/dev/null || true
            
            if ls *tar.gz 1> /dev/null 2>&1; then
                tar -xzf *.tar.gz 2>/dev/null || tar -xzf *.tar.gz --no-same-owner 2>/dev/null || true
                rm -f *.tar.gz
                if [ -f "$GAME_BINARY" ]; then
                    echo "✓ Game binary downloaded and extracted"
                else
                    echo "⚠ Extraction failed, will build from source"
                fi
            else
                echo "⚠ No matching release found, will build from source"
            fi
        fi
    else
        echo "⚠ No releases found, will build from source"
    fi
else
    echo "⚠ GitHub CLI (gh) not found, will build from source"
fi

echo ""
echo "════════════════════════════════════════════════════════════"
echo "  Step 4/5: Building game (if needed)..."
echo "════════════════════════════════════════════════════════════"
echo ""

cd "$PROJECT_ROOT"

if [ ! -f "$GAME_BINARY" ] && [ ! -f "target/release/$GAME_BINARY" ]; then
    if [ ! -f "Cargo.toml" ]; then
        echo "→ Cloning repository..."
        
        if command -v git &> /dev/null; then
            git clone "https://github.com/$GITHUB_REPO.git" sas-source
            cd sas-source
            echo "✓ Repository cloned"
        else
            echo "✗ Git not found. Cannot clone repository."
            exit 1
        fi
    fi
    
    echo "→ Building game from source..."
    
    if command -v cargo &> /dev/null; then
        cargo build --release
        
        if [ -f "target/release/$GAME_BINARY" ]; then
            cp "target/release/$GAME_BINARY" "$PROJECT_ROOT/"
            echo "✓ Game built successfully"
        else
            echo "✗ Build failed"
            exit 1
        fi
    else
        echo "✗ Cargo not found."
        echo ""
        echo "Please install Rust toolchain:"
        echo "  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
        echo ""
        echo "Or wait for GitHub Actions to build releases (check: https://github.com/$GITHUB_REPO/releases)"
        exit 1
    fi
else
    echo "✓ Game binary already exists"
fi

echo ""
echo "════════════════════════════════════════════════════════════"
echo "  Step 5/5: Launching game..."
echo "════════════════════════════════════════════════════════════"
echo ""

cd "$PROJECT_ROOT"

if [ -f "$GAME_BINARY" ]; then
    chmod +x "$GAME_BINARY"
    echo "✓ Starting SAS (Shoot and Strafe)..."
    echo ""
    ./"$GAME_BINARY"
elif [ -f "sas-source/$GAME_BINARY" ]; then
    cd sas-source
    chmod +x "$GAME_BINARY"
    echo "✓ Starting SAS (Shoot and Strafe)..."
    echo ""
    ./"$GAME_BINARY"
elif [ -f "sas-source/target/release/$GAME_BINARY" ]; then
    cd sas-source
    chmod +x "target/release/$GAME_BINARY"
    echo "✓ Starting SAS (Shoot and Strafe)..."
    echo ""
    ./target/release/"$GAME_BINARY"
else
    echo "✗ Game binary not found"
    exit 1
fi


