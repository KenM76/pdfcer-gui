"""Author `overlapping-boxes.pdf` - one page of three filled boxes, the
first two overlapping, for the Arrange commands on page content.

    python fixtures/overlapping-boxes.PROVENANCE.py

Deterministic: no timestamps, no randomness, no producer string.

Driven by `tools/ui-verify/src/checks/restack.rs`. A click where the red and
blue boxes overlap selects whichever is drawn last, so it names the stacking
order without reading pixels.

    page 0  /MediaBox [0 0 400 300]
            object 0  red box   (60,60)..(220,180)
            object 1  blue box  (160,100)..(320,220), over the red one
            object 2  green box (20,230)..(80,270), overlapping neither
"""

import io
import os

NL = chr(10)

BODY = (
    "1 0 0 rg 60 60 160 120 re f" + NL
    + "0 0 1 rg 160 100 160 120 re f" + NL
    + "0 0.6 0 rg 20 230 60 40 re f" + NL
).encode()

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

out = os.path.join(os.path.dirname(os.path.abspath(__file__)), "overlapping-boxes.pdf")
io.open(out, "wb").write(raw)
print("wrote", out, len(raw), "bytes;", len(OBJECTS), "objects, xref at", startxref)
