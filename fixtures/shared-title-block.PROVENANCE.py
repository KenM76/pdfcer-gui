# -*- coding: utf-8 -*-
"""Build fixtures/shared-title-block.pdf.

Three 400 x 300 pages. Form 4, the "title block", is placed on every page and
twice on page 2, so the engine's `invocation_map` says 4 places on 3 pages.
Form 5, a "detail", is placed once, on page 1 only. Each form strokes one
rectangle; nothing else is on any page.

Run from the repository root: python -I fixtures/shared-title-block.PROVENANCE.py
"""
from pathlib import Path

TITLE = b"0 0 0 RG 4 w 0 0 m 160 0 l 160 60 l 0 60 l h S\n"
DETAIL = b"0 0 0 RG 2 w 0 0 m 80 0 l 80 80 l 0 80 l h S\n"
PAGES = [
    b"q 1 0 0 1 220 20 cm /Tb Do Q q 1 0 0 1 40 180 cm /Dt Do Q\n",
    b"q 1 0 0 1 220 20 cm /Tb Do Q q 1 0 0 1 20 200 cm /Tb Do Q\n",
    b"q 1 0 0 1 220 20 cm /Tb Do Q\n",
]


def stream(head, body):
    return head + b" /Length %d >>\nstream\n" % len(body) + body + b"endstream"


def build():
    # 1 catalog, 2 pages, 3 resources, 4 title form, 5 detail form,
    # then page and content pairs from 6.
    kids = b" ".join(b"%d 0 R" % (6 + 2 * i) for i in range(len(PAGES)))
    objects = [
        b"<< /Type /Catalog /Pages 2 0 R >>",
        b"<< /Type /Pages /Kids [" + kids + b"] /Count %d >>" % len(PAGES),
        b"<< /XObject << /Tb 4 0 R /Dt 5 0 R >> >>",
        stream(b"<< /Type /XObject /Subtype /Form /BBox [-4 -4 164 64] /Resources << >>", TITLE),
        stream(b"<< /Type /XObject /Subtype /Form /BBox [-2 -2 82 82] /Resources << >>", DETAIL),
    ]
    for i, content in enumerate(PAGES):
        objects.append(
            b"<< /Type /Page /Parent 2 0 R /MediaBox [0 0 400 300] "
            b"/Resources 3 0 R /Contents %d 0 R >>" % (7 + 2 * i)
        )
        objects.append(stream(b"<<", content))
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


Path(__file__).resolve().parent.joinpath("shared-title-block.pdf").write_bytes(build())
