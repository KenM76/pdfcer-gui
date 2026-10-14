"""Author `annots-align.pdf` — markups, a link, a ce dimension and one patch of
page content, for selecting several annotations and aligning them.

    python fixtures/annots-align.PROVENANCE.py

Deterministic: no timestamps, no randomness, no producer string.

Driven by `tools/ui-verify/src/checks/annot_multiselect.rs` and
`annot_delete_many.rs`. One 400 x 300
page. Every annotation carries an `/NM` the check finds it by in a saved copy:

    /NM (SQ)   /Square [40 200 100 240]   top edge 240, the highest
    /NM (CI)   /Circle [140 180 200 220]
    /NM (LK)   /Link   [240 160 300 180]  border 0, a jump to its own page (a URI would raise the reach-out disclosure on open)
    /NM (LO)   /Square [20 20 80 50]      locked (/F 132), outside both bands
    ce dimension, added by `pdfcer dimension-add` from (60, 80) to (200, 80),
               offset 20: the engine's own sidecar and appearance
    page content: one filled square [330 20 370 60], nothing else

The band from (20, 250) to (210, 170) holds the square and the circle and no
content; the band from (230, 190) to (380, 10) crosses the link and the
content square.
"""
import os
import subprocess
import sys
import tempfile

PDFCER = os.environ.get('PDFCER_EXE', 'D:/Dev/pdfcer/target/release/pdfcer.exe')
OUT = sys.argv[1] if len(sys.argv) > 1 else 'fixtures/annots-align.pdf'
NL = b"\n"

SQUARE_AP = b"0 0 1 RG 2 w 1 1 58 38 re S\n"
LOCKED_AP = b"0 0.5 0 RG 2 w 1 1 58 28 re S\n"
CIRCLE_AP = b"1 0 0 RG 2 w 30 1 m 47 1 59 10 59 20 c 59 30 47 39 30 39 c 13 39 1 30 1 20 c 1 10 13 1 30 1 c S\n"


def stream(dict_body: bytes, data: bytes) -> bytes:
    return (b"<< " + dict_body + b" /Length " + str(len(data)).encode() + b" >>\nstream\n"
            + data + b"endstream")


def form(bbox: bytes, data: bytes) -> bytes:
    return stream(b"/Type /XObject /Subtype /Form /BBox " + bbox + b" /Resources << >>", data)


OBJECTS = [
    b"<< /Type /Catalog /Pages 2 0 R >>",
    b"<< /Type /Pages /Kids [3 0 R] /Count 1 >>",
    b"<< /Type /Page /Parent 2 0 R /MediaBox [0 0 400 300] /Resources << >> "
    b"/Contents 4 0 R /Annots [5 0 R 7 0 R 9 0 R 10 0 R] >>",
    stream(b"", b"0.3 g 330 20 40 40 re f\n"),
    b"<< /Type /Annot /Subtype /Square /NM (SQ) /Rect [40 200 100 240] /P 3 0 R /F 4 "
    b"/C [0 0 1] /BS << /W 2 >> /AP << /N 6 0 R >> >>",
    form(b"[0 0 60 40]", SQUARE_AP),
    b"<< /Type /Annot /Subtype /Circle /NM (CI) /Rect [140 180 200 220] /P 3 0 R /F 4 "
    b"/C [1 0 0] /BS << /W 2 >> /AP << /N 8 0 R >> >>",
    form(b"[0 0 60 40]", CIRCLE_AP),
    b"<< /Type /Annot /Subtype /Link /NM (LK) /Rect [240 160 300 180] /P 3 0 R "
    b"/Border [0 0 0] /Dest [3 0 R /XYZ 0 300 0] >>",
    b"<< /Type /Annot /Subtype /Square /NM (LO) /Rect [20 20 80 50] /P 3 0 R /F 132 "
    b"/C [0 0.5 0] /BS << /W 2 >> /AP << /N 11 0 R >> >>",
    form(b"[0 0 60 30]", LOCKED_AP),
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


with tempfile.TemporaryDirectory() as tmp:
    base = os.path.join(tmp, 'base.pdf')
    with open(base, 'wb') as f:
        f.write(pdf())
    subprocess.run([PDFCER, 'dimension-add', '--points', '60,80 200,80',
                    '--offset', '20', '-o', OUT, base], check=True)
print('wrote', OUT)
