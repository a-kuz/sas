#!/bin/bash

PORT=${1:-27960}
MAX_PLAYERS=${2:-16}
MAP=${3:-0-arena}

echo "Starting dedicated server..."
echo "Port: $PORT"
echo "Max players: $MAX_PLAYERS"
echo "Map: $MAP"
echo ""
cargo run --bin dedicated_server -- $PORT $MAX_PLAYERS $MAP

