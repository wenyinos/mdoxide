#!/usr/bin/env python3
"""
Generate all icon sizes from a source PNG.
Usage: python generate_all_icons.py source.png

Requires: pip install Pillow
"""

import sys
from pathlib import Path
from PIL import Image

def make_square(img):
    """Center image on a square transparent canvas."""
    width, height = img.size
    size = max(width, height)
    
    # Create square transparent canvas
    square = Image.new('RGBA', (size, size), (0, 0, 0, 0))
    
    # Calculate position to center the image
    x = (size - width) // 2
    y = (size - height) // 2
    
    # Paste original onto center
    square.paste(img, (x, y), img if img.mode == 'RGBA' else None)
    return square

def generate_icons(source_path: str):
    source = Path(source_path)
    if not source.exists():
        print(f"Error: {source} not found")
        sys.exit(1)
    
    # Load source image
    img = Image.open(source)
    print(f"Loaded: {source} ({img.size[0]}x{img.size[1]})")
    
    # Ensure RGBA
    if img.mode != 'RGBA':
        img = img.convert('RGBA')
    
    # Make square if not already
    if img.size[0] != img.size[1]:
        print(f"Image is not square, centering on {max(img.size)}x{max(img.size)} canvas...")
        img = make_square(img)
        print(f"New size: {img.size[0]}x{img.size[1]}")
    
    output_dir = source.parent / "generated"
    output_dir.mkdir(exist_ok=True)
    
    # === PNG sizes for all platforms ===
    png_sizes = [16, 32, 48, 64, 128, 192, 256, 512, 1024]
    
    print("\n[PNG] Generating PNG sizes...")
    for size in png_sizes:
        resized = img.resize((size, size), Image.Resampling.LANCZOS)
        output_path = output_dir / f"icon-{size}x{size}.png"
        resized.save(output_path, "PNG")
        print(f"  OK {output_path.name}")
    
    # === Windows .ico (multi-size) ===
    print("\n[ICO] Generating Windows .ico...")
    ico_sizes = [(16, 16), (32, 32), (48, 48), (256, 256)]
    ico_images = [img.resize(size, Image.Resampling.LANCZOS) for size in ico_sizes]
    ico_path = output_dir / "app.ico"
    ico_images[0].save(
        ico_path,
        format="ICO",
        sizes=ico_sizes,
        append_images=ico_images[1:]
    )
    print(f"  OK {ico_path.name} (contains {len(ico_sizes)} sizes)")
    
    # === macOS .icns (if on macOS with iconutil) ===
    print("\n[ICNS] Generating macOS iconset folder...")
    iconset_dir = output_dir / "app.iconset"
    iconset_dir.mkdir(exist_ok=True)
    
    # macOS icon sizes and their names
    macos_icons = [
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
    
    for size, filename in macos_icons:
        resized = img.resize((size, size), Image.Resampling.LANCZOS)
        resized.save(iconset_dir / filename, "PNG")
        print(f"  OK {filename}")
    
    print(f"\n  TIP: To create .icns on macOS, run:")
    print(f"       iconutil -c icns {iconset_dir}")
    
    # === Linux desktop icons ===
    print("\n[LINUX] Generating Linux icons...")
    linux_dir = output_dir / "linux"
    linux_dir.mkdir(exist_ok=True)
    
    linux_sizes = [16, 32, 48, 64, 128, 256, 512]
    for size in linux_sizes:
        resized = img.resize((size, size), Image.Resampling.LANCZOS)
        size_dir = linux_dir / f"{size}x{size}"
        size_dir.mkdir(exist_ok=True)
        resized.save(size_dir / "ferrite.png", "PNG")
        print(f"  OK {size}x{size}/ferrite.png")
    
    # === PortableApps.com icons ===
    print("\n[PAF] Generating PortableApps.com icons...")
    paf_dir = output_dir / "portableapps"
    paf_dir.mkdir(exist_ok=True)

    # PAF requires specific PNG sizes and a multi-format ICO
    for size, name in [(16, "appicon_16.png"), (32, "appicon_32.png"), (128, "appicon_128.png")]:
        resized = img.resize((size, size), Image.Resampling.LANCZOS)
        resized.save(paf_dir / name, "PNG")
        print(f"  OK {name}")

    # PAF ICO: 16/32/48 in both 32-bit RGBA and 256-color, plus optional 256px PNG
    paf_ico_sizes = [(16, 16), (32, 32), (48, 48), (256, 256)]
    paf_ico_images = [img.resize(s, Image.Resampling.LANCZOS) for s in paf_ico_sizes]
    paf_ico_path = paf_dir / "appicon.ico"
    paf_ico_images[0].save(
        paf_ico_path,
        format="ICO",
        sizes=paf_ico_sizes,
        append_images=paf_ico_images[1:]
    )
    print(f"  OK appicon.ico (contains {len(paf_ico_sizes)} sizes)")
    print(f"\n  TIP: Copy contents of {paf_dir} into")
    print(f"       portable/FerriteMDPortable/App/AppInfo/")

    # === Summary ===
    print(f"\n=== DONE ===")
    print(f"All icons saved to: {output_dir}")
    print(f"\nFiles created:")
    print(f"  - app.ico          -> Windows app icon")
    print(f"  - app.iconset/     -> macOS iconset (use iconutil on Mac)")
    print(f"  - linux/           -> Linux hicolor icons")
    print(f"  - icon-NxN.png     -> Generic PNG sizes")
    print(f"  - portableapps/    -> PortableApps.com appicon files")


if __name__ == "__main__":
    if len(sys.argv) < 2:
        print("Usage: python generate_all_icons.py <source.png>")
        print("Example: python generate_all_icons.py logo-1024.png")
        sys.exit(1)
    
    generate_icons(sys.argv[1])
