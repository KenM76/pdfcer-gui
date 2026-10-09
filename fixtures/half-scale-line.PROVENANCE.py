"""Author `half-scale-line.pdf` - one page carrying a blue line drawn under a
half-scale CTM, and a small grey picture.

    python fixtures/half-scale-line.PROVENANCE.py

Deterministic: no timestamps, no randomness, no producer string.

Driven by `tools/ui-verify/src/checks/stroke_style.rs`. The line's user-space
width is 4 under `0.5 0 0 0.5 0 0 cm`, so it is 2 pt on the page: a width typed
in points must reach the file as twice that number, and a panel that skipped
the conversion reads back half of what was typed. The inline picture is the
subject of the picture-opacity half.

    page 0  /MediaBox [0 0 400 300]
            line (50,150)-(350,150) in page space, 2 pt, solid, opaque
            inline 2x2 grey image at (50,40) 60x40
"""

import io
import os

NL = chr(10)

IMAGE = (
    b"q 60 0 0 40 50 40 cm" + NL.encode()
    + b"BI /W 2 /H 2 /CS /G /BPC 8 ID " + bytes([0x40, 0x80, 0xC0, 0x20]) + NL.encode()
    + b"EI Q" + NL.encode()
)
LINE = (
    "q 0.5 0 0 0.5 0 0 cm" + NL
    + "4 w 0 0 1 RG" + NL
    + "100 300 m 700 300 l S" + NL
    + "Q" + NL
).encode()
BODY = IMAGE + LINE

OBJECTS = [
    b"<< /Type /Catalog /Pages 2 0 R >>",
    b"<< /Type /Pages /Kids [3 0 R] /Count 1 >>",
    b"<< /Type /Page /Parent 2 0 R /MediaBox [0 0 400 300] /Contents 4 0 R >>",
    b"<< /Length " + str(len(BODY)).encode() + b" >>" + NL.encode()
    + b"stream" + NL.encode() + BODY + b"endstream",
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

out = os.path.join(os.path.dirname(os.path.abspath(__file__)), "half-scale-line.pdf")
io.open(out, "wb").write(raw)
print("wrote", out, len(raw), "bytes;", len(OBJECTS), "objects, xref at", startxref)
