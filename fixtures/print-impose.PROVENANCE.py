# -*- coding: utf-8 -*-
"""Build `fixtures/print-impose.pdf`: five Letter pages, each naming its
number in large type, for the Print dialog's Multiple and Booklet layouts.

Five is chosen because it is not a multiple of four: a booklet of it needs
three blank pages and two folded sheets (four sides), and two pages per sheet
need three sheets.

Run: python fixtures/print-impose.PROVENANCE.py
"""
import pathlib

PAGES = 5
objects = [
    b"<< /Type /Catalog /Pages 2 0 R >>",
    b"<< /Type /Pages /Kids [%s] /Count %d >>"
    % (b" ".join(b"%d 0 R" % (4 + 2 * i) for i in range(PAGES)), PAGES),
    b"<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>",
]
for i in range(PAGES):
    text = b"BT /F1 144 Tf 250 350 Td (%d) Tj ET" % (i + 1)
    objects.append(
        b"<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] /Contents %d 0 R "
        b"/Resources << /Font << /F1 3 0 R >> >> >>" % (5 + 2 * i)
    )
    objects.append(b"<< /Length %d >>\nstream\n" % len(text) + text + b"\nendstream")
out = bytearray(b"%PDF-1.7\n")
offsets = []
for n, body in enumerate(objects, 1):
    offsets.append(len(out))
    out += b"%d 0 obj\n" % n + body + b"\nendobj\n"
xref = len(out)
out += b"xref\n0 %d\n0000000000 65535 f \n" % (len(objects) + 1)
for o in offsets:
    out += b"%010d 00000 n \n" % o
out += b"trailer\n<< /Size %d /Root 1 0 R >>\nstartxref\n%d\n%%%%EOF\n" % (len(objects) + 1, xref)
pathlib.Path(__file__).with_name("print-impose.pdf").write_bytes(bytes(out))
