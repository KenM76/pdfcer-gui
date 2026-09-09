#!/usr/bin/env python3
"""Write `fixtures/rotated-90.pdf` — one landscape sheet stored PORTRAIT with
`/Rotate 90`, the shape of every page of the operator's Ghostscript-produced
drawing set (2026-09-09: *"the text comes out vertical"*).

Why a generator and not a copied file: the fixture must be tiny, uncompressed
and byte-predictable, so a test that authors a stamp on it and reads the
appearance `/Matrix` back is asserting about the ROTATION and nothing else.
The page draws one line of text so extraction has something to find.

Run from the repository root:  python tools/gen-rotated-page-fixture.py
"""
from pathlib import Path

content = b"BT /F1 18 Tf 72 700 Td (ROTATED SHEET) Tj ET"
objects = [
    b"<< /Type /Catalog /Pages 2 0 R >>",
    b"<< /Type /Pages /Kids [3 0 R] /Count 1 >>",
    b"<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] /Rotate 90 "
    b"/Resources << /Font << /F1 5 0 R >> >> /Contents 4 0 R >>",
    b"<< /Length " + str(len(content)).encode() + b" >>\nstream\n" + content + b"\nendstream",
    b"<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>",
]
out = bytearray(b"%PDF-1.4\n%\xe2\xe3\xcf\xd3\n")
offsets = []
for i, body in enumerate(objects, start=1):
    offsets.append(len(out))
    out += f"{i} 0 obj\n".encode() + body + b"\nendobj\n"
xref = len(out)
out += f"xref\n0 {len(objects) + 1}\n".encode()
out += b"0000000000 65535 f \n"
for off in offsets:
    out += f"{off:010d} 00000 n \n".encode()
out += f"trailer\n<< /Size {len(objects) + 1} /Root 1 0 R >>\nstartxref\n{xref}\n%%EOF\n".encode()
target = Path(__file__).resolve().parent.parent / "fixtures" / "rotated-90.pdf"
target.write_bytes(bytes(out))
print(f"wrote {target} ({len(out)} bytes)")
