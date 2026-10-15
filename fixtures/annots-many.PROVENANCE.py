"""Author `annots-many.pdf` — more markups than one undo step can move.

    python fixtures/annots-many.PROVENANCE.py

Deterministic: no timestamps, no randomness, no producer string.

Driven by `tools/ui-verify/src/checks/annot_too_many.rs`. One 600 x 500 page,
no page content, and 300 Square markups in a 20-column, 15-row grid:

    column i, row j:  /NM (Si_j)  /Rect [20+28i  20+30j  36+28i  32+30j]

All of them share one appearance stream. The window band from (5, 495) to
(595, 5) holds every one. Align left moves the 285 right of column 0, which
is more than the 256 commands one undo step folds.
"""
import sys

OUT = sys.argv[1] if len(sys.argv) > 1 else 'fixtures/annots-many.pdf'
NL = b"\n"
COLS, ROWS = 20, 15
FIRST_ANNOT = 5

AP = b"0 0 1 RG 1 w 0.5 0.5 15 11 re S\n"


def stream(dict_body: bytes, data: bytes) -> bytes:
    return (b"<< " + dict_body + b" /Length " + str(len(data)).encode() + b" >>\nstream\n"
            + data + b"endstream")


def square(i: int, j: int) -> bytes:
    x, y = 20 + 28 * i, 20 + 30 * j
    return (b"<< /Type /Annot /Subtype /Square /NM (S%d_%d) /Rect [%d %d %d %d] /P 3 0 R "
            b"/F 4 /C [0 0 1] /BS << /W 1 >> /AP << /N 4 0 R >> >>"
            % (i, j, x, y, x + 16, y + 12))


annots = [square(i, j) for j in range(ROWS) for i in range(COLS)]
refs = b" ".join(b"%d 0 R" % (FIRST_ANNOT + n) for n in range(len(annots)))
OBJECTS = [
    b"<< /Type /Catalog /Pages 2 0 R >>",
    b"<< /Type /Pages /Kids [3 0 R] /Count 1 >>",
    b"<< /Type /Page /Parent 2 0 R /MediaBox [0 0 600 500] /Resources << >> /Annots ["
    + refs + b"] >>",
    stream(b"/Type /XObject /Subtype /Form /BBox [0 0 16 12] /Resources << >>", AP),
] + annots


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
