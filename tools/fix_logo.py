import sys
import os

def remove_black_background_cv2(input_path, output_path):
    import cv2
    import numpy as np
    print(f"Processing {input_path} with OpenCV (Smooth)...")
    img = cv2.imread(input_path)
    if img is None:
        print(f"Error: Could not read {input_path}")
        sys.exit(1)

    # Convert to float for processing
    img_float = img.astype(float) / 255.0
    
    # Calculate luminance (standard weights)
    luminance = 0.114 * img_float[:,:,0] + 0.587 * img_float[:,:,1] + 0.299 * img_float[:,:,2]
    
    # Create alpha channel based on luminance
    # Smoothstep-like curve to make darks transparent and brights opaque
    # Adjust these values to tune the look
    low = 0.05
    high = 0.8
    alpha = np.clip((luminance - low) / (high - low), 0.0, 1.0)
    
    # Square alpha for smoother falloff? Or just use as is.
    # alpha = alpha * alpha 
    
    alpha = (alpha * 255).astype(np.uint8)
    
    img_rgba = cv2.cvtColor(img, cv2.COLOR_BGR2BGRA)
    img_rgba[:, :, 3] = alpha

    cv2.imwrite(output_path, img_rgba)
    print(f"Saved smooth transparent image to {output_path}")

def remove_black_background_pil(input_path, output_path):
    from PIL import Image
    print(f"Processing {input_path} with PIL (Smooth)...")
    img = Image.open(input_path)
    img = img.convert("RGBA")
    datas = img.getdata()

    newData = []
    for item in datas:
        # Calculate luminance
        r, g, b, a = item
        lum = 0.299 * r + 0.587 * g + 0.114 * b
        
        # Smooth alpha
        low = 10.0
        high = 200.0
        
        if lum < low:
            new_a = 0
        elif lum > high:
            new_a = 255
        else:
            new_a = int(255 * (lum - low) / (high - low))
            
        newData.append((r, g, b, new_a))

    img.putdata(newData)
    img.save(output_path, "PNG")
    print(f"Saved smooth transparent image to {output_path}")

if __name__ == "__main__":
    input_file = "assets/logo.png"
    output_file = "assets/logo-alfa.png"
    
    if not os.path.exists(input_file):
        print(f"Error: Input file {input_file} does not exist.")
        sys.exit(1)

    try:
        remove_black_background_cv2(input_file, output_file)
    except ImportError:
        try:
            remove_black_background_pil(input_file, output_file)
        except ImportError:
            print("Error: Neither cv2 nor PIL is installed. Please install opencv-python or Pillow.")
            sys.exit(1)
