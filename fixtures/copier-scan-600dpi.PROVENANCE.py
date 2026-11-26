"""Author `copier-scan-600dpi.pdf`: one legal page holding one 600 dpi colour
JPEG, 5100 x 8400 samples (42.84 Mpx), white with a solid blue block.

    python fixtures/copier-scan-600dpi.PROVENANCE.py

Deterministic: no timestamps, no producer string. Needs Pillow.

The shape an office copier writes by default: a 600 dpi `/DCTDecode`
`/DeviceRGB` image filling an 8.5 x 14 in page. It is past the old 32 Mpx
`MAX_IMAGE_PIXELS` and past the 2^24 px a page may send to the graphics card,
so it renders on the processor. Driven by
`tools/ui-verify/src/checks/copier_scan.rs`.
"""

import io
from pathlib import Path

from PIL import Image, ImageDraw

W, H = 5100, 8400
PAGE_W, PAGE_H = 612, 1008
# The block in page points (x0, y0, x1, y1), origin bottom-left.
BLOCK = (150, 300, 462, 708)
BLUE = (20, 60, 200)


def page_image() -> bytes:
    im = Image.new("RGB", (W, H), (255, 255, 255))
    sx, sy = W / PAGE_W, H / PAGE_H
    x0, y0, x1, y1 = BLOCK
    # Image rows run top-down, page y bottom-up.
    box = (round(x0 * sx), round((PAGE_H - y1) * sy), round(x1 * sx), round((PAGE_H - y0) * sy))
    ImageDraw.Draw(im).rectangle(box, fill=BLUE)
    out = io.BytesIO()
    im.save(out, format="JPEG", quality=60, optimize=True)
    return out.getvalue()


def pdf(jpeg: bytes) -> bytes:
    content = f"q {PAGE_W} 0 0 {PAGE_H} 0 0 cm /Im0 Do Q".encode()
    objs = [
        b"<< /Type /Catalog /Pages 2 0 R >>",
        b"<< /Type /Pages /Kids [3 0 R] /Count 1 >>",
        f"<< /Type /Page /Parent 2 0 R /MediaBox [0 0 {PAGE_W} {PAGE_H}] "
        f"/Resources << /XObject << /Im0 4 0 R >> >> /Contents 5 0 R >>".encode(),
        f"<< /Type /XObject /Subtype /Image /Width {W} /Height {H} /ColorSpace /DeviceRGB "
        f"/BitsPerComponent 8 /Filter /DCTDecode /Length {len(jpeg)} >>\nstream\n".encode()
        + jpeg
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
    target = Path(__file__).with_name("copier-scan-600dpi.pdf")
    target.write_bytes(pdf(page_image()))
    print(f"wrote {target}")
