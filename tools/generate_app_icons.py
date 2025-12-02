import sys
import os
from PIL import Image
import subprocess

def create_ico(source_png, output_ico):
    print(f"Creating Windows icon from {source_png}...")
    img = Image.open(source_png)
    
    if img.mode != 'RGB':
        if img.mode == 'RGBA':
            background = Image.new('RGB', img.size, (0, 0, 0))
            background.paste(img, mask=img.split()[3] if len(img.split()) == 4 else None)
            img = background
        else:
            img = img.convert('RGB')
    
    sizes = [(16, 16), (32, 32), (48, 48), (64, 64), (128, 128), (256, 256)]
    icons = []
    for size in sizes:
        resized = img.resize(size, Image.Resampling.LANCZOS)
        icons.append(resized)
    
    icons[0].save(output_ico, format='ICO', sizes=[(s[0], s[1]) for s in sizes])
    print(f"Created {output_ico}")

def create_icns_macos(source_png, output_icns):
    print(f"Creating macOS icon from {source_png}...")
    
    iconset_dir = "assets/AppIcon.iconset"
    os.makedirs(iconset_dir, exist_ok=True)
    
    img = Image.open(source_png)
    if img.mode == 'RGBA':
        background = Image.new('RGB', img.size, (0, 0, 0))
        background.paste(img, mask=img.split()[3])
        img = background
    elif img.mode != 'RGB':
        img = img.convert('RGB')
    
    sizes = [
        (16, "icon_16x16.png"),
        (32, "icon_16x16@2x.png"),
        (32, "icon_32x32.png"),
        (64, "icon_32x32@2x.png"),
        (128, "icon_128x128.png"),
        (256, "icon_128x128@2x.png"),
        (256, "icon_256x256.png"),
        (512, "icon_256x256@2x.png"),
        (512, "icon_512x512.png"),
        (1024, "icon_512x512@2x.png"),
    ]
    
    for size, filename in sizes:
        resized = img.resize((size, size), Image.Resampling.LANCZOS)
        resized.save(os.path.join(iconset_dir, filename), 'PNG')
    
    try:
        subprocess.run(['iconutil', '-c', 'icns', iconset_dir, '-o', output_icns], check=True)
        print(f"Created {output_icns}")
    except subprocess.CalledProcessError as e:
        print(f"Error running iconutil: {e}")
        sys.exit(1)
    except FileNotFoundError:
        print("Error: iconutil not found. This tool only works on macOS.")
        sys.exit(1)
    
    import shutil
    shutil.rmtree(iconset_dir)
    print(f"Cleaned up {iconset_dir}")

if __name__ == "__main__":
    source_file = "assets/logo-opaque.png"
    ico_file = "assets/icon.ico"
    icns_file = "assets/icon.icns"
    
    if not os.path.exists(source_file):
        print(f"Error: Source file {source_file} does not exist.")
        print("Run create_opaque_icon.py first to create the opaque logo.")
        sys.exit(1)
    
    create_ico(source_file, ico_file)
    
    if sys.platform == 'darwin':
        create_icns_macos(source_file, icns_file)
    else:
        print("Skipping .icns creation (only available on macOS)")
    
    print("\nDone! Icon files created successfully.")



