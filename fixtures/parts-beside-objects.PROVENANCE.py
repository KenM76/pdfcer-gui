# -*- coding: utf-8 -*-
"""Build fixtures/parts-beside-objects.pdf.

One 400 x 300 page: a page object beside two separate placed drawings, so an
Align can be asked of a selection mixing whole objects with parts of placed
drawings, and of parts of two different drawings.

    page object 0  a green filled rectangle, (300,220)..(360,260)
    page object 1  form Fm0 placed at (40, 40), scale 1, holding
                   leaf 0  a red filled rectangle, page (60,60)..(120,100)
                   leaf 1  a blue filled rectangle, page (60,140)..(100,170)
    page object 2  form Fm1 (a separate XObject) placed at (200, 40), holding
                   leaf 2  an orange filled rectangle, page (220,60)..(270,110)

Run from the repository root: python fixtures/parts-beside-objects.PROVENANCE.py
"""
from pathlib import Path

PAGE = (
    b"0 1 0 rg 300 220 60 40 re f\n"
    b"q 1 0 0 1 40 40 cm /Fm0 Do Q\n"
    b"q 1 0 0 1 200 40 cm /Fm1 Do Q\n"
)
FM0 = b"1 0 0 rg 20 20 60 40 re f\n0 0 1 rg 20 100 40 30 re f\n"
FM1 = b"1 0.5 0 rg 20 20 50 50 re f\n"


def stream(head: bytes, data: bytes) -> bytes:
    return b"<< " + head + b" /Length %d >>\nstream\n" % len(data) + data + b"endstream"


def form(data: bytes) -> bytes:
    return stream(b"/Type /XObject /Subtype /Form /FormType 1 /BBox [0 0 160 160] /Resources << >>", data)


objects = [
    b"<< /Type /Catalog /Pages 2 0 R >>",
    b"<< /Type /Pages /Kids [3 0 R] /Count 1 >>",
    b"<< /Type /Page /Parent 2 0 R /MediaBox [0 0 400 300] "
    b"/Resources << /XObject << /Fm0 5 0 R /Fm1 6 0 R >> >> /Contents 4 0 R >>",
    stream(b"", PAGE),
    form(FM0),
    form(FM1),
]
out = bytearray(b"%PDF-1.7\n%\xe2\xe3\xcf\xd3\n")
offsets = []
for number, body in enumerate(objects, start=1):
    offsets.append(len(out))
    out += b"%d 0 obj\n" % number + body + b"\nendobj\n"
xref = len(out)
out += b"xref\n0 %d\n0000000000 65535 f \n" % (len(objects) + 1)
for offset in offsets:
    out += b"%010d 00000 n \n" % offset
out += b"trailer\n<< /Size %d /Root 1 0 R >>\nstartxref\n%d\n%%%%EOF\n" % (len(objects) + 1, xref)
target = Path("fixtures") / "parts-beside-objects.pdf"
target.write_bytes(bytes(out))
print(f"wrote {target} ({len(out)} bytes)")
