@echo off

echo Building standalone executable (self-contained)...

cargo build --release --bin sas

set STANDALONE_DIR=standalone_build
if exist "%STANDALONE_DIR%" rmdir /s /q "%STANDALONE_DIR%"
mkdir "%STANDALONE_DIR%"

copy target\release\sas.exe "%STANDALONE_DIR%\"

cd "%STANDALONE_DIR%"

mkdir assets\fonts
xcopy /E /I /Y ..\assets\fonts assets\fonts

mkdir maps
xcopy /E /I /Y ..\maps maps

mkdir q3-resources
xcopy /E /I /Y ..\q3-resources q3-resources

cd ..

echo Creating archive...
powershell -Command "Compress-Archive -Path '%STANDALONE_DIR%\*' -DestinationPath sas_release.zip -Force"
echo Created sas_release.zip

echo Standalone build complete!
echo Directory: %STANDALONE_DIR%\
dir "%STANDALONE_DIR%"




