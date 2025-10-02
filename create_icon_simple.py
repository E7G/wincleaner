#!/usr/bin/env python3
"""
Simple icon creation for WinCleaner without external dependencies
Creates a basic PNG icon using only built-in libraries
"""
import struct
import os

def create_simple_png():
    """Create a simple 32x32 PNG icon with basic shapes"""
    
    # Simple 32x32 image data (RGBA)
    width, height = 32, 32
    
    # Create a simple pattern - blue circle with white center
    pixels = []
    center_x, center_y = width // 2, height // 2
    radius = 12
    
    for y in range(height):
        row = []
        for x in range(width):
            # Calculate distance from center
            dx = x - center_x
            dy = y - center_y
            distance = (dx**2 + dy**2)**0.5
            
            if distance <= radius:
                # Inside circle - gradient effect
                if distance <= radius * 0.3:
                    # Center - white
                    row.extend([255, 255, 255, 255])
                else:
                    # Outer part - blue gradient
                    intensity = 1 - (distance - radius * 0.3) / (radius * 0.7)
                    blue = int(150 + 105 * intensity)
                    green = int(100 + 50 * intensity)
                    red = int(50 + 50 * intensity)
                    row.extend([red, green, blue, 255])
            else:
                # Outside circle - transparent
                row.extend([0, 0, 0, 0])
        
        pixels.extend(row)
    
    # Convert to bytes
    pixel_data = bytes(pixels)
    
    # Create PNG file
    assets_dir = "assets"
    if not os.path.exists(assets_dir):
        os.makedirs(assets_dir)
    
    icon_path = os.path.join(assets_dir, "wincleaner_icon.png")
    
    # Simple PNG creation (IDAT chunk with basic compression)
    with open(icon_path, 'wb') as f:
        # PNG signature
        f.write(b'\x89PNG\r\n\x1a\n')
        
        # IHDR chunk
        ihdr_data = struct.pack('>IIBBBBB', width, height, 8, 6, 0, 0, 0)
        ihdr_crc = crc32(b'IHDR' + ihdr_data)
        f.write(struct.pack('>I', 13))
        f.write(b'IHDR')
        f.write(ihdr_data)
        f.write(struct.pack('>I', ihdr_crc))
        
        # IDAT chunk (simplified - no compression for demonstration)
        # This is a very basic implementation
        scanline_data = b''
        for y in range(height):
            scanline_data += b'\x00'  # Filter type 0 (None)
            scanline_data += pixel_data[y*width*4:(y+1)*width*4]
        
        # Simple "compression" - just store the data
        idat_data = scanline_data
        idat_crc = crc32(b'IDAT' + idat_data)
        f.write(struct.pack('>I', len(idat_data)))
        f.write(b'IDAT')
        f.write(idat_data)
        f.write(struct.pack('>I', idat_crc))
        
        # IEND chunk
        f.write(struct.pack('>I', 0))
        f.write(b'IEND')
        f.write(struct.pack('>I', crc32(b'IEND')))
    
    print(f"Simple icon created: {icon_path}")
    return icon_path

def crc32(data):
    """Simple CRC32 implementation for PNG"""
    crc = 0xFFFFFFFF
    for byte in data:
        crc ^= byte
        for _ in range(8):
            if crc & 1:
                crc = (crc >> 1) ^ 0xEDB88320
            else:
                crc >>= 1
    return crc ^ 0xFFFFFFFF

if __name__ == "__main__":
    try:
        create_simple_png()
        print("Simple icon creation completed!")
    except Exception as e:
        print(f"Error creating simple icon: {e}")
        print("Note: This is a basic implementation. For production use, install PIL:")
        print("pip install pillow")