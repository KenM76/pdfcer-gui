"""Author `annots-nest.pdf` — three plain markups on one page, for grouping a
group with a third markup and taking it apart one level.

    python fixtures/annots-nest.PROVENANCE.py

Deterministic: no timestamps, no randomness, no producer string. Pure Python.

Driven by `tools/ui-verify/src/checks/group_of_groups.rs`. One 400 x 300
page, no page content. Every annotation carries an `/NM` the check finds it
by in a saved copy:

    /NM (SQ)   /Square [40 200 100 240]
    /NM (CI)   /Circle [140 180 200 220]
    /NM (TR)   /Square [260 200 320 240]
"""
import sys

OUT = sys.argv[1] if len(sys.argv) > 1 else 'fixtures/annots-nest.pdf'
NL = b"\n"

SQUARE_AP = b"0 0 1 RG 2 w 1 1 58 38 re S\n"
THIRD_AP = b"0 0.5 0 RG 2 w 1 1 58 38 re S\n"
CIRCLE_AP = b"1 0 0 RG 2 w 30 1 m 47 1 59 10 59 20 c 59 30 47 39 30 39 c 13 39 1 30 1 20 c 1 10 13 1 30 1 c S\n"


def stream(dict_body: bytes, data: bytes) -> bytes:
    return (b"<< " + dict_body + b" /Length " + str(len(data)).encode() + b" >>\nstream\n"
            + data + b"endstream")


def form(data: bytes) -> bytes:
    return stream(b"/Type /XObject /Subtype /Form /BBox [0 0 60 40] /Resources << >>", data)


OBJECTS = [
    b"<< /Type /Catalog /Pages 2 0 R >>",
    b"<< /Type /Pages /Kids [3 0 R] /Count 1 >>",
    b"<< /Type /Page /Parent 2 0 R /MediaBox [0 0 400 300] /Resources << >> "
    b"/Contents 4 0 R /Annots [5 0 R 7 0 R 9 0 R] >>",
    stream(b"", b""),
    b"<< /Type /Annot /Subtype /Square /NM (SQ) /Rect [40 200 100 240] /P 3 0 R /F 4 "
    b"/C [0 0 1] /BS << /W 2 >> /AP << /N 6 0 R >> >>",
    form(SQUARE_AP),
    b"<< /Type /Annot /Subtype /Circle /NM (CI) /Rect [140 180 200 220] /P 3 0 R /F 4 "
    b"/C [1 0 0] /BS << /W 2 >> /AP << /N 8 0 R >> >>",
    form(CIRCLE_AP),
    b"<< /Type /Annot /Subtype /Square /NM (TR) /Rect [260 200 320 240] /P 3 0 R /F 4 "
    b"/C [0 0.5 0] /BS << /W 2 >> /AP << /N 10 0 R >> >>",
    form(THIRD_AP),
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
