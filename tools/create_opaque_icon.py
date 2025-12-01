import sys
import os

def create_opaque_icon_pil(input_path, output_path, bg_color=(0, 0, 0)):
    from PIL import Image
    print(f"Creating opaque icon from {input_path}...")
    img = Image.open(input_path)
    
    if img.mode != 'RGBA':
        img = img.convert('RGBA')
    
    background = Image.new('RGB', img.size, bg_color)
    background.paste(img, mask=img.split()[3])
    
    background.save(output_path, 'PNG')
    print(f"Saved opaque icon to {output_path}")

def create_opaque_icon_cv2(input_path, output_path, bg_color=(0, 0, 0)):
    import cv2
    import numpy as np
    print(f"Creating opaque icon from {input_path}...")
    img = cv2.imread(input_path, cv2.IMREAD_UNCHANGED)
    
    if img is None:
        print(f"Error: Could not read {input_path}")
        sys.exit(1)
    
    if img.shape[2] == 4:
        alpha = img[:, :, 3] / 255.0
        alpha = alpha[:, :, np.newaxis]
        
        bgr = img[:, :, :3]
        background = np.full_like(bgr, bg_color[::-1], dtype=np.uint8)
        
        result = (bgr * alpha + background * (1 - alpha)).astype(np.uint8)
    else:
        result = img
    
    cv2.imwrite(output_path, result)
    print(f"Saved opaque icon to {output_path}")

if __name__ == "__main__":
    input_file = "assets/logo.png"
    output_file = "assets/logo-opaque.png"
    
    if not os.path.exists(input_file):
        print(f"Error: Input file {input_file} does not exist.")
        sys.exit(1)
    
    try:
        create_opaque_icon_cv2(input_file, output_file)
    except ImportError:
        try:
            create_opaque_icon_pil(input_file, output_file)
        except ImportError:
            print("Error: Neither cv2 nor PIL is installed. Please install opencv-python or Pillow.")
            sys.exit(1)


