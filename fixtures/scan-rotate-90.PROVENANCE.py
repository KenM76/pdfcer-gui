"""Author `scan-rotate-90.pdf`: an image-only page stored sideways with
`/Rotate 90`, so it displays upright and holds three lines of text to recognise.

    python fixtures/scan-rotate-90.PROVENANCE.py

Deterministic: no timestamps, no producer string. Needs Pillow and
`C:/Windows/Fonts/arial.ttf`.

Driven by `tools/ui-verify/src/checks/ocr_turned_page.rs`. A scanner that
stores a portrait sheet on its side and sets `/Rotate` is the common shape of
a straightened scan, and the shape on which the OCR layer was once written at
90 degrees to the words it covers. No other fixture has an image-only page
with a non-zero `/Rotate`.
"""

import zlib
from pathlib import Path

from PIL import Image, ImageDraw, ImageFont

LINES = ["The quick brown fox jumps", "over the lazy sleeping dog", "General notes for drawing"]
W, H = 1600, 500


def page_image() -> Image.Image:
    im = Image.new("L", (W, H), 255)
    draw = ImageDraw.Draw(im)
    font = ImageFont.truetype("C:/Windows/Fonts/arial.ttf", 60)
    for i, text in enumerate(LINES):
        draw.text((40, 60 + 140 * i), text, font=font, fill=0)
    # Stored a quarter-turn clockwise; /Rotate 90 turns it back for display.
    return im.rotate(90, expand=True)


def pdf(img: Image.Image, rotate: int) -> bytes:
    w, h = img.size
    data = zlib.compress(img.tobytes(), 9)
    pw, ph = w * 0.72, h * 0.72
    content = f"q {pw:g} 0 0 {ph:g} 0 0 cm /Im0 Do Q".encode()
    objs = [
        b"<< /Type /Catalog /Pages 2 0 R >>",
        b"<< /Type /Pages /Kids [3 0 R] /Count 1 >>",
        f"<< /Type /Page /Parent 2 0 R /MediaBox [0 0 {pw:g} {ph:g}] /Rotate {rotate} "
        f"/Resources << /XObject << /Im0 4 0 R >> >> /Contents 5 0 R >>".encode(),
        f"<< /Type /XObject /Subtype /Image /Width {w} /Height {h} /ColorSpace /DeviceGray "
        f"/BitsPerComponent 8 /Filter /FlateDecode /Length {len(data)} >>\nstream\n".encode()
        + data
        + b"\nendstream",
        f"<< /Length {len(content)} >>\nstream\n".encode() + content + b"\nendstream",
    ]
    out = b"%PDF-1.7\n"
    offsets = []
    for i, body in enumerate(objs):
        offsets.append(len(out))
        out += f"{i + 1} 0 obj\n".encode() + body + b"\nendobj\n"
    xref = len(out)
    out += f"xref\n0 {len(objs) + 1}\n0000000000 65535 f \n".encode()
    for off in offsets:
        out += f"{off:010d} 00000 n \n".encode()
    out += f"trailer\n<< /Size {len(objs) + 1} /Root 1 0 R >>\nstartxref\n{xref}\n%%EOF\n".encode()
    return out


if __name__ == "__main__":
    target = Path(__file__).with_name("scan-rotate-90.pdf")
    target.write_bytes(pdf(page_image(), 90))
    print(f"wrote {target}")
