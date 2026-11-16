"""Author `markup-stack.pdf` — a small unfilled circle under a large unfilled
square, for clicking through an empty box to the markup beneath it and for
stepping a markup forward past a popup.

    python fixtures/markup-stack.PROVENANCE.py

Deterministic: no timestamps, no randomness, no producer string.

Driven by `tools/ui-verify/src/checks/markup_click_through.rs`. One 400 x 400
page. `/Annots` order, bottom first:

    0  /Circle, /Rect [40 180 80 220], red 2 pt, no /IC, its ring centred
       on the square's left edge
    1  /Popup of the circle, closed, so never painted
    2  /Square, /Rect [60 60 340 340], blue 2 pt, no /IC

The square is on top. Near (60, 219) the circle's ring crosses the square's
edge, so whichever is higher takes a click there.
"""
import io
import os

NL = "\n"

CIRCLE_AP = (
    b"1 0 0 RG 2 w 20 39 m 30.5 39 39 30.5 39 20 c 39 9.5 30.5 1 20 1 c "
    b"9.5 1 1 9.5 1 20 c 1 30.5 9.5 39 20 39 c S\n"
)
SQUARE_AP = b"0 0 1 RG 2 w 1 1 278 278 re S\n"


def stream(dict_body: bytes, data: bytes) -> bytes:
    return (
        b"<< " + dict_body + b" /Length " + str(len(data)).encode() + b" >>\nstream\n"
        + data + b"endstream"
    )


def form(bbox: bytes, data: bytes) -> bytes:
    return stream(b"/Type /XObject /Subtype /Form /BBox " + bbox + b" /Resources << >>", data)


OBJECTS = [
    b"<< /Type /Catalog /Pages 2 0 R >>",
    b"<< /Type /Pages /Kids [3 0 R] /Count 1 >>",
    b"<< /Type /Page /Parent 2 0 R /MediaBox [0 0 400 400] /Resources << >> "
    b"/Contents 4 0 R /Annots [5 0 R 9 0 R 7 0 R] >>",
    stream(b"", b""),
    b"<< /Type /Annot /Subtype /Circle /Rect [40 180 80 220] /P 3 0 R /F 4 "
    b"/C [1 0 0] /BS << /W 2 >> /Popup 9 0 R /AP << /N 6 0 R >> >>",
    form(b"[0 0 40 40]", CIRCLE_AP),
    b"<< /Type /Annot /Subtype /Square /Rect [60 60 340 340] /P 3 0 R /F 4 "
    b"/C [0 0 1] /BS << /W 2 >> /AP << /N 8 0 R >> >>",
    form(b"[0 0 280 280]", SQUARE_AP),
    b"<< /Type /Annot /Subtype /Popup /Rect [100 300 200 350] /P 3 0 R /Parent 5 0 R "
    b"/Open false >>",
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

out = os.path.join(os.path.dirname(os.path.abspath(__file__)), "markup-stack.pdf")
io.open(out, "wb").write(raw)
print("wrote", out, len(raw), "bytes;", len(OBJECTS), "objects, xref at", startxref)
