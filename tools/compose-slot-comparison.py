#!/usr/bin/env python3
"""Compare native baseline/current captures at identical scale, without warping."""
import argparse
from pathlib import Path
from PIL import Image, ImageDraw, ImageFont

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument("baseline", type=Path)
parser.add_argument("revised", type=Path)
parser.add_argument("output", type=Path)
args = parser.parse_args()
image = Image.new("RGB", (1520, 650), "#f5f5f2")
draw = ImageDraw.Draw(image)


def font(size):
    return ImageFont.truetype("DejaVuSans.ttf", size)


draw.text((48, 30), "Cartridge housing · photo comparison", fill="#202930", font=font(36))
draw.text((48, 86), "Same cartridge size, seating depth and opening. Revised housing follows the supplied SP photos.",
          fill="#53616b", font=font(21))
for x, title, detail in [(48, "Before · main", "476px housing / 308px opening"),
                         (784, "Revised", "400px housing / 308px opening")]:
    draw.text((x, 147), title, fill="#202930", font=font(28))
    draw.text((x, 187), detail, fill="#53616b", font=font(20))
for y, pose, label in [(245, "empty", "Empty"), (427, "seated", "GBA seated")]:
    draw.text((48, y - 30), label, fill="#53616b", font=font(18))
    for x, folder in [(48, args.baseline), (784, args.revised)]:
        source = Image.open(folder / f"platinum-gba-{pose}.png").convert("RGB")
        if source.size != (640, 480):
            parser.error("comparison requires matching 640×480 native captures")
        crop = source.crop((40, 385, 600, 480))
        # Apply one identical isotropic display scale to all four native crops.
        crop = crop.resize((688, 117), Image.Resampling.LANCZOS)
        image.paste(crop, (x, y))
draw.text((48, 592), "Native GLES captures · both views use the production renderer · no cartridge motion changes",
          fill="#53616b", font=font(19))
args.output.parent.mkdir(parents=True, exist_ok=True)
image.save(args.output)
print(args.output)
