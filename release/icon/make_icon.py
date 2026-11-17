#!/usr/bin/env python3
"""Draws the Naran icon and writes `icon-1024.png` and `Naran.icns` next to it.

Three upright bars side by side (the name means "side by side"), in the colors
of the commit graph lanes, on a dark rounded square. Needs Pillow and, for the
.icns file, macOS `iconutil`.
"""
import os
import subprocess
import sys
import tempfile

from PIL import Image, ImageDraw

HERE = os.path.dirname(os.path.abspath(__file__))
SIZE = 1024
SCALE = 4  # draw large, then shrink, for smooth edges


def rounded(draw, box, radius, fill):
    draw.rounded_rectangle(box, radius=radius, fill=fill)


def row_color(t, top, bottom):
    return tuple(round(top[i] + (bottom[i] - top[i]) * t) for i in range(3))


def gradient(size, top, bottom):
    image = Image.new("RGB", (size, size))
    colors = [row_color(y / (size - 1), top, bottom) for y in range(size)]
    image.putdata([color for color in colors for _ in range(size)])
    return image


def body_geometry():
    """The rounded square: its margin, its box and its corner radius, at drawing size."""
    big = SIZE * SCALE
    margin = 100 * SCALE  # macOS icons leave a margin around the square
    box = (margin, margin, big - margin, big - margin)
    return margin, box, int((big - 2 * margin) * 0.225)


def mask(box, radius):
    image = Image.new("L", (SIZE * SCALE, SIZE * SCALE), 0)
    rounded(ImageDraw.Draw(image), box, radius, 255)
    return image


def tile(box, radius):
    """The dark rounded square with its gradient."""
    big = SIZE * SCALE
    canvas = Image.new("RGBA", (big, big), (0, 0, 0, 0))
    background = gradient(big, (0x24, 0x28, 0x33), (0x0F, 0x11, 0x15)).convert("RGBA")
    canvas.paste(background, (0, 0), mask(box, radius))
    return canvas


def draw_rim(draw, box, radius):
    rounded(draw, box, radius, (255, 255, 255, 26))
    inner = tuple(v + 3 * SCALE if i < 2 else v - 3 * SCALE for i, v in enumerate(box))
    rounded(draw, inner, radius - 3 * SCALE, (0, 0, 0, 0))


def rim(canvas, box, radius):
    """A thin light edge at the top, like the rim of a glass tile."""
    edge = Image.new("RGBA", (SIZE * SCALE, SIZE * SCALE), (0, 0, 0, 0))
    draw_rim(ImageDraw.Draw(edge), box, radius)
    return Image.alpha_composite(canvas, edge)


BAR_COLORS = [(0x7A, 0xA2, 0xF7), (0x9E, 0xCE, 0x6A), (0xE0, 0xAF, 0x68)]


def bar_sizes(margin):
    inner = SIZE * SCALE - 2 * margin
    return int(inner * 0.17), int(inner * 0.085), int(inner * 0.52)


def bar_boxes(margin):
    """Three bars of the same size, evenly spaced: the box of each."""
    width, gap, height = bar_sizes(margin)
    left = (SIZE * SCALE - (3 * width + 2 * gap)) // 2
    top = (SIZE * SCALE - height) // 2
    xs = [left + index * (width + gap) for index in range(3)]
    return [(x, top, x + width, top + height) for x in xs]


def bars(canvas, margin):
    draw = ImageDraw.Draw(canvas)
    for box, color in zip(bar_boxes(margin), BAR_COLORS):
        rounded(draw, box, (box[2] - box[0]) // 2, color + (255,))
    return canvas


def draw_icon():
    margin, box, radius = body_geometry()
    canvas = bars(rim(tile(box, radius), box, radius), margin)
    return canvas.resize((SIZE, SIZE), Image.LANCZOS)


def write_iconset(icon, iconset):
    """The sizes macOS asks for, each at 1x and 2x."""
    os.makedirs(iconset)
    for size in (16, 32, 128, 256, 512):
        icon.resize((size, size), Image.LANCZOS).save(os.path.join(iconset, f"icon_{size}x{size}.png"))
        icon.resize((size * 2, size * 2), Image.LANCZOS).save(os.path.join(iconset, f"icon_{size}x{size}@2x.png"))


def write_icns(icon, icns):
    with tempfile.TemporaryDirectory() as directory:
        iconset = os.path.join(directory, "Naran.iconset")
        write_iconset(icon, iconset)
        subprocess.run(["iconutil", "-c", "icns", iconset, "-o", icns], check=True)


def main():
    icon = draw_icon()
    png, icns = (os.path.join(HERE, name) for name in ("icon-1024.png", "Naran.icns"))
    icon.save(png)
    write_icns(icon, icns)
    print("wrote", png, "and", icns)


if __name__ == "__main__":
    sys.exit(main())
