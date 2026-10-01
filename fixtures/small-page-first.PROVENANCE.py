# -*- coding: utf-8 -*-
"""Build `fixtures/small-page-first.pdf`: a postcard-sized page 1 (200 x 280 pt)
above a US Letter page 2, each with a filled box so it is visibly drawn.

Used by `a_small_first_page_is_the_current_page`: at launch, continuous mode
shows page 1 whole and page 2 covering more of the view; page 1 must be the
page that page commands act on.

Run: python fixtures/small-page-first.PROVENANCE.py
"""
from pathlib import Path

PAGES = [(200, 280, b"0.8 0.2 0.2 rg 20 20 160 240 re f\n"), (612, 792, b"0.2 0.2 0.8 rg 72 72 468 648 re f\n")]

objects = [b"<< /Type /Catalog /Pages 2 0 R >>", b"<< /Type /Pages /Kids [3 0 R 5 0 R] /Count 2 >>"]
for n, (w, h, content) in enumerate(PAGES):
    contents = 4 + 2 * n
    objects.append(
        f"<< /Type /Page /Parent 2 0 R /MediaBox [0 0 {w} {h}] /Contents {contents} 0 R /Resources << >> >>".encode()
    )
    objects.append(b"<< /Length " + str(len(content)).encode() + b" >>\nstream\n" + content + b"endstream")
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
Path(__file__).with_name("small-page-first.pdf").write_bytes(bytes(out))
