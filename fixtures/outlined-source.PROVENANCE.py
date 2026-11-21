"""Author `outlined-source.pdf` — two pages, a bookmark aimed at each, and a
named destination whose key `outlined-target.pdf` also defines.

    python fixtures/outlined-source.PROVENANCE.py

Deterministic: no timestamps, no randomness, no producer string, so a re-run in
a clean tree leaves `git status` clean.

Driven by `tools/ui-verify/src/checks/insert_outline.rs`: inserting both pages
into `outlined-target.pdf` must bring both bookmarks (`outline_items=2`) and the
one destination (`dests=1`), and the destination's key collides with the
target's, so it arrives renamed (`dests_renamed=1`). An insert that drops the
outline reports `outline_items=0`, which no carrying insert produces.

Structure:

    1  /Catalog   -> /Pages 2, /Outlines 5, /Names << /Dests 8 >>
    2  /Pages     -> [3 4], /Count 2
    3  /Page      Letter, /Contents 9      "SOURCE SHEET A"
    4  /Page      Letter, /Contents 10     "SOURCE SHEET B"
    5  /Outlines  /First 6, /Last 7, /Count 2
    6  item       "Source sheet A", explicit /Dest [3 0 R /Fit]
    7  item       "Source sheet B", explicit /Dest [4 0 R /XYZ 0 792 0]
    8  name tree  (sheet) -> [3 0 R /Fit]; the key `outlined-target.pdf` holds too

Explicit destinations on the bookmarks rather than named ones, so the count of
carried items does not depend on the named-destination half working.

A cross-reference table rather than an xref stream, so byte offsets are
auditable against a hex dump; the offsets are computed from the assembled body,
never typed.
"""

import io
import os

NL = chr(10)
KEY = b"sheet"


def stream(text):
    body = text.encode()
    return (b"<< /Length " + str(len(body)).encode() + b" >>" + NL.encode()
            + b"stream" + NL.encode() + body + b"endstream")


def sheet(label):
    """A frame and one label, so a capture shows which page is which."""
    return stream("0 G 1 w 36 36 540 720 re S" + NL
                  + "BT /Helv 24 Tf 72 700 Td (" + label + ") Tj ET" + NL)


PAGE = (b"<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] "
        b"/Resources << /Font << /Helv << /Type /Font /Subtype /Type1 "
        b"/BaseFont /Helvetica >> >> >> /Contents %d 0 R >>")

OBJECTS = [
    b"<< /Type /Catalog /Pages 2 0 R /Outlines 5 0 R /Names << /Dests 8 0 R >> >>",
    b"<< /Type /Pages /Kids [3 0 R 4 0 R] /Count 2 >>",
    PAGE % 9,
    PAGE % 10,
    b"<< /Type /Outlines /First 6 0 R /Last 7 0 R /Count 2 >>",
    b"<< /Title (Source sheet A) /Parent 5 0 R /Next 7 0 R /Dest [3 0 R /Fit] >>",
    b"<< /Title (Source sheet B) /Parent 5 0 R /Prev 6 0 R /Dest [4 0 R /XYZ 0 792 0] >>",
    b"<< /Names [(" + KEY + b") [3 0 R /Fit]] >>",
    sheet("SOURCE SHEET A"),
    sheet("SOURCE SHEET B"),
]

buf = io.BytesIO()
buf.write(b"%PDF-1.7" + NL.encode())
buf.write(b"%" + bytes([0xE2, 0xE3, 0xCF, 0xD3]) + NL.encode())
offsets = []
for n, body in enumerate(OBJECTS, start=1):
    offsets.append(buf.tell())
    buf.write(str(n).encode() + b" 0 obj" + NL.encode())
    buf.write(body + NL.encode())
    buf.write(b"endobj" + NL.encode())
startxref = buf.tell()
buf.write(b"xref" + NL.encode())
buf.write(b"0 " + str(len(OBJECTS) + 1).encode() + NL.encode())
buf.write(b"0000000000 65535 f " + NL.encode())
for off in offsets:
    # Exactly 20 bytes per entry — ISO 32000-1 7.5.4.
    buf.write(str(off).zfill(10).encode() + b" 00000 n " + NL.encode())
buf.write(b"trailer" + NL.encode())
buf.write(b"<< /Size " + str(len(OBJECTS) + 1).encode() + b" /Root 1 0 R >>" + NL.encode())
buf.write(b"startxref" + NL.encode())
buf.write(str(startxref).encode() + NL.encode())
buf.write(b"%%EOF" + NL.encode())
raw = buf.getvalue()

for n, off in enumerate(offsets, start=1):
    header = str(n).encode() + b" 0 obj"
    assert raw[off:off + len(header)] == header, (n, off, raw[off:off + 16])
assert raw[startxref:startxref + 4] == b"xref", raw[startxref:startxref + 16]

out = os.path.join(os.path.dirname(os.path.abspath(__file__)), "outlined-source.pdf")
with io.open(out, "wb") as f:
    f.write(raw)
print("wrote", out, len(raw), "bytes;", len(OBJECTS), "objects, xref at", startxref)
