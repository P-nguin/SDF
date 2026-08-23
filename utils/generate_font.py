import os
from PIL import Image, ImageDraw, ImageFont

def generate_font():
    # Ensure current directory is the root of the project
    # when running this script
    script_dir = os.path.dirname(os.path.abspath(__file__))
    root_dir = os.path.dirname(script_dir)
    texture_dir = os.path.join(root_dir, 'assets', 'textures')
    
    os.makedirs(texture_dir, exist_ok=True)
    font_path = os.path.join(root_dir, 'assets', 'fonts', 'CommitMono-VariableFont.ttf')
    output_path = os.path.join(texture_dir, 'font.png')
    
    # Increased to 128x128 square cells to allow for massive scaling later
    WIDTH, HEIGHT = 128, 128 
    
    # 12x12 tightly packs 144 characters (which covers standard ASCII up to 127)
    # in an optimal square, avoiding excessive empty atlas padding limits.
    img = Image.new('L', (WIDTH * 12, HEIGHT * 12), color=0)
    draw = ImageDraw.Draw(img)
    
    try:
        # Heavily scaled font to fill the 128x128 square block
        font = ImageFont.truetype(font_path, 110)
    except Exception as e:
        print(f"Error loading TTF: {e}")
        print(f"Make sure {font_path} exists.")
        exit(1)
    # We only need up to char 143 to fit inside 12x12
    for i in range(144):
        col, row = i % 12, i // 12
        x, y = col * WIDTH, row * HEIGHT
        
        # Determine the character
        char = chr(i) if 31 < i < 127 else ' '
        
        # Explicit overrides for the specific characters the shader expects
        if i == 124: char = '|'
        elif i == 45: char = '-'
        elif i == 35: char = '#'
        elif i == 87: char = 'W'
        
        # Calculate bounding box to perfectly center the character
        bbox = font.getbbox(char)
        char_w = bbox[2] - bbox[0]
        char_h = bbox[3] - bbox[1]
        
        cx = x + (WIDTH - char_w) / 2
        cy = y + (HEIGHT - char_h) / 2 - bbox[1]
        
        draw.text((cx, cy), char, fill=255, font=font)

    img.save(output_path)
    print(f"Rasterized 16x16 ASCII grid successfully at {output_path}")

if __name__ == '__main__':
    generate_font()
