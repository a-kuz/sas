#!/bin/bash

cd web
echo "Starting server at http://localhost:8003"
echo "Press Ctrl+C to stop"
python3 -m http.server 8003

