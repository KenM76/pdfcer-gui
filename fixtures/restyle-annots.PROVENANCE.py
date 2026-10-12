"""Author `restyle-annots.pdf` — three annotations whose look another program
drew, for the Properties panel's restyle rows and its redraw offer.

    python fixtures/restyle-annots.PROVENANCE.py

Deterministic: no timestamps, no randomness, no producer string, so a re-run in
a clean tree leaves `git status` clean.

Driven by `tools/ui-verify/src/checks/annot_restyle.rs`. One 400 x 400 page;
`/Annots` order is the index space the checks address:

    0  /FreeText  [40 300 240 340]  border 1 pt black, background pale yellow,
                  text Helvetica 12 pt black; its `/AP` is a hand-drawn
                  rectangle and words, not pdfcer's re-bake, so a restyle is
                  refused as foreign until the operator asks for a redraw
    1  /Stamp     [40 180 200 240]  `/Name /Draft`, no `/CA`; its `/AP` shows
                  the word DRAFT, so opacity (dictionary only) applies directly
    2  /Caret     [280 300 300 320] `/C` red; its `/AP` is a hand-drawn
                  triangle, so a recolour is refused as foreign until redrawn
"""
import io
import os

NL = "\n"

FREETEXT_AP = b"1 1 0.8 rg 0 0 200 40 re f 0 G 1 w 0.5 0.5 199 39 re S BT 0 g /F1 12 Tf 4 24 Td (Hand drawn) Tj ET\n"
STAMP_AP = b"1 0 0 RG 3 w 2 2 156 56 re S BT 1 0 0 rg /F1 28 Tf 30 18 Td (DRAFT) Tj ET\n"
CARET_AP = b"1 0 0 rg 0 0 m 20 0 l 10 20 l f\n"
HELV = b"<< /Font << /F1 << /Type /Font /Subtype /Type1 /BaseFont /Helvetica >> >> >>"


def stream(dict_body: bytes, data: bytes) -> bytes:
    return (
        b"<< " + dict_body + b" /Length " + str(len(data)).encode() + b" >>\nstream\n"
        + data + b"endstream"
    )


def form(bbox: bytes, data: bytes, resources: bytes = b"<< >>") -> bytes:
    return stream(b"/Type /XObject /Subtype /Form /BBox " + bbox + b" /Resources " + resources, data)


OBJECTS = [
    # 1 catalog, 2 pages, 3 page
    b"<< /Type /Catalog /Pages 2 0 R >>",
    b"<< /Type /Pages /Kids [3 0 R] /Count 1 >>",
    b"<< /Type /Page /Parent 2 0 R /MediaBox [0 0 400 400] /Resources << >> "
    b"/Contents 4 0 R /Annots [5 0 R 7 0 R 9 0 R] >>",
    # 4 page content: a frame, so the page is not blank
    stream(b"", b"0.5 G 1 w 10 10 380 380 re S\n"),
    # 5 /FreeText and 6 its appearance
    b"<< /Type /Annot /Subtype /FreeText /Rect [40 300 240 340] /P 3 0 R /F 4 "
    b"/Contents (Hand drawn) /DA (0 g /Helv 12 Tf) /Q 0 /C [0 0 0] "
    b"/BS << /W 1 /S /S >> /IC [1 1 0.8] /AP << /N 6 0 R >> >>",
    form(b"[0 0 200 40]", FREETEXT_AP, HELV),
    # 7 /Stamp and 8 its appearance
    b"<< /Type /Annot /Subtype /Stamp /Rect [40 180 200 240] /P 3 0 R /F 4 "
    b"/Name /Draft /C [1 0 0] /AP << /N 8 0 R >> >>",
    form(b"[0 0 160 60]", STAMP_AP, HELV),
    # 9 /Caret and 10 its appearance
    b"<< /Type /Annot /Subtype /Caret /Rect [280 300 300 320] /P 3 0 R /F 4 "
    b"/C [1 0 0] /AP << /N 10 0 R >> >>",
    form(b"[0 0 20 20]", CARET_AP),
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

out = os.path.join(os.path.dirname(os.path.abspath(__file__)), "restyle-annots.pdf")
io.open(out, "wb").write(raw)
print("wrote", out, len(raw), "bytes;", len(OBJECTS), "objects, xref at", startxref)
