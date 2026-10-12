"""Author `link-edit.pdf` — two pages, the first carrying one `/Link` that
runs a JavaScript action, for adding a link by drawing its box and for
changing a selected link's destination and border in Properties.

    python fixtures/link-edit.PROVENANCE.py

Deterministic: no timestamps, no randomness, no producer string, so a re-run in
a clean tree leaves `git status` clean.

Driven by `tools/ui-verify/src/checks/links.rs`. Two 400 x 400 pages. Page 1
draws "See page 2" in Helvetica 12 pt and carries one link:

    /Rect [100 180 300 220], /A << /S /JavaScript /JS (app.alert(1)) >>,
    /Border [0 0 1], /C [0 0 1], no /AP

so Properties reads a one-point blue border and an action pdfcer does not
author, and re-pointing it must disclose that a JavaScript action went.
Page 1 is otherwise empty above y = 260, where the add check draws.
"""
import io
import os

NL = "\n"

TEXT = b"BT /Helv 12 Tf 0 g 110 195 Td (See page 2) Tj ET\n"
HELV = b"<< /Font << /Helv << /Type /Font /Subtype /Type1 /BaseFont /Helvetica >> >> >>"


def stream(dict_body: bytes, data: bytes) -> bytes:
    return (
        b"<< " + dict_body + b" /Length " + str(len(data)).encode() + b" >>\nstream\n"
        + data + b"endstream"
    )


OBJECTS = [
    # 1 catalog, 2 pages, 3 page 1, 4 its content, 5 the link, 6 page 2, 7 its content
    b"<< /Type /Catalog /Pages 2 0 R >>",
    b"<< /Type /Pages /Kids [3 0 R 6 0 R] /Count 2 >>",
    b"<< /Type /Page /Parent 2 0 R /MediaBox [0 0 400 400] /Resources " + HELV
    + b" /Contents 4 0 R /Annots [5 0 R] >>",
    stream(b"", TEXT),
    b"<< /Type /Annot /Subtype /Link /Rect [100 180 300 220] /P 3 0 R /F 4 "
    b"/A << /S /JavaScript /JS (app.alert(1)) >> /Border [0 0 1] /C [0 0 1] >>",
    b"<< /Type /Page /Parent 2 0 R /MediaBox [0 0 400 400] /Resources << >> /Contents 7 0 R >>",
    stream(b"", b""),
]

buf = io.BytesIO()
buf.write(b"%PDF-1.7" + NL.encode() + b"%\xe2\xe3\xcf\xd3" + NL.encode())
offsets = []
for n, body in enumerate(OBJECTS, start=1):
    offsets.append(buf.tell())
    buf.write(str(n).encode() + b" 0 obj" + NL.encode() + body + NL.encode() + b"endobj" + NL.encode())
startxref = buf.tell()
buf.write(b"xref" + NL.encode() + b"0 " + str(len(OBJECTS) + 1).encode() + NL.encode())
buf.write(b"0000000000 65535 f " + NL.encode())
for off in offsets:
    buf.write(b"%010d 00000 n " % off + NL.encode())
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

out = os.path.join(os.path.dirname(os.path.abspath(__file__)), "link-edit.pdf")
io.open(out, "wb").write(raw)
print("wrote", out, len(raw), "bytes;", len(OBJECTS), "objects, xref at", startxref)
