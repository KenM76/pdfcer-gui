# -*- coding: utf-8 -*-
"""Build `fixtures/print-colour.pdf`: one Letter page that is mostly saturated
red, a 468 x 648 pt rectangle in DeviceRGB 1 0 0 at 72 72.

Printed in colour the preview holds red; in grey every red pixel becomes the
same mid grey; in black only every red pixel becomes black. A page this full
of one colour makes "the recolour changed pixels" a large, unmistakable count.

Run: python fixtures/print-colour.PROVENANCE.py
"""
import pathlib

content = b"1 0 0 rg 72 72 468 648 re f"
objects = [
    b"<< /Type /Catalog /Pages 2 0 R >>",
    b"<< /Type /Pages /Kids [3 0 R] /Count 1 >>",
    b"<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] /Contents 4 0 R >>",
    b"<< /Length %d >>\nstream\n" % len(content) + content + b"\nendstream",
]
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
pathlib.Path(__file__).with_name("print-colour.pdf").write_bytes(bytes(out))
