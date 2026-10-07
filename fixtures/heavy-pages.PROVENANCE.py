"""Author `heavy-pages.pdf` - six letter pages that share one dense, compressed
field of short stroked segments, so every page and every thumbnail takes a
measurable time to render, plus a page-specific mark and one filled rectangle
near the top left that a driven check can select and move.

    python fixtures/heavy-pages.PROVENANCE.py

Deterministic: no timestamps, no randomness, no producer string.

Driven by `tools/ui-verify/src/checks/edit_never_blanks.rs`: one edit on page
1 must re-render page 1 only, must never show a blank page or thumbnail, and
must leave the next input accepted promptly.

    pages 0-5  /MediaBox [0 0 612 792], /Contents [shared own]
    shared     object 3: SEGMENTS stroked segments on a 1.6 pt lattice, Flate
    own        a filled square whose position names the page, then
               `q 0 0 1 rg 40 700 120 60 re f Q` - the movable rectangle
"""

import io
import os
import zlib

NL = chr(10)
PAGES = 6
COLS = 350
ROWS = 430
SEGMENTS = COLS * ROWS


def stream(data, flate=False):
    if flate:
        data = zlib.compress(data, 9)
    head = b"<< /Length " + str(len(data)).encode()
    if flate:
        head += b" /Filter /FlateDecode"
    return head + b" >>" + NL.encode() + b"stream" + NL.encode() + data + NL.encode() + b"endstream"


def shared_content():
    lines = ["q 0.12 w 0.2 0.2 0.2 RG"]
    for r in range(ROWS):
        y = 20 + r * 1.75
        for c in range(COLS):
            x = 26 + c * 1.6
            # A diagonal whose direction depends on (r, c): no randomness.
            if (r * 7 + c * 3) % 4 < 2:
                lines.append(f"{x:.2f} {y:.2f} m {x + 1.2:.2f} {y + 1.2:.2f} l")
            else:
                lines.append(f"{x:.2f} {y + 1.2:.2f} m {x + 1.2:.2f} {y:.2f} l")
    lines.append("S Q")
    return (NL.join(lines) + NL).encode()


def own_content(page):
    x = 200 + page * 60
    return (f"q 0.8 0 0 rg {x} 40 40 40 re f Q" + NL
            + "q 0 0 1 rg 40 700 120 60 re f Q" + NL).encode()


OBJECTS = [
    b"<< /Type /Catalog /Pages 2 0 R >>",
    b"<< /Type /Pages /Kids [" + b" ".join(str(4 + 2 * i).encode() + b" 0 R" for i in range(PAGES))
    + b"] /Count " + str(PAGES).encode() + b" >>",
    stream(shared_content(), flate=True),
]
for i in range(PAGES):
    own = 5 + 2 * i
    OBJECTS.append(b"<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] /Contents [3 0 R "
                   + str(own).encode() + b" 0 R] >>")
    OBJECTS.append(stream(own_content(i)))

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

out = os.path.join(os.path.dirname(os.path.abspath(__file__)), "heavy-pages.pdf")
io.open(out, "wb").write(raw)
print("wrote", out, len(raw), "bytes;", len(OBJECTS), "objects, xref at", startxref)
