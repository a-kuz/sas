#!/bin/bash

set -e

GITHUB_REPO="a-kuz/sas"
GAME_BINARY="sas"

INSTALL_DIR="$HOME/Library/Application Support/SAS"
Q3_RESOURCES="$INSTALL_DIR/q3-resources"


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
echo "Installation directory: $INSTALL_DIR"
echo ""
echo "This installer will:"
echo "  • Download Quake 3 Arena resources (baseq3)"
echo "  • Convert textures to PNG format"
echo "  • Download and build SAS game"
echo "  • Create app in Applications folder"
echo "  • Launch the game"
echo ""
echo "⚠  Note: You must own a legal copy of Quake 3 Arena."
echo ""

mkdir -p "$INSTALL_DIR"

mkdir -p "$Q3_RESOURCES"
cd "$Q3_RESOURCES"

TEMP_DIR="$Q3_RESOURCES/.temp"
mkdir -p "$TEMP_DIR"

echo "════════════════════════════════════════════════════════════"
echo "  Step 1/5: Downloading and extracting baseq3 resources..."
echo "════════════════════════════════════════════════════════════"
echo ""

if [ ! -f ".installed" ]; then
    for i in {0..8}; do
        echo "→ Downloading pak$i.pk3..."
        if [ $i -eq 0 ]; then
            curl -L --progress-bar $pak0 > "$TEMP_DIR/pak$i.pk3"
        else
            curl -L --progress-bar "${pak/@/$i}" > "$TEMP_DIR/pak$i.pk3"
        fi
        echo "→ Extracting pak$i.pk3..."
        unzip -q -o "$TEMP_DIR/pak$i.pk3" -d "$Q3_RESOURCES"
        rm -f "$TEMP_DIR/pak$i.pk3"
        echo "✓ pak$i.pk3 processed"
    done

    echo "→ Downloading High Resolution Pack..."
    curl -L --progress-bar $hi_res > "$TEMP_DIR/xcsv_hires.zip"
    unzip -q -o "$TEMP_DIR/xcsv_hires.zip" -d "$Q3_RESOURCES"
    rm -f "$TEMP_DIR/xcsv_hires.zip"
    echo "✓ High Resolution Pack installed"

    echo "→ Downloading Extra Pack Resolutions..."
    curl -L --progress-bar $xpr > "$TEMP_DIR/pak9hqq37test20181106.pk3"
    unzip -q -o "$TEMP_DIR/pak9hqq37test20181106.pk3" -d "$Q3_RESOURCES"
    rm -f "$TEMP_DIR/pak9hqq37test20181106.pk3"
    echo "✓ Extra Pack Resolutions installed"

    echo "→ Downloading Quake3 Live Soundpack..."
    curl -L --progress-bar $q3_ls > "$TEMP_DIR/quake3-live-soundpack.pk3"
    unzip -q -o "$TEMP_DIR/quake3-live-soundpack.pk3" -d "$Q3_RESOURCES"
    rm -f "$TEMP_DIR/quake3-live-soundpack.pk3"
    echo "✓ Quake3 Live Soundpack installed"

    echo "→ Downloading HD Weapons..."
    curl -L --progress-bar $hd_weapons > "$TEMP_DIR/pakxy01Tv5.pk3"
    unzip -q -o "$TEMP_DIR/pakxy01Tv5.pk3" -d "$Q3_RESOURCES"
    rm -f "$TEMP_DIR/pakxy01Tv5.pk3"
    echo "✓ HD Weapons installed"

    echo "→ Downloading ZPack Weapons..."
    curl -L --progress-bar $zpack_weapons > "$TEMP_DIR/zpack-weapons.pk3"
    unzip -q -o "$TEMP_DIR/zpack-weapons.pk3" -d "$Q3_RESOURCES"
    rm -f "$TEMP_DIR/zpack-weapons.pk3"
    echo "✓ ZPack Weapons installed"

    echo "→ Downloading CPMA Map-Pack..."
    curl -L --progress-bar $mappack > "$TEMP_DIR/cpma-mappack-full.zip"
    unzip -q -o "$TEMP_DIR/cpma-mappack-full.zip" -d "$Q3_RESOURCES"
    rm -f "$TEMP_DIR/cpma-mappack-full.zip"
    echo "✓ CPMA Map-Pack installed"

    echo "→ Downloading CPMA Mod..."
    curl -L --progress-bar $cpma > "$TEMP_DIR/cpma.zip"
    unzip -q -o "$TEMP_DIR/cpma.zip" -d "$Q3_RESOURCES"
    rm -f "$TEMP_DIR/cpma.zip"
    echo "✓ CPMA Mod installed"
    
    rm -rf "$TEMP_DIR"
    touch ".installed"
    echo ""
    echo "✓ All resources extracted to: $Q3_RESOURCES"
else
    echo "✓ Resources already installed (found .installed marker)"
fi

echo ""
echo "════════════════════════════════════════════════════════════"
echo "  Step 2/5: Converting textures to PNG..."
echo "════════════════════════════════════════════════════════════"
echo ""

cd "$Q3_RESOURCES"

if [ ! -f ".converted" ]; then
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
                        echo "✓ Converted: $(basename "$file")"
                        ((count++))
                    fi
                fi
            done < <(find "$Q3_RESOURCES" -name "*.$ext" -type f -print0)
        done
        
        touch ".converted"
        echo ""
        echo "✓ Converted $count texture files to PNG"
    else
        echo "⚠ Neither 'sips' nor 'convert' found, skipping texture conversion"
    fi
else
    echo "✓ Textures already converted (found .converted marker)"
fi

echo ""
echo "════════════════════════════════════════════════════════════"
echo "  Step 3/5: Downloading latest game release..."
echo "════════════════════════════════════════════════════════════"
echo ""

if command -v gh &> /dev/null; then
    echo "→ Fetching latest release from GitHub..."
    
    cd "$INSTALL_DIR"
    
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
                tar -xzf *.tar.gz --strip-components=1 2>/dev/null || tar -xzf *.tar.gz --strip-components=1 --no-same-owner 2>/dev/null || true
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

cd "$INSTALL_DIR"

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
            cp "target/release/$GAME_BINARY" "$INSTALL_DIR/"
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
echo "  Step 5/6: Creating Application..."
echo "════════════════════════════════════════════════════════════"
echo ""

cd "$INSTALL_DIR"

if [ -f "sas-source/$GAME_BINARY" ]; then
    cp "sas-source/$GAME_BINARY" .
    cp -r sas-source/assets .
    cp -r sas-source/maps .
    chmod +x "$GAME_BINARY"
elif [ -f "sas-source/target/release/$GAME_BINARY" ]; then
    cp "sas-source/target/release/$GAME_BINARY" .
    cp -r sas-source/assets .
    cp -r sas-source/maps .
    chmod +x "$GAME_BINARY"
elif [ ! -f "$GAME_BINARY" ]; then
    echo "✗ Game binary not found"
    exit 1
fi

echo "✓ Game files copied"

APP_DIR="$HOME/Applications/SAS.app"
mkdir -p "$APP_DIR/Contents/MacOS"
mkdir -p "$APP_DIR/Contents/Resources"

if [ ! -L "$INSTALL_DIR/q3-resources" ]; then
    ln -sf "$HOME/Library/Application Support/SAS/q3-resources" "$INSTALL_DIR/q3-resources" 2>/dev/null || true
fi

cat > "$APP_DIR/Contents/MacOS/sas-launcher" << 'EOF'
#!/bin/bash
cd "$HOME/Library/Application Support/SAS"
if [ ! -e "q3-resources" ]; then
    ln -sf "$HOME/Library/Application Support/SAS/q3-resources" "q3-resources"
fi
./sas
EOF

chmod +x "$APP_DIR/Contents/MacOS/sas-launcher"

cat > "$APP_DIR/Contents/Info.plist" << EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleExecutable</key>
    <string>sas-launcher</string>
    <key>CFBundleIdentifier</key>
    <string>com.akuz.sas</string>
    <key>CFBundleName</key>
    <string>SAS</string>
    <key>CFBundleDisplayName</key>
    <string>SAS III</string>
    <key>CFBundleVersion</key>
    <string>0.0.1</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
</dict>
</plist>
EOF

echo "✓ Application created: $APP_DIR"

echo ""
echo "════════════════════════════════════════════════════════════"
echo "  Step 6/6: Launching game..."
echo "════════════════════════════════════════════════════════════"
echo ""
echo "✓ Installation complete!"
echo "  • Game installed to: $INSTALL_DIR"
echo "  • App available in: ~/Applications/SAS.app"
echo ""
echo "✓ Starting SAS (Shoot and Strafe)..."
echo ""

cd "$INSTALL_DIR"
./"$GAME_BINARY"


