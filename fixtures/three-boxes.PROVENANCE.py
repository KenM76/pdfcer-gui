# -*- coding: utf-8 -*-
"""Build `fixtures/three-boxes.pdf`: one Letter page holding three separate
filled rectangles, each its own path object, at staggered x and y.

Used by `align_left_moves_every_box_in_one_undo` (O263). The boxes, in PDF
user space (x0, y0, width, height), in paint order:

    0: (100, 600, 60, 40)
    1: (250, 500, 80, 50)
    2: (180, 350, 40, 70)

Aligning their left edges relative to the selection area moves boxes 1 and 2
to x0 = 100 and leaves box 0 where it is.

Run: python fixtures/three-boxes.PROVENANCE.py
"""
from pathlib import Path

BOXES = [(100, 600, 60, 40, "0.8 0.2 0.2"), (250, 500, 80, 50, "0.2 0.6 0.2"), (180, 350, 40, 70, "0.2 0.2 0.8")]
content = "".join(f"{c} rg {x} {y} {w} {h} re f\n" for x, y, w, h, c in BOXES).encode()

objects = [
    b"<< /Type /Catalog /Pages 2 0 R >>",
    b"<< /Type /Pages /Kids [3 0 R] /Count 1 >>",
    b"<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] /Contents 4 0 R /Resources << >> >>",
    b"<< /Length " + str(len(content)).encode() + b" >>\nstream\n" + content + b"endstream",
]
out = bytearray(b"%PDF-1.7\n")
offsets = []
for n, body in enumerate(objects, 1):
    offsets.append(len(out))
    out += f"{n} 0 obj\n".encode() + body + b"\nendobj\n"
xref = len(out)
out += f"xref\n0 {len(objects) + 1}\n0000000000 65535 f \n".encode()
for off in offsets:
    out += f"{off:010d} 00000 n \n".encode()
out += f"trailer\n<< /Size {len(objects) + 1} /Root 1 0 R >>\nstartxref\n{xref}\n%%EOF\n".encode()
Path(__file__).with_name("three-boxes.pdf").write_bytes(bytes(out))
