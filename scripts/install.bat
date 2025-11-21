@echo off
setlocal enabledelayedexpansion

set "SCRIPT_DIR=%~dp0"
set "PROJECT_ROOT=%SCRIPT_DIR%.."
set "Q3_RESOURCES=%PROJECT_ROOT%\q3-resources"
set "GAME_BINARY=sas.exe"

set "pak0=https://github.com/nrempel/q3-server/raw/master/baseq3/pak0.pk3"
set "hi_res=https://files.ioquake3.org/xcsv_hires.zip"
set "xpr=https://github.com/diegoulloao/ioquake3-mac-install/raw/master/extras/extra-pack-resolution.pk3"
set "q3_ls=https://github.com/diegoulloao/ioquake3-mac-install/raw/master/extras/quake3-live-sounds.pk3"
set "hd_weapons=https://github.com/diegoulloao/ioquake3-mac-install/raw/master/extras/hd-weapons.pk3"
set "zpack_weapons=https://github.com/diegoulloao/ioquake3-mac-install/raw/master/extras/zpack-weapons.pk3"
set "mappack=https://cdn.playmorepromode.com/files/cpma-mappack-full.zip"
set "cpma=https://cdn.playmorepromode.com/files/cpma/cpma-1.53-nomaps.zip"

echo ================================================================
echo          SAS (Shoot and Strafe) Installer
echo ================================================================
echo.

set /p answer="Do you own a legal copy of Quake 3 Arena? (yes/no): "

if /i "%answer:~0,1%"=="y" (
    echo.
    echo [OK] Great! Proceeding with installation...
    echo.
) else (
    echo.
    echo [ERROR] You need to own a legal copy of Quake 3 Arena to play this game.
    echo         Please purchase it and try again later.
    echo.
    exit /b 1
)

if not exist "%Q3_RESOURCES%" mkdir "%Q3_RESOURCES%"
if not exist "%Q3_RESOURCES%\baseq3" mkdir "%Q3_RESOURCES%\baseq3"

cd /d "%Q3_RESOURCES%\baseq3"

echo ================================================================
echo   Step 1/5: Downloading baseq3 resources...
echo ================================================================
echo.

where curl >nul 2>nul
if %errorlevel% neq 0 (
    echo [ERROR] curl not found. Please install curl to continue.
    exit /b 1
)

if not exist "pak0.pk3" (
    echo Downloading pak0.pk3...
    curl -L --progress-bar "%pak0%" -o pak0.pk3
    echo [OK] pak0.pk3 downloaded
) else (
    echo [OK] pak0.pk3 already exists
)

for /l %%i in (1,1,8) do (
    if not exist "pak%%i.pk3" (
        echo Downloading pak%%i.pk3...
        set "pak_url=https://github.com/diegoulloao/ioquake3-mac-install/raw/master/dependencies/baseq3/pak%%i.pk3"
        curl -L --progress-bar "!pak_url!" -o "pak%%i.pk3"
        echo [OK] pak%%i.pk3 downloaded
    ) else (
        echo [OK] pak%%i.pk3 already exists
    )
)

if not exist "xcsv_bq3hi-res.pk3" (
    echo Downloading High Resolution Pack...
    curl -L --progress-bar "%hi_res%" -o xcsv_hires.zip
    tar -xf xcsv_hires.zip
    del xcsv_hires.zip
    echo [OK] High Resolution Pack installed
) else (
    echo [OK] High Resolution Pack already exists
)

if not exist "pak9hqq37test20181106.pk3" (
    echo Downloading Extra Pack Resolutions...
    curl -L --progress-bar "%xpr%" -o pak9hqq37test20181106.pk3
    echo [OK] Extra Pack Resolutions installed
) else (
    echo [OK] Extra Pack Resolutions already exists
)

if not exist "quake3-live-soundpack.pk3" (
    echo Downloading Quake3 Live Soundpack...
    curl -L --progress-bar "%q3_ls%" -o quake3-live-soundpack.pk3
    echo [OK] Quake3 Live Soundpack installed
) else (
    echo [OK] Quake3 Live Soundpack already exists
)

if not exist "pakxy01Tv5.pk3" (
    echo Downloading HD Weapons...
    curl -L --progress-bar "%hd_weapons%" -o pakxy01Tv5.pk3
    echo [OK] HD Weapons installed
) else (
    echo [OK] HD Weapons already exists
)

if not exist "zpack-weapons.pk3" (
    echo Downloading ZPack Weapons...
    curl -L --progress-bar "%zpack_weapons%" -o zpack-weapons.pk3
    echo [OK] ZPack Weapons installed
) else (
    echo [OK] ZPack Weapons already exists
)

if not exist "cpma-mappack-full.pk3" (
    if not exist "maps" (
        echo Downloading CPMA Map-Pack...
        curl -L --progress-bar "%mappack%" -o cpma-mappack-full.zip
        tar -xf cpma-mappack-full.zip
        del cpma-mappack-full.zip
        echo [OK] CPMA Map-Pack installed
    ) else (
        echo [OK] CPMA Map-Pack already exists
    )
) else (
    echo [OK] CPMA Map-Pack already exists
)

cd /d "%Q3_RESOURCES%"

if not exist "cpma" (
    echo Downloading CPMA Mod...
    curl -L --progress-bar "%cpma%" -o cpma.zip
    tar -xf cpma.zip
    del cpma.zip
    echo [OK] CPMA Mod installed
) else (
    echo [OK] CPMA Mod already exists
)

echo.
echo ================================================================
echo   Extracting all pk3 files...
echo ================================================================
echo.

cd /d "%Q3_RESOURCES%\baseq3"

for %%f in (*.pk3) do (
    echo Extracting %%f...
    tar -xf "%%f" -C "%Q3_RESOURCES%"
)

echo [OK] All pk3 files extracted

echo.
echo ================================================================
echo   Step 2/5: Converting textures to PNG...
echo ================================================================
echo.

where magick >nul 2>nul
if %errorlevel% equ 0 (
    set count=0
    
    for /r "%Q3_RESOURCES%" %%f in (*.tga *.jpg *.jpeg) do (
        set "file=%%f"
        set "png_file=!file:~0,-4!.png"
        
        if not exist "!png_file!" (
            magick "!file!" "!png_file!" 2>nul
            if !errorlevel! equ 0 (
                echo [OK] Converted: %%~nxf
                set /a count+=1
            )
        )
    )
    
    echo.
    echo [OK] Converted !count! texture files to PNG
) else (
    echo [WARNING] ImageMagick not found
    echo           Textures will not be converted. Install ImageMagick to enable conversion.
)

echo.
echo ================================================================
echo   Step 3/5: Downloading latest game release...
echo ================================================================
echo.

where gh >nul 2>nul
if %errorlevel% equ 0 (
    echo Fetching latest release from GitHub...
    cd /d "%PROJECT_ROOT%"
    
    echo [WARNING] Automatic download not implemented for Windows
    echo           Will build from source instead
) else (
    echo [WARNING] GitHub CLI (gh) not found, will build from source
)

echo.
echo ================================================================
echo   Step 4/5: Building game (if needed)...
echo ================================================================
echo.

cd /d "%PROJECT_ROOT%"

if not exist "%GAME_BINARY%" (
    if not exist "target\release\%GAME_BINARY%" (
        echo Building game from source...
        
        where cargo >nul 2>nul
        if %errorlevel% equ 0 (
            cargo build --release
            
            if exist "target\release\%GAME_BINARY%" (
                copy "target\release\%GAME_BINARY%" "%PROJECT_ROOT%\"
                echo [OK] Game built successfully
            ) else (
                echo [ERROR] Build failed
                exit /b 1
            )
        ) else (
            echo [ERROR] Cargo not found. Please install Rust toolchain.
            exit /b 1
        )
    ) else (
        echo [OK] Game binary already exists
    )
) else (
    echo [OK] Game binary already exists
)

echo.
echo ================================================================
echo   Step 5/5: Launching game...
echo ================================================================
echo.

cd /d "%PROJECT_ROOT%"

if exist "%GAME_BINARY%" (
    echo [OK] Starting SAS (Shoot and Strafe)...
    echo.
    start "" "%GAME_BINARY%"
) else if exist "target\release\%GAME_BINARY%" (
    echo [OK] Starting SAS (Shoot and Strafe)...
    echo.
    cargo run --release
) else (
    echo [ERROR] Game binary not found
    exit /b 1
)



