"""Author `labelled-pages.pdf` - four pages whose page labels are
`i ii 1 2`: a roman front-matter range, then a decimal range from 1.

    python fixtures/labelled-pages.PROVENANCE.py

Deterministic: no timestamps, no randomness, no producer string.

Driven by `tools/ui-verify/src/checks/extract_pages_labels.rs`. Extracting
pages 3 and 4 keeps their labels as `1 2` (one decimal range in the new file)
or drops the tree, and the extraction's report says which.

    pages 0-3  /MediaBox [0 0 300 200], page n carries a bar n+1 units wide
    /PageLabels  /Nums [0 << /S /r >> 2 << /S /D >>]
"""

import io
import os

NL = chr(10)


def stream(body):
    data = body.encode()
    return (b"<< /Length " + str(len(data)).encode() + b" >>" + NL.encode()
            + b"stream" + NL.encode() + data + b"endstream")


PAGES = 4
OBJECTS = [
    b"<< /Type /Catalog /Pages 2 0 R /PageLabels 3 0 R >>",
    b"<< /Type /Pages /Kids [" + b" ".join(str(4 + 2 * i).encode() + b" 0 R" for i in range(PAGES))
    + b"] /Count " + str(PAGES).encode() + b" >>",
    b"<< /Nums [0 << /S /r >> 2 << /S /D >>] >>",
]
for i in range(PAGES):
    contents = 5 + 2 * i
    OBJECTS.append(b"<< /Type /Page /Parent 2 0 R /MediaBox [0 0 300 200] /Contents "
                   + str(contents).encode() + b" 0 R >>")
    OBJECTS.append(stream("0 g" + NL + "20 80 " + str(40 * (i + 1)) + " 40 re f" + NL))

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

out = os.path.join(os.path.dirname(os.path.abspath(__file__)), "labelled-pages.pdf")
io.open(out, "wb").write(raw)
print("wrote", out, len(raw), "bytes;", len(OBJECTS), "objects, xref at", startxref)
