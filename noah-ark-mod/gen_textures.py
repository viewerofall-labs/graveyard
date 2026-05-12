from PIL import Image

def create_checkerboard(width, height, pixel_size):
    """Generates a magenta and black checkerboard image."""
    img = Image.new("RGB", (width, height))
    pixels = img.load()

    for y in range(height):
        for x in range(width):
            # Determine color based on the grid position
            if (x // pixel_size) % 2 == (y // pixel_size) % 2:
                pixels[x, y] = (255, 0, 255) # Magenta
            else:
                pixels[x, y] = (0, 0, 0)       # Black
    return img

# 1. 16x16 Texture (Standard Source)
# Using a 8px tile size for a clean 2x2 grid
tex_16 = create_checkerboard(16, 16, 8)
tex_16.save("missing_16x16.png")

# 2. 32x16 Texture
# Keeping 8px tiles creates a 4x2 grid
tex_32 = create_checkerboard(32, 16, 8)
tex_32.save("missing_32x16.png")

# 3. Minecraft Skin (64x64)
# Modern MC skins are 64x64. 4px tiles make it look "glitchy" but readable.
mc_skin = create_checkerboard(64, 64, 4)
mc_skin.save("missing_skin_64x64.png")

print("Files generated: missing_16x16.png, missing_32x16.png, missing_skin_64x64.png")
