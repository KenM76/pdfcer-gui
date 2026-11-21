# -*- coding: utf-8 -*-
"""Build fixtures/clear-image-over-ink.pdf.

One 400 x 300 page. Object 0 is a black square path filling page (220,100)
to (300,200). Object 1, painted over it, is a 2 x 1 RGB image placed at
(100,100) size 200 x 100, with a 2 x 1 /SMask whose left sample is 255
(opaque) and right sample is 0 (fully clear). So the image's right half,
(200..300, 100..200), shows the black square through it, and a click there
belongs to the square; a click in the left half belongs to the image.

Run from the repository root: python -I fixtures/clear-image-over-ink.PROVENANCE.py
"""
from pathlib import Path

CONTENT = b"0 g 220 100 80 100 re f\nq 200 0 0 100 100 100 cm /Im0 Do Q\n"
RGB = b"\xc0\x20\x20\x20\x20\xc0"
ALPHA = b"\xff\x00"


def stream(head, body):
    return head + b" /Length %d >>\nstream\n" % len(body) + body + b"\nendstream"


def build():
    objects = [
        b"<< /Type /Catalog /Pages 2 0 R >>",
        b"<< /Type /Pages /Kids [3 0 R] /Count 1 >>",
        b"<< /Type /Page /Parent 2 0 R /MediaBox [0 0 400 300] "
        b"/Resources << /XObject << /Im0 5 0 R >> >> /Contents 4 0 R >>",
        stream(b"<<", CONTENT),
        stream(
            b"<< /Type /XObject /Subtype /Image /Width 2 /Height 1 /ColorSpace /DeviceRGB "
            b"/BitsPerComponent 8 /SMask 6 0 R",
            RGB,
        ),
        stream(
            b"<< /Type /XObject /Subtype /Image /Width 2 /Height 1 /ColorSpace /DeviceGray "
            b"/BitsPerComponent 8",
            ALPHA,
        ),
    ]
    out = bytearray(b"%PDF-1.7\n%\xe2\xe3\xcf\xd3\n")
    offsets = []
    for number, body in enumerate(objects, start=1):
        offsets.append(len(out))
        out += b"%d 0 obj\n" % number + body + b"\nendobj\n"
    start = len(out)
    out += b"xref\n0 %d\n0000000000 65535 f \n" % (len(objects) + 1)
    for offset in offsets:
        out += b"%010d 00000 n \n" % offset
    out += b"trailer\n<< /Size %d /Root 1 0 R >>\nstartxref\n%d\n%%%%EOF\n" % (
        len(objects) + 1,
        start,
    )
    return bytes(out)


Path(__file__).resolve().parent.joinpath("clear-image-over-ink.pdf").write_bytes(build())
