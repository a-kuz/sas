#!/bin/bash

rustup target add wasm32-unknown-unknown

export PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:$PATH"

cargo build --release --target wasm32-unknown-unknown --bin sas

mkdir -p web

cp target/wasm32-unknown-unknown/release/sas.wasm web/

if [ ! -L web/q3-resources ]; then
    ln -sf ../q3-resources web/q3-resources
fi
if [ ! -L web/maps ]; then
    ln -sf ../maps web/maps
fi
if [ ! -L web/assets ]; then
    ln -sf ../assets web/assets
fi

cp index.html web/

echo "Build complete! WASM file is in web/"
echo "To run: ./run_server.sh"
echo "Then open http://localhost:8003 in your browser"

