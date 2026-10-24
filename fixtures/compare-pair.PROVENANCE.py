"""Author `compare-old.pdf` and `compare-new.pdf` — two versions of one
two-sheet note set with known differences, for Tools > Compare.

    python fixtures/compare-pair.PROVENANCE.py fixtures

Deterministic: no timestamps, no randomness, no producer string.

Driven by `tools/ui-verify/src/checks/compare_documents.rs`. Two 612 x 792
pages each, Helvetica 14 pt, one line per `Tj`. Page 1 is the same in both, so
every change is on page 2 and a jump to one has to turn the page:

    page 2, y  old                                  new
    700        Pump station drawing revision A      Pump station drawing revision B
    650        Pipe diameter 200 mm                 Pipe diameter 250 mm
    600        Remove the old valve                 (gone)
    550        Contractor to verify levels          Contractor to verify levels
    500        (absent)                             Add a check valve at the outlet

The text comparison therefore finds two replacements (`A` -> `B`, `200` ->
`250`), one deletion and one insertion, all on page 2. The new page 2 also
carries a filled square at [400 100 500 200] the old one lacks, which only
the appearance comparison sees.
"""
import sys

DIR = sys.argv[1] if len(sys.argv) > 1 else 'fixtures'
NL = b"\n"

SHEET_ONE = [
    (700, b"General notes"),
    (650, b"All dimensions in millimetres"),
]
OLD_LINES = [
    (700, b"Pump station drawing revision A"),
    (650, b"Pipe diameter 200 mm"),
    (600, b"Remove the old valve"),
    (550, b"Contractor to verify levels"),
]
NEW_LINES = [
    (700, b"Pump station drawing revision B"),
    (650, b"Pipe diameter 250 mm"),
    (550, b"Contractor to verify levels"),
    (500, b"Add a check valve at the outlet"),
]
NEW_DRAWING = b"0.2 g 400 100 100 100 re f\n"


def stream(data: bytes) -> bytes:
    return b"<< /Length " + str(len(data)).encode() + b" >>\nstream\n" + data + b"endstream"


def content(lines, drawing: bytes) -> bytes:
    out = bytearray()
    for y, text in lines:
        out += b"BT /F1 14 Tf 72 " + str(y).encode() + b" Td (" + text + b") Tj ET\n"
    return bytes(out) + drawing


def page(contents: int) -> bytes:
    return (b"<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] "
            b"/Resources << /Font << /F1 3 0 R >> >> /Contents "
            + str(contents).encode() + b" 0 R >>")


def pdf(lines, drawing: bytes) -> bytes:
    objects = [
        b"<< /Type /Catalog /Pages 2 0 R >>",
        b"<< /Type /Pages /Kids [4 0 R 6 0 R] /Count 2 >>",
        b"<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica /Encoding /WinAnsiEncoding >>",
        page(5),
        stream(content(SHEET_ONE, b"")),
        page(7),
        stream(content(lines, drawing)),
    ]
    out = bytearray(b"%PDF-1.7" + NL + b"%\xe2\xe3\xcf\xd3" + NL)
    offsets = []
    for n, body in enumerate(objects, start=1):
        offsets.append(len(out))
        out += str(n).encode() + b" 0 obj" + NL + body + NL + b"endobj" + NL
    xref = len(out)
    out += b"xref" + NL + b"0 " + str(len(objects) + 1).encode() + NL
    out += b"0000000000 65535 f " + NL
    for off in offsets:
        out += b"%010d 00000 n " % off + NL
    out += b"trailer" + NL + b"<< /Size " + str(len(objects) + 1).encode() + b" /Root 1 0 R >>" + NL
    out += b"startxref" + NL + str(xref).encode() + NL + b"%%EOF" + NL
    return bytes(out)


for name, lines, drawing in (
    ('compare-old.pdf', OLD_LINES, b""),
    ('compare-new.pdf', NEW_LINES, NEW_DRAWING),
):
    with open(f"{DIR}/{name}", 'wb') as f:
        f.write(pdf(lines, drawing))
