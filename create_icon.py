#!/usr/bin/env python3
"""
Create a simple icon for WinCleaner using Python
"""
import os
from PIL import Image, ImageDraw, ImageFont
import numpy as np

def create_wincleaner_icon():
    """Create a simple cleaning-themed icon"""
    # Create a 256x256 image with transparent background
    size = 256
    image = Image.new('RGBA', (size, size), (0, 0, 0, 0))
    draw = ImageDraw.Draw(image)
    
    # Draw a circular background with gradient
    center = size // 2
    radius = size // 2 - 8
    
    # Create gradient effect
    for r in range(radius, 0, -1):
        # Gradient from light blue to darker blue
        ratio = r / radius
        color = (
            int(100 + 50 * ratio),  # R
            int(150 + 50 * ratio),  # G  
            int(200 + 55 * ratio),  # B
            255  # A
        )
        draw.ellipse([
            center - r, center - r, center + r, center + r
        ], fill=color)
    
    # Draw a simple broom icon (just lines to represent cleaning)
    broom_color = (255, 255, 255, 255)  # White
    
    # Broom handle
    handle_width = 8
    handle_height = 80
    handle_x = center - handle_width // 2
    handle_y = center - handle_height // 2 - 20
    draw.rectangle([
        handle_x, handle_y,
        handle_x + handle_width, handle_y + handle_height
    ], fill=broom_color)
    
    # Broom bristles (simplified)
    bristle_start = handle_y + handle_height
    bristle_end = bristle_start + 30
    bristle_width = 40
    
    # Draw bristles as lines
    for i in range(-15, 16, 5):
        draw.line([
            center + i, bristle_start,
            center + i + np.random.randint(-3, 3), bristle_end
        ], fill=broom_color, width=3)
    
    # Add some sparkle effects (stars)
    sparkle_color = (255, 255, 200, 200)  # Light yellow
    sparkles = [
        (center - 60, center - 40),
        (center + 50, center - 30),
        (center - 40, center + 50),
        (center + 60, center + 40)
    ]
    
    for x, y in sparkles:
        # Draw small star-like shapes
        star_size = 8
        draw.polygon([
            (x, y - star_size),
            (x + star_size//2, y - star_size//2),
            (x + star_size, y),
            (x + star_size//2, y + star_size//2),
            (x, y + star_size),
            (x - star_size//2, y + star_size//2),
            (x - star_size, y),
            (x - star_size//2, y - star_size//2)
        ], fill=sparkle_color)
    
    # Save the icon
    assets_dir = "assets"
    if not os.path.exists(assets_dir):
        os.makedirs(assets_dir)
    
    icon_path = os.path.join(assets_dir, "wincleaner_icon.png")
    image.save(icon_path)
    print(f"Icon created: {icon_path}")
    
    # Also create a smaller version for the window (32x32)
    small_icon = image.resize((32, 32), Image.Resampling.LANCZOS)
    small_icon_path = os.path.join(assets_dir, "wincleaner_icon_small.png")
    small_icon.save(small_icon_path)
    print(f"Small icon created: {small_icon_path}")
    
    return icon_path

if __name__ == "__main__":
    try:
        create_wincleaner_icon()
        print("Icon creation completed successfully!")
    except ImportError as e:
        print(f"Error: {e}")
        print("Please install required packages:")
        print("pip install pillow numpy")
    except Exception as e:
        print(f"Error creating icon: {e}")