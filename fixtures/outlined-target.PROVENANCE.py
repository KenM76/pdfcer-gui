"""Author `outlined-target.pdf` — one page, one bookmark and one named
destination: the document `outlined-source.pdf`'s pages are inserted into.

    python fixtures/outlined-target.PROVENANCE.py

Deterministic: no timestamps, no randomness, no producer string, so a re-run in
a clean tree leaves `git status` clean.

Driven by `tools/ui-verify/src/checks/insert_outline.rs`. Its bookmark is the
one the carried bookmarks must follow — `InsertedOutline::Append` puts them
after the target's last — and its destination key `sheet` is the one the source
also defines, so the carried destination must be renamed rather than replace
this one.

Structure:

    1  /Catalog   -> /Pages 2, /Outlines 4, /Names << /Dests 6 >>
    2  /Pages     -> [3], /Count 1
    3  /Page      Letter, /Contents 7      "TARGET COVER"
    4  /Outlines  /First 5, /Last 5, /Count 1
    5  item       "Target cover", explicit /Dest [3 0 R /Fit]
    6  name tree  (sheet) -> [3 0 R /Fit]

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


OBJECTS = [
    b"<< /Type /Catalog /Pages 2 0 R /Outlines 4 0 R /Names << /Dests 6 0 R >> >>",
    b"<< /Type /Pages /Kids [3 0 R] /Count 1 >>",
    b"<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] "
    b"/Resources << /Font << /Helv << /Type /Font /Subtype /Type1 "
    b"/BaseFont /Helvetica >> >> >> /Contents 7 0 R >>",
    b"<< /Type /Outlines /First 5 0 R /Last 5 0 R /Count 1 >>",
    b"<< /Title (Target cover) /Parent 4 0 R /Dest [3 0 R /Fit] >>",
    b"<< /Names [(" + KEY + b") [3 0 R /Fit]] >>",
    stream("0 G 1 w 36 36 540 720 re S" + NL
           + "BT /Helv 24 Tf 72 700 Td (TARGET COVER) Tj ET" + NL),
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

out = os.path.join(os.path.dirname(os.path.abspath(__file__)), "outlined-target.pdf")
with io.open(out, "wb") as f:
    f.write(raw)
print("wrote", out, len(raw), "bytes;", len(OBJECTS), "objects, xref at", startxref)
