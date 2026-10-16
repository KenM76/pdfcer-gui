"""Author `three-squares.pdf`: an upright 300 x 200 pt sheet holding three
separate black squares, each its own path object.

    python fixtures/three-squares.PROVENANCE.py

Deterministic, hand-written PDF syntax. Driven by
`tools/ui-verify/src/checks/tab_object_ring.rs`: three stops is the fewest
that tells a Tab that steps from one that only bounces between two.
"""

from pathlib import Path

CONTENT = b"0 0 0 rg\n30 80 40 40 re f\n130 80 40 40 re f\n230 80 40 40 re f\n"


def build() -> bytes:
    objs = [
        b"<< /Type /Catalog /Pages 2 0 R >>",
        b"<< /Type /Pages /Kids [3 0 R] /Count 1 >>",
        b"<< /Type /Page /Parent 2 0 R /MediaBox [0 0 300 200] "
        b"/Resources << >> /Contents 4 0 R >>",
        b"<< /Length %d >>\nstream\n" % len(CONTENT) + CONTENT + b"endstream",
    ]
    out = bytearray(b"%PDF-1.4\n%\xe2\xe3\xcf\xd3\n")
    offsets = []
    for i, body in enumerate(objs, start=1):
        offsets.append(len(out))
        out += b"%d 0 obj\n" % i + body + b"\nendobj\n"
    xref = len(out)
    out += b"xref\n0 %d\n0000000000 65535 f \n" % (len(objs) + 1)
    for off in offsets:
        out += b"%010d 00000 n \n" % off
    out += b"trailer\n<< /Size %d /Root 1 0 R >>\nstartxref\n%d\n%%%%EOF\n" % (len(objs) + 1, xref)
    return bytes(out)


if __name__ == "__main__":
    Path(__file__).with_name("three-squares.pdf").write_bytes(build())
