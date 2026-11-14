"""Author `user-unit.pdf` — one page whose `/UserUnit 2` makes a 306 x 396 box
a Letter sheet (612 x 792 pt) in physical size (ISO 32000-1 §14.11.2, Table 30).

    python fixtures/user-unit.PROVENANCE.py

Deterministic: no timestamps, no randomness, no producer string.

Driven by `print_sizes_a_user_unit_page_physically`: the Print window must plan
the page at its physical size, so Actual size puts 612 x 792 pt on the paper,
not the 306 x 396 the box reads in points. The page draws a frame inset 18 box
units and a filled square, so a half-size print is visible on the preview.
"""

from pathlib import Path

CONTENT = (
    b"0 0 0 RG 2 w 18 18 270 360 re S\n"
    b"0.2 0.4 0.8 rg 103 148 100 100 re f\n"
)

def build() -> bytes:
    objects = [
        b"<< /Type /Catalog /Pages 2 0 R >>",
        b"<< /Type /Pages /Kids [3 0 R] /Count 1 >>",
        b"<< /Type /Page /Parent 2 0 R /MediaBox [0 0 306 396] /UserUnit 2"
        b" /Contents 4 0 R /Resources << >> >>",
        b"<< /Length %d >>\nstream\n" % len(CONTENT) + CONTENT + b"endstream",
    ]
    out = bytearray(b"%PDF-1.6\n%\xe2\xe3\xcf\xd3\n")
    offsets = []
    for number, body in enumerate(objects, start=1):
        offsets.append(len(out))
        out += b"%d 0 obj\n" % number + body + b"\nendobj\n"
    xref = len(out)
    out += b"xref\n0 %d\n0000000000 65535 f \n" % (len(objects) + 1)
    for offset in offsets:
        out += b"%010d 00000 n \n" % offset
    out += b"trailer\n<< /Size %d /Root 1 0 R >>\nstartxref\n%d\n%%%%EOF\n" % (
        len(objects) + 1,
        xref,
    )
    return bytes(out)


if __name__ == "__main__":
    Path(__file__).with_name("user-unit.pdf").write_bytes(build())
