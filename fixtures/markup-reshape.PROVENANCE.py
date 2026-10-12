"""Author `markup-reshape.pdf` — a highlight over real text and an arrow with
closed heads, for re-spanning a text markup and filling a line's arrowheads.

    python fixtures/markup-reshape.PROVENANCE.py

Deterministic: no timestamps, no randomness, no producer string, so a re-run in
a clean tree leaves `git status` clean.

Driven by `tools/ui-verify/src/checks/markup_reshape.rs`. One 400 x 400 page.
The text is Courier 12 pt, so every character is 7.2 pt wide and a caret's x is
`40 + 7.2 * index`:

    line 1, baseline y 300:  "alpha beta gamma delta"
    line 2, baseline y 280:  "epsilon zeta eta theta"

`/Annots` order:

    0  /Highlight over "beta" (characters 6..10 of line 1): one quad
       [83.2 298 112 308]; its `/AP` is a hand-drawn yellow block
    1  /Line [60 120 300 120], `/LE [/ClosedArrow /ClosedArrow]`, `/C` blue,
       2 pt, no `/IC`; its `/AP` is a hand-drawn line with outlined heads
"""
import io
import os

NL = "\n"

TEXT = (
    b"BT /F1 12 Tf 40 300 Td (alpha beta gamma delta) Tj ET\n"
    b"BT /F1 12 Tf 40 280 Td (epsilon zeta eta theta) Tj ET\n"
)
HIGHLIGHT_AP = b"/GS0 gs 1 1 0 rg 0 0 28.8 10 re f\n"
LINE_AP = (
    b"0 0 1 RG 2 w 10 10 m 250 10 l S "
    b"10 10 m 22 16 l 22 4 l h S 250 10 m 238 16 l 238 4 l h S\n"
)
COURIER = b"<< /Font << /F1 << /Type /Font /Subtype /Type1 /BaseFont /Courier >> >> >>"
MULTIPLY = b"<< /ExtGState << /GS0 << /BM /Multiply >> >> >>"


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
    b"<< /Type /Page /Parent 2 0 R /MediaBox [0 0 400 400] /Resources " + COURIER + b" "
    b"/Contents 4 0 R /Annots [5 0 R 7 0 R] >>",
    # 4 page content: two lines of text
    stream(b"", TEXT),
    # 5 /Highlight and 6 its appearance
    b"<< /Type /Annot /Subtype /Highlight /Rect [83.2 298 112 308] /P 3 0 R /F 4 "
    b"/C [1 1 0] /QuadPoints [83.2 308 112 308 83.2 298 112 298] /AP << /N 6 0 R >> >>",
    form(b"[0 0 28.8 10]", HIGHLIGHT_AP, MULTIPLY),
    # 7 /Line and 8 its appearance
    b"<< /Type /Annot /Subtype /Line /Rect [50 110 310 130] /P 3 0 R /F 4 "
    b"/L [60 120 300 120] /LE [/ClosedArrow /ClosedArrow] /C [0 0 1] /BS << /W 2 >> "
    b"/AP << /N 8 0 R >> >>",
    form(b"[0 0 260 20]", LINE_AP),
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

out = os.path.join(os.path.dirname(os.path.abspath(__file__)), "markup-reshape.pdf")
io.open(out, "wb").write(raw)
print("wrote", out, len(raw), "bytes;", len(OBJECTS), "objects, xref at", startxref)
