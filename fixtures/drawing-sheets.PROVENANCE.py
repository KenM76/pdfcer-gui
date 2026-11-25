"""Author `drawing-sheets.pdf` - a four-sheet drawing set labelled
`A-101 A-102 A-103 A-104`, with a document information dictionary and an
XMP packet, for the title-block properties and page-label checks.

    python fixtures/drawing-sheets.PROVENANCE.py

Deterministic: no timestamps, no randomness, no producer string.

Driven by `tools/ui-verify/src/checks/info_entries.rs` (an added custom entry
must reach both `/Info` and the XMP packet of the saved file) and
`tools/ui-verify/src/checks/page_labels_follow.rs` (a moved or deleted sheet
keeps its own label). `labelled-pages.pdf` will not do for the second: its
labels `i ii 1 2` are two ranges that read the same under both label policies
for the one move a check can make cheaply; one prefixed range starting at 101
reads differently after any move or delete.

The XMP packet is unfiltered and claims no PDF/A part, so the engine mirrors a
custom key into it (`pdfx:<key>`) and the saved bytes can be searched.

    1  /Catalog  /Pages 2, /PageLabels 3, /Metadata 4
    2  /Pages    4 kids
    3  /Nums [0 << /S /D /P (A-) /St 101 >>]
    4  /Metadata stream, XMP with dc:title
    5  /Info     /Title (Site plan) /Author (Drafting)
    6.. pages    /MediaBox [0 0 420 297]; sheet n carries a bar n units wide
"""

import io
import os

NL = chr(10)


def stream(head, body):
    data = body.encode()
    return (b"<< " + head + b"/Length " + str(len(data)).encode() + b" >>" + NL.encode()
            + b"stream" + NL.encode() + data + NL.encode() + b"endstream")


XMP = NL.join([
    '<?xpacket begin="" id="W5M0MpCehiHzreSzNTczkc9d"?>',
    '<x:xmpmeta xmlns:x="adobe:ns:meta/">',
    '<rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#">',
    '<rdf:Description rdf:about="" xmlns:dc="http://purl.org/dc/elements/1.1/">',
    '<dc:title><rdf:Alt><rdf:li xml:lang="x-default">Site plan</rdf:li></rdf:Alt></dc:title>',
    '</rdf:Description>',
    '</rdf:RDF>',
    '</x:xmpmeta>',
    '<?xpacket end="w"?>',
])

PAGES = 4
FIRST_PAGE = 6
OBJECTS = [
    b"<< /Type /Catalog /Pages 2 0 R /PageLabels 3 0 R /Metadata 4 0 R >>",
    b"<< /Type /Pages /Kids ["
    + b" ".join(str(FIRST_PAGE + 2 * i).encode() + b" 0 R" for i in range(PAGES))
    + b"] /Count " + str(PAGES).encode() + b" >>",
    b"<< /Nums [0 << /S /D /P (A-) /St 101 >>] >>",
    stream(b"/Type /Metadata /Subtype /XML ", XMP),
    b"<< /Title (Site plan) /Author (Drafting) >>",
]
for i in range(PAGES):
    contents = FIRST_PAGE + 2 * i + 1
    OBJECTS.append(b"<< /Type /Page /Parent 2 0 R /MediaBox [0 0 420 297] /Contents "
                   + str(contents).encode() + b" 0 R >>")
    OBJECTS.append(stream(b"", "0 g" + NL + "30 120 " + str(60 * (i + 1)) + " 50 re f"))

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
buf.write(b"<< /Size " + str(len(OBJECTS) + 1).encode() + b" /Root 1 0 R /Info 5 0 R >>"
          + NL.encode())
buf.write(b"startxref" + NL.encode())
buf.write(str(startxref).encode() + NL.encode())
buf.write(b"%%EOF" + NL.encode())

raw = buf.getvalue()
for n, off in enumerate(offsets, start=1):
    header = str(n).encode() + b" 0 obj"
    assert raw[off:off + len(header)] == header, (n, off, raw[off:off + 16])
assert raw[startxref:startxref + 4] == b"xref", raw[startxref:startxref + 16]

out = os.path.join(os.path.dirname(os.path.abspath(__file__)), "drawing-sheets.pdf")
io.open(out, "wb").write(raw)
print("wrote", out, len(raw), "bytes;", len(OBJECTS), "objects, xref at", startxref)
