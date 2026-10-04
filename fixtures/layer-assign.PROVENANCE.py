# -*- coding: utf-8 -*-
"""Build `fixtures/layer-assign.pdf` for `layer_assign_moves_the_selection`.

The page (800 x 600 pt) holds what Move to layer acts on:

- a blue filled box, 100..400 x 100..300, on NO layer: the page object the
  check moves onto `Walls` from the Properties panel's Layer combo;
- a grey outlined box inside `/OC /OC1` (`Walls`), so the layer paints
  something before the move;
- a line of text inside `/OC /OC2` (`Notes`);
- a `/Square` annotation, `/Rect [480 350 720 520]`, with no `/OC`: the
  annotation the check moves onto `Notes` through the Move to layer window.

Two layers, so the combo offers a choice other than the one the object is
already on. Rebuild with `python fixtures/layer-assign.PROVENANCE.py`.
"""

import io

CONTENT = (
    b"q 0 0 0.8 rg 100 100 300 200 re f Q\n"
    b"/OC /OC1 BDC\nq 0.4 0.4 0.4 RG 4 w 450 100 300 150 re S Q\nEMC\n"
    b"/OC /OC2 BDC\nq BT /Helv 20 Tf 0 0.5 0 rg 100 520 Td "
    b"(NOTES LAYER) Tj ET Q\nEMC\n"
)
AP = b"q 1 0 0 RG 3 w 1.5 1.5 237 167 re S Q\n"

objects = [
    b"<< /Type /Catalog /Pages 2 0 R /OCProperties << /OCGs [6 0 R 7 0 R] "
    b"/D << /Name (Default) /BaseState /ON /Order [6 0 R 7 0 R] >> >> >>",
    b"<< /Type /Pages /Kids [3 0 R] /Count 1 >>",
    b"<< /Type /Page /Parent 2 0 R /MediaBox [0 0 800 600] "
    b"/Resources << /Font << /Helv 5 0 R >> /Properties << /OC1 6 0 R "
    b"/OC2 7 0 R >> >> /Contents 4 0 R /Annots [8 0 R] >>",
    b"<< /Length %d >>\nstream\n" % len(CONTENT) + CONTENT + b"endstream",
    b"<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica "
    b"/Encoding /WinAnsiEncoding >>",
    b"<< /Type /OCG /Name (Walls) >>",
    b"<< /Type /OCG /Name (Notes) >>",
    b"<< /Type /Annot /Subtype /Square /Rect [480 350 720 520] /C [1 0 0] "
    b"/BS << /W 3 >> /P 3 0 R /AP << /N 9 0 R >> >>",
    b"<< /Type /XObject /Subtype /Form /BBox [0 0 240 170] /Length %d >>\n"
    b"stream\n" % len(AP) + AP + b"endstream",
]

out = bytearray(b"%PDF-1.7\n%\xe2\xe3\xcf\xd3\n")
offsets = []
for i, body in enumerate(objects, start=1):
    offsets.append(len(out))
    out += b"%d 0 obj\n" % i + body + b"\nendobj\n"
xref_at = len(out)
n = len(objects) + 1
out += b"xref\n0 %d\n0000000000 65535 f \n" % n
for off in offsets:
    out += b"%010d 00000 n \n" % off
out += b"trailer\n<< /Size %d /Root 1 0 R >>\nstartxref\n%d\n%%%%EOF\n" % (n, xref_at)

path = "fixtures/layer-assign.pdf"
io.open(path, "wb").write(bytes(out))
print("wrote %s  %d bytes" % (path, len(out)))
