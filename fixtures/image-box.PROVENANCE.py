"""Author `image-box.pdf` - one page holding one image XObject, two blue
pixels by one, drawn into a 200 x 100 pt box, for Format > Replace image.

    python fixtures/image-box.PROVENANCE.py

Deterministic: no timestamps, no randomness, no producer string.

Driven by `tools/ui-verify/src/checks/replaceimage.rs`.

    page 0  /MediaBox [0 0 400 300]
            object 0  image /Im0 (object 5), 2 x 1 DeviceRGB, unfiltered,
                      under 200 0 0 100 100 100 cm: x 100..300, y 100..200
"""

import io
import os

NL = chr(10)

BODY = ("q 200 0 0 100 100 100 cm /Im0 Do Q" + NL).encode()

PIXELS = bytes([0, 64, 200, 0, 64, 200])

OBJECTS = [
    b"<< /Type /Catalog /Pages 2 0 R >>",
    b"<< /Type /Pages /Kids [3 0 R] /Count 1 >>",
    b"<< /Type /Page /Parent 2 0 R /MediaBox [0 0 400 300] /Contents 4 0 R"
    + b" /Resources << /XObject << /Im0 5 0 R >> >> >>",
    b"<< /Length " + str(len(BODY)).encode() + b" >>" + NL.encode()
    + b"stream" + NL.encode() + BODY + b"endstream",
    b"<< /Type /XObject /Subtype /Image /Width 2 /Height 1 /ColorSpace /DeviceRGB"
    + b" /BitsPerComponent 8 /Length " + str(len(PIXELS)).encode() + b" >>" + NL.encode()
    + b"stream" + NL.encode() + PIXELS + NL.encode() + b"endstream",
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

out = os.path.join(os.path.dirname(os.path.abspath(__file__)), "image-box.pdf")
io.open(out, "wb").write(raw)
print("wrote", out, len(raw), "bytes;", len(OBJECTS), "objects, xref at", startxref)
