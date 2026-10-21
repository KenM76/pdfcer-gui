"""Author `two-photos.pdf` - one page holding two 400 x 400 grey noise images,
each its own image XObject, Flate-compressed, drawn 96 pt square: 300 pixels
per inch, twice the 150 ppi default of the picture options. Two pictures, so a
multi-select has something to gather; noise, so scaling down really shrinks.

    python fixtures/two-photos.PROVENANCE.py

Deterministic: the noise is a fixed linear congruential sequence; no
timestamps, no producer string.

Driven by `tools/ui-verify/src/checks/shrink_pictures.rs`.

    page 0  /MediaBox [0 0 400 200]
            object 0  image /Im0 (object 5), 400 x 400 DeviceGray, FlateDecode,
                      under 96 0 0 96 52 52 cm
            object 1  image /Im1 (object 6), the same, a different noise,
                      under 96 0 0 96 252 52 cm
"""

import io
import os
import zlib

NL = chr(10)

BODY = ("q 96 0 0 96 52 52 cm /Im0 Do Q" + NL + "q 96 0 0 96 252 52 cm /Im1 Do Q" + NL).encode()

SIDE = 400


def noise(seed):
    raw_pixels = bytearray()
    for _ in range(SIDE * SIDE):
        seed = (seed * 1103515245 + 12345) % (1 << 32)
        raw_pixels.append((seed >> 16) & 0xFF)
    return zlib.compress(bytes(raw_pixels), 9)


def image(pixels):
    return (b"<< /Type /XObject /Subtype /Image /Width 400 /Height 400 /ColorSpace /DeviceGray"
            + b" /BitsPerComponent 8 /Filter /FlateDecode /Length " + str(len(pixels)).encode() + b" >>"
            + NL.encode() + b"stream" + NL.encode() + pixels + NL.encode() + b"endstream")


OBJECTS = [
    b"<< /Type /Catalog /Pages 2 0 R >>",
    b"<< /Type /Pages /Kids [3 0 R] /Count 1 >>",
    b"<< /Type /Page /Parent 2 0 R /MediaBox [0 0 400 200] /Contents 4 0 R"
    + b" /Resources << /XObject << /Im0 5 0 R /Im1 6 0 R >> >> >>",
    b"<< /Length " + str(len(BODY)).encode() + b" >>" + NL.encode()
    + b"stream" + NL.encode() + BODY + b"endstream",
    image(noise(12345)),
    image(noise(67890)),
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

out = os.path.join(os.path.dirname(os.path.abspath(__file__)), "two-photos.pdf")
io.open(out, "wb").write(raw)
print("wrote", out, len(raw), "bytes;", len(OBJECTS), "objects, xref at", startxref)
