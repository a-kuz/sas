$ErrorActionPreference = "Stop"

$SCRIPT_DIR = Split-Path -Parent $MyInvocation.MyCommand.Path
$PROJECT_ROOT = Split-Path -Parent $SCRIPT_DIR
$Q3_RESOURCES = "q3-resources"
$GAME_BINARY = "sas.exe"

Write-Host "Checking for ImageMagick..." -ForegroundColor Cyan
if (-not (Get-Command magick -ErrorAction SilentlyContinue)) {
    Write-Host "ImageMagick not found. Installing via winget..." -ForegroundColor Yellow
    try {
        winget install --id ImageMagick.ImageMagick -e --silent
        Write-Host "[OK] ImageMagick installed successfully" -ForegroundColor Green
    } catch {
        Write-Host "[WARNING] Failed to install ImageMagick automatically" -ForegroundColor Yellow
        Write-Host "          You can install it manually from: https://imagemagick.org/" -ForegroundColor Yellow
    }
}

$pak0 = "https://github.com/nrempel/q3-server/raw/master/baseq3/pak0.pk3"
$hi_res = "https://files.ioquake3.org/xcsv_hires.zip"
$xpr = "https://github.com/diegoulloao/ioquake3-mac-install/raw/master/extras/extra-pack-resolution.pk3"
$q3_ls = "https://github.com/diegoulloao/ioquake3-mac-install/raw/master/extras/quake3-live-sounds.pk3"
$hd_weapons = "https://github.com/diegoulloao/ioquake3-mac-install/raw/master/extras/hd-weapons.pk3"
$zpack_weapons = "https://github.com/diegoulloao/ioquake3-mac-install/raw/master/extras/zpack-weapons.pk3"
$mappack = "https://cdn.playmorepromode.com/files/cpma-mappack-full.zip"
$cpma = "https://cdn.playmorepromode.com/files/cpma/cpma-1.53-nomaps.zip"

Write-Host "================================================================" -ForegroundColor Cyan
Write-Host "         SAS (Shoot and Strafe) Installer" -ForegroundColor Cyan
Write-Host "================================================================" -ForegroundColor Cyan
Write-Host ""

$answer = Read-Host "Do you own a legal copy of Quake 3 Arena? (yes/no)"

if ($answer -match "^[yY]") {
    Write-Host ""
    Write-Host "[OK] Great! Proceeding with installation..." -ForegroundColor Green
    Write-Host ""
} else {
    Write-Host ""
    Write-Host "[ERROR] You need to own a legal copy of Quake 3 Arena to play this game." -ForegroundColor Red
    Write-Host "        Please purchase it and try again later." -ForegroundColor Red
    Write-Host ""
    exit 1
}

if (-not (Test-Path $Q3_RESOURCES)) {
    New-Item -ItemType Directory -Path $Q3_RESOURCES | Out-Null
}

if (-not (Test-Path "$Q3_RESOURCES\baseq3")) {
    New-Item -ItemType Directory -Path "$Q3_RESOURCES\baseq3" | Out-Null
}

Set-Location "$Q3_RESOURCES\baseq3"

Write-Host "================================================================" -ForegroundColor Cyan
Write-Host "  Step 1/5: Downloading baseq3 resources..." -ForegroundColor Cyan
Write-Host "================================================================" -ForegroundColor Cyan
Write-Host ""

if (-not (Get-Command curl -ErrorAction SilentlyContinue)) {
    Write-Host "[ERROR] curl not found. Please install curl to continue." -ForegroundColor Red
    exit 1
}

if (-not (Test-Path "pak0.pk3")) {
    Write-Host "Downloading pak0.pk3..."
    curl -L --progress-bar $pak0 -o pak0.pk3
    Write-Host "[OK] pak0.pk3 downloaded" -ForegroundColor Green
} else {
    Write-Host "[OK] pak0.pk3 already exists" -ForegroundColor Green
}

for ($i = 1; $i -le 8; $i++) {
    $pakFile = "pak$i.pk3"
    if (-not (Test-Path $pakFile)) {
        Write-Host "Downloading $pakFile..."
        $pak_url = "https://github.com/diegoulloao/ioquake3-mac-install/raw/master/dependencies/baseq3/$pakFile"
        curl -L --progress-bar $pak_url -o $pakFile
        Write-Host "[OK] $pakFile downloaded" -ForegroundColor Green
    } else {
        Write-Host "[OK] $pakFile already exists" -ForegroundColor Green
    }
}

if (-not (Test-Path "xcsv_bq3hi-res.pk3")) {
    Write-Host "Downloading High Resolution Pack..."
    curl -L --progress-bar $hi_res -o xcsv_hires.zip
    Expand-Archive -Path xcsv_hires.zip -DestinationPath . -Force
    Remove-Item xcsv_hires.zip
    Write-Host "[OK] High Resolution Pack installed" -ForegroundColor Green
} else {
    Write-Host "[OK] High Resolution Pack already exists" -ForegroundColor Green
}

if (-not (Test-Path "pak9hqq37test20181106.pk3")) {
    Write-Host "Downloading Extra Pack Resolutions..."
    curl -L --progress-bar $xpr -o pak9hqq37test20181106.pk3
    Write-Host "[OK] Extra Pack Resolutions installed" -ForegroundColor Green
} else {
    Write-Host "[OK] Extra Pack Resolutions already exists" -ForegroundColor Green
}

if (-not (Test-Path "quake3-live-soundpack.pk3")) {
    Write-Host "Downloading Quake3 Live Soundpack..."
    curl -L --progress-bar $q3_ls -o quake3-live-soundpack.pk3
    Write-Host "[OK] Quake3 Live Soundpack installed" -ForegroundColor Green
} else {
    Write-Host "[OK] Quake3 Live Soundpack already exists" -ForegroundColor Green
}

if (-not (Test-Path "pakxy01Tv5.pk3")) {
    Write-Host "Downloading HD Weapons..."
    curl -L --progress-bar $hd_weapons -o pakxy01Tv5.pk3
    Write-Host "[OK] HD Weapons installed" -ForegroundColor Green
} else {
    Write-Host "[OK] HD Weapons already exists" -ForegroundColor Green
}

if (-not (Test-Path "zpack-weapons.pk3")) {
    Write-Host "Downloading ZPack Weapons..."
    curl -L --progress-bar $zpack_weapons -o zpack-weapons.pk3
    Write-Host "[OK] ZPack Weapons installed" -ForegroundColor Green
} else {
    Write-Host "[OK] ZPack Weapons already exists" -ForegroundColor Green
}

if (-not (Test-Path "cpma-mappack-full.pk3")) {
    if (-not (Test-Path "maps")) {
        Write-Host "Downloading CPMA Map-Pack..."
        curl -L --progress-bar $mappack -o cpma-mappack-full.zip
        Expand-Archive -Path cpma-mappack-full.zip -DestinationPath . -Force
        Remove-Item cpma-mappack-full.zip
        Write-Host "[OK] CPMA Map-Pack installed" -ForegroundColor Green
    } else {
        Write-Host "[OK] CPMA Map-Pack already exists" -ForegroundColor Green
    }
} else {
    Write-Host "[OK] CPMA Map-Pack already exists" -ForegroundColor Green
}

Set-Location ".."

if (-not (Test-Path "cpma")) {
    Write-Host "Downloading CPMA Mod..."
    curl -L --progress-bar $cpma -o cpma.zip
    Expand-Archive -Path cpma.zip -DestinationPath . -Force
    Remove-Item cpma.zip
    Write-Host "[OK] CPMA Mod installed" -ForegroundColor Green
} else {
    Write-Host "[OK] CPMA Mod already exists" -ForegroundColor Green
}

Write-Host ""
Write-Host "================================================================" -ForegroundColor Cyan
Write-Host "  Extracting all pk3 files..." -ForegroundColor Cyan
Write-Host "================================================================" -ForegroundColor Cyan
Write-Host ""

Set-Location "baseq3"

Get-ChildItem -Filter "*.pk3" | ForEach-Object {
    Write-Host "Extracting $($_.Name)..."
    Expand-Archive -Path $_.FullName -DestinationPath ".." -Force
}

Write-Host "[OK] All pk3 files extracted" -ForegroundColor Green

Write-Host ""
Write-Host "================================================================" -ForegroundColor Cyan
Write-Host "  Step 2/5: Converting textures to PNG..." -ForegroundColor Cyan
Write-Host "================================================================" -ForegroundColor Cyan
Write-Host ""

if (Get-Command magick -ErrorAction SilentlyContinue) {
    $count = 0
    
    Get-ChildItem -Path ".." -Recurse -Include "*.tga", "*.jpg", "*.jpeg" | ForEach-Object {
        $file = $_.FullName
        $png_file = [System.IO.Path]::ChangeExtension($file, ".png")
        
        if (-not (Test-Path $png_file)) {
            try {
                magick $file $png_file 2>$null
                Write-Host "[OK] Converted: $($_.Name)" -ForegroundColor Green
                $count++
            } catch {
            }
        }
    }
    
    Write-Host ""
    Write-Host "[OK] Converted $count texture files to PNG" -ForegroundColor Green
} else {
    Write-Host "[WARNING] ImageMagick not found" -ForegroundColor Yellow
    Write-Host "          Textures will not be converted. Install ImageMagick to enable conversion." -ForegroundColor Yellow
}

Write-Host ""
Write-Host "================================================================" -ForegroundColor Cyan
Write-Host "  Step 3/5: Downloading latest game release..." -ForegroundColor Cyan
Write-Host "================================================================" -ForegroundColor Cyan
Write-Host ""

if (Get-Command gh -ErrorAction SilentlyContinue) {
    Write-Host "Fetching latest release from GitHub..."
    Set-Location $PROJECT_ROOT
    
    Write-Host "[WARNING] Automatic download not implemented for Windows" -ForegroundColor Yellow
    Write-Host "          Will build from source instead" -ForegroundColor Yellow
} else {
    Write-Host "[WARNING] GitHub CLI (gh) not found, will build from source" -ForegroundColor Yellow
}

Write-Host ""
Write-Host "================================================================" -ForegroundColor Cyan
Write-Host "  Step 4/5: Building game (if needed)..." -ForegroundColor Cyan
Write-Host "================================================================" -ForegroundColor Cyan
Write-Host ""

Set-Location $PROJECT_ROOT

if (-not (Test-Path $GAME_BINARY)) {
    if (-not (Test-Path "target\release\$GAME_BINARY")) {
        Write-Host "Building game from source..."
        
        if (Get-Command cargo -ErrorAction SilentlyContinue) {
            cargo build --release
            
            if (Test-Path "target\release\$GAME_BINARY") {
                Copy-Item "target\release\$GAME_BINARY" $PROJECT_ROOT
                Write-Host "[OK] Game built successfully" -ForegroundColor Green
            } else {
                Write-Host "[ERROR] Build failed" -ForegroundColor Red
                exit 1
            }
        } else {
            Write-Host "[ERROR] Cargo not found. Please install Rust toolchain." -ForegroundColor Red
            exit 1
        }
    } else {
        Write-Host "[OK] Game binary already exists" -ForegroundColor Green
    }
} else {
    Write-Host "[OK] Game binary already exists" -ForegroundColor Green
}

Write-Host ""
Write-Host "================================================================" -ForegroundColor Cyan
Write-Host "  Step 5/5: Launching game..." -ForegroundColor Cyan
Write-Host "================================================================" -ForegroundColor Cyan
Write-Host ""

Set-Location $PROJECT_ROOT

if (Test-Path $GAME_BINARY) {
    Write-Host "[OK] Starting SAS (Shoot and Strafe)..." -ForegroundColor Green
    Write-Host ""
    Start-Process $GAME_BINARY
} elseif (Test-Path "target\release\$GAME_BINARY") {
    Write-Host "[OK] Starting SAS (Shoot and Strafe)..." -ForegroundColor Green
    Write-Host ""
    cargo run --release
} else {
    Write-Host "[ERROR] Game binary not found" -ForegroundColor Red
    exit 1
}

