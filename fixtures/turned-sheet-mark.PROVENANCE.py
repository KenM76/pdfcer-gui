"""Author `turned-sheet-mark.pdf`: a letter sheet stored upright with
`/Rotate 270`, blank except one black square near a corner.

    python fixtures/turned-sheet-mark.PROVENANCE.py

Deterministic, hand-written PDF syntax. Driven by
`tools/ui-verify/src/checks/turned_region.rs`: past the zoom where only the
visible window is rasterized, the square must be drawn where the page puts
it. A turned sheet is the case where the window's canvas rectangle and its
PDF user-space rectangle differ by a quarter turn and a flip.
"""

from pathlib import Path

CONTENT = b"0 0 0 rg\n60 600 140 140 re f\n"


def build() -> bytes:
    objs = [
        b"<< /Type /Catalog /Pages 2 0 R >>",
        b"<< /Type /Pages /Kids [3 0 R] /Count 1 >>",
        b"<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] /Rotate 270 "
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
    Path(__file__).with_name("turned-sheet-mark.pdf").write_bytes(build())
