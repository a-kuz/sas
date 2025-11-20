@echo off

echo Building release executable with resources...

cargo build --release --bin sas

set RELEASE_DIR=release_package
if exist "%RELEASE_DIR%" rmdir /s /q "%RELEASE_DIR%"
mkdir "%RELEASE_DIR%"

copy target\release\sas.exe "%RELEASE_DIR%\"

xcopy /E /I /Y assets "%RELEASE_DIR%\assets"
xcopy /E /I /Y maps "%RELEASE_DIR%\maps"
xcopy /E /I /Y q3-resources "%RELEASE_DIR%\q3-resources"

echo Package created in %RELEASE_DIR%\
dir "%RELEASE_DIR%"




