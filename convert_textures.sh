#!/bin/bash

echo "Converting all TGA files to PNG in q3-resources..."

count=0

find q3-resources -name "*.tga" -type f | while read tga_file; do
    png_file="${tga_file%.tga}.png"
    
    if [ ! -f "$png_file" ]; then
        if command -v sips &> /dev/null; then
            sips -s format png "$tga_file" --out "$png_file" > /dev/null 2>&1
            if [ $? -eq 0 ]; then
                echo "✓ Converted: $tga_file -> $png_file"
                ((count++))
            fi
        elif command -v convert &> /dev/null; then
            convert "$tga_file" "$png_file"
            if [ $? -eq 0 ]; then
                echo "✓ Converted: $tga_file -> $png_file"
                ((count++))
            fi
        else
            echo "Error: Neither 'sips' nor 'convert' (ImageMagick) found"
            exit 1
        fi
    fi
done

echo ""
echo "Done! Converted $count TGA files to PNG."

