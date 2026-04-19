#!/usr/bin/env python3
"""
Extract a single Regular-S glyph from an SF Symbols 7 template SVG.

Apple ships SF Symbols as template SVGs containing 3 weights (Ultralight,
Regular, Black) * 3 scales (Small, Medium, Large) in a 3300x2200 layout,
wrapped in annotation art. This script pulls the Regular-S glyph out and
emits a standalone monochrome SVG sized to the glyph's own margins so it
renders correctly via GPUI's SVG path with text_color tinting.

Usage:
    ./extract_sf_symbol.py <sf-sym-7-dir> <symbol-name> <output-path>

Example:
    ./extract_sf_symbol.py /Users/me/Dev/sf-sym-7 magnifyingglass \
        ../assets/icons/sf/magnifyingglass.svg
"""

import re
import sys
import pathlib
import xml.etree.ElementTree as ET


NS = "http://www.w3.org/2000/svg"
ET.register_namespace("", NS)

# The template always places Regular-S at template y=696 (baseline). The
# glyph's margin lines span y=600.785..720.121 in template coords, so
# after we strip the transform the glyph's local y ranges roughly from
# -95 (top) to +24 (below baseline). Pad a few units on each side for
# antialiasing.
VIEWBOX_Y = -100.0
VIEWBOX_HEIGHT = 128.0
VIEWBOX_X_PAD = 4.0


def extract(template_path: pathlib.Path) -> str:
    tree = ET.parse(template_path)
    root = tree.getroot()

    # Find the Regular-S group and its transform + path.
    group = None
    left_margin = None
    right_margin = None
    for el in root.iter():
        tag = el.tag.split("}", 1)[-1]
        elid = el.get("id", "")
        if tag == "g" and elid == "Regular-S":
            group = el
        elif tag == "line" and elid == "left-margin-Regular-S":
            left_margin = float(el.get("x1"))
        elif tag == "line" and elid == "right-margin-Regular-S":
            right_margin = float(el.get("x1"))

    if group is None:
        raise SystemExit(f"Regular-S group not found in {template_path.name}")
    if left_margin is None or right_margin is None:
        raise SystemExit(
            f"Regular-S margin lines not found in {template_path.name} — "
            "template may be from a newer version with different ids."
        )

    transform = group.get("transform", "")
    m = re.match(
        r"matrix\(\s*1\s+0\s+0\s+1\s+([-\d.]+)\s+([-\d.]+)\s*\)",
        transform,
    )
    if not m:
        raise SystemExit(
            f"Expected identity matrix transform on Regular-S in "
            f"{template_path.name}; got {transform!r}"
        )
    tx = float(m.group(1))

    path = None
    for child in group:
        if child.tag.split("}", 1)[-1] == "path":
            path = child
            break
    if path is None:
        raise SystemExit(f"No <path> inside Regular-S group in {template_path.name}")

    d = path.get("d", "")
    if not d:
        raise SystemExit(f"Empty path d in {template_path.name}")

    # Template has TX == LX for every symbol seen so far — use that as the
    # local x origin. RX - LX is the glyph's ideal horizontal advance.
    width = right_margin - left_margin
    if abs(tx - left_margin) > 0.01:
        # Very rare but possible — fall back to the transform origin for
        # the glyph and extend the viewBox to cover whichever is further.
        width = max(width, right_margin - tx)

    vb_x = -VIEWBOX_X_PAD
    vb_w = width + 2 * VIEWBOX_X_PAD

    return (
        '<?xml version="1.0" encoding="UTF-8"?>\n'
        f'<svg xmlns="{NS}" viewBox="{vb_x:.3f} {VIEWBOX_Y} {vb_w:.3f} {VIEWBOX_HEIGHT}" '
        'fill="currentColor">\n'
        f'  <path fill="currentColor" d="{d}"/>\n'
        '</svg>\n'
    )


def main() -> None:
    if len(sys.argv) != 4:
        print(__doc__, file=sys.stderr)
        raise SystemExit(2)
    sf_dir = pathlib.Path(sys.argv[1])
    name = sys.argv[2]
    out = pathlib.Path(sys.argv[3])
    template = sf_dir / f"{name}.svg"
    if not template.exists():
        raise SystemExit(f"{template} does not exist")
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(extract(template))
    print(f"{name} -> {out}")


if __name__ == "__main__":
    main()
