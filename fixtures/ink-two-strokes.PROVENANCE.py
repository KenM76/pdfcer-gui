"""Author `ink-two-strokes.pdf` — one freehand mark of two strokes, for
editing one stroke of it.

    python fixtures/ink-two-strokes.PROVENANCE.py

Deterministic: no timestamps, no randomness, no producer string.

Driven by `tools/ui-verify/src/checks/ink_stroke_edit.rs`. One 400 x 300 page,
no page content, and one `/Ink` annotation, `/NM (INK)`, black, width 3:

    stroke 0   (60, 200) -> (160, 200) -> (160, 240)    the upper hook
    stroke 1   (60, 80)  -> (160, 80)                   the lower bar

`/Rect [55 75 165 245]`. Its appearance is pdfcer's polyline of the two
strokes, so a re-bake changes nothing a reader would notice.
"""
import sys

OUT = sys.argv[1] if len(sys.argv) > 1 else 'fixtures/ink-two-strokes.pdf'
NL = b"\n"

STROKES = [[(60, 200), (160, 200), (160, 240)], [(60, 80), (160, 80)]]
RECT = (55, 75, 165, 245)


def stream(dict_body: bytes, data: bytes) -> bytes:
    return (b"<< " + dict_body + b" /Length " + str(len(data)).encode() + b" >>\nstream\n"
            + data + b"endstream")


def appearance() -> bytes:
    ops = [b"0 G 3 w 1 J 1 j"]
    for stroke in STROKES:
        (x, y), rest = stroke[0], stroke[1:]
        ops.append(b"%d %d m" % (x - RECT[0], y - RECT[1]))
        ops += [b"%d %d l" % (px - RECT[0], py - RECT[1]) for px, py in rest]
        ops.append(b"S")
    return b"\n".join(ops) + NL


INK_LIST = b"[" + b" ".join(
    b"[" + b" ".join(b"%d %d" % p for p in s) + b"]" for s in STROKES) + b"]"
BBOX = b"[0 0 %d %d]" % (RECT[2] - RECT[0], RECT[3] - RECT[1])

OBJECTS = [
    b"<< /Type /Catalog /Pages 2 0 R >>",
    b"<< /Type /Pages /Kids [3 0 R] /Count 1 >>",
    b"<< /Type /Page /Parent 2 0 R /MediaBox [0 0 400 300] /Resources << >> "
    b"/Contents 4 0 R /Annots [5 0 R] >>",
    stream(b"", b""),
    b"<< /Type /Annot /Subtype /Ink /NM (INK) /Rect [%d %d %d %d] /P 3 0 R /F 4 "
    % RECT + b"/C [0 0 0] /BS << /W 3 >> /InkList " + INK_LIST + b" /AP << /N 6 0 R >> >>",
    stream(b"/Type /XObject /Subtype /Form /BBox " + BBOX + b" /Resources << >>", appearance()),
]


def pdf() -> bytes:
    out = bytearray(b"%PDF-1.7" + NL + b"%\xe2\xe3\xcf\xd3" + NL)
    offsets = []
    for n, body in enumerate(OBJECTS, start=1):
        offsets.append(len(out))
        out += str(n).encode() + b" 0 obj" + NL + body + NL + b"endobj" + NL
    xref = len(out)
    out += b"xref" + NL + b"0 " + str(len(OBJECTS) + 1).encode() + NL
    out += b"0000000000 65535 f " + NL
    for off in offsets:
        out += b"%010d 00000 n " % off + NL
    out += b"trailer" + NL + b"<< /Size " + str(len(OBJECTS) + 1).encode() + b" /Root 1 0 R >>" + NL
    out += b"startxref" + NL + str(xref).encode() + NL + b"%%EOF" + NL
    return bytes(out)


with open(OUT, 'wb') as f:
    f.write(pdf())
print('wrote', OUT)
