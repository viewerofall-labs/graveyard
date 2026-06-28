#!/usr/bin/env python3
from PIL import Image, ImageDraw

size = 256
bg_color = (10, 0, 16)  # #0a0010
accent1 = (199, 146, 234)  # #c792ea
accent2 = (0, 229, 200)  # #00e5c8

img = Image.new('RGBA', (size, size), bg_color + (255,))
draw = ImageDraw.Draw(img)

# Left box (removed - red tint with accent1)
left_x = 30
left_y = 50
box_w = 80
box_h = 140
draw.rectangle([left_x, left_y, left_x + box_w, left_y + box_h],
               outline=accent1, width=3)
draw.line([(left_x + 10, left_y + 30), (left_x + box_w - 10, left_y + 30)],
          fill=accent1, width=2)
draw.line([(left_x + 10, left_y + 60), (left_x + box_w - 10, left_y + 60)],
          fill=(220, 100, 100), width=2)
draw.line([(left_x + 10, left_y + 90), (left_x + box_w - 10, left_y + 90)],
          fill=(220, 100, 100), width=2)

# Right box (added - cyan tint with accent2)
right_x = 146
right_y = 50
draw.rectangle([right_x, right_y, right_x + box_w, right_y + box_h],
               outline=accent2, width=3)
draw.line([(right_x + 10, right_y + 30), (right_x + box_w - 10, right_y + 30)],
          fill=accent2, width=2)
draw.line([(right_x + 10, right_y + 60), (right_x + box_w - 10, right_y + 60)],
          fill=(100, 220, 200), width=2)
draw.line([(right_x + 10, right_y + 90), (right_x + box_w - 10, right_y + 90)],
          fill=(100, 220, 200), width=2)

# Arrow in middle
arrow_x = size // 2
arrow_y = size // 2 + 20
draw.line([(arrow_x - 15, arrow_y), (arrow_x + 15, arrow_y)], fill=accent1, width=2)
draw.line([(arrow_x + 10, arrow_y - 5), (arrow_x + 15, arrow_y)], fill=accent1, width=2)
draw.line([(arrow_x + 10, arrow_y + 5), (arrow_x + 15, arrow_y)], fill=accent1, width=2)

img.save('/home/abyss/.local/share/icons/hicolor/256x256/apps/filediff.png')
print("Icon created at ~/.local/share/icons/hicolor/256x256/apps/filediff.png")
