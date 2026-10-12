"""Author `widget-turn.pdf` — one text field with a drawn appearance, for
tilting a form field's box by its rotate grip and setting its opacity.

    python fixtures/widget-turn.PROVENANCE.py

Deterministic: no timestamps, no randomness, no producer string, so a re-run in
a clean tree leaves `git status` clean.

Driven by `tools/ui-verify/src/checks/widget_turn.rs`. One 400 x 400 page with
one text field, `Name`, its widget merged into the field:

    /Rect [100 180 300 220], upright (no `/MK /R`), no `/CA`
    /AP /N: a form XObject, `/BBox [0 0 200 40]`, no `/Matrix`, drawing a
    grey border and the value "Ada" in Helvetica 12 pt

`turn_widget` needs a normal appearance with a `/BBox`; this one has both, and
is referenced from one widget only, so it is never shared.
"""
import io
import os

NL = "\n"

AP = b"0.5 G 1 w 0.5 0.5 199 39 re S BT /Helv 12 Tf 0 g 4 14 Td (Ada) Tj ET\n"
HELV = b"<< /Font << /Helv << /Type /Font /Subtype /Type1 /BaseFont /Helvetica >> >> >>"


def stream(dict_body: bytes, data: bytes) -> bytes:
    return (
        b"<< " + dict_body + b" /Length " + str(len(data)).encode() + b" >>\nstream\n"
        + data + b"endstream"
    )


OBJECTS = [
    # 1 catalog, 2 pages, 3 page
    b"<< /Type /Catalog /Pages 2 0 R /AcroForm << /Fields [5 0 R] /DA (/Helv 12 Tf 0 g) "
    b"/DR " + HELV + b" >> >>",
    b"<< /Type /Pages /Kids [3 0 R] /Count 1 >>",
    b"<< /Type /Page /Parent 2 0 R /MediaBox [0 0 400 400] /Resources << >> "
    b"/Contents 4 0 R /Annots [5 0 R] >>",
    # 4 an empty page content stream
    stream(b"", b""),
    # 5 the field and its widget, 6 the normal appearance
    b"<< /Type /Annot /Subtype /Widget /FT /Tx /T (Name) /V (Ada) /Rect [100 180 300 220] "
    b"/P 3 0 R /F 4 /DA (/Helv 12 Tf 0 g) /MK << /BC [0.5 0.5 0.5] >> /AP << /N 6 0 R >> >>",
    stream(b"/Type /XObject /Subtype /Form /BBox [0 0 200 40] /Resources " + HELV, AP),
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

out = os.path.join(os.path.dirname(os.path.abspath(__file__)), "widget-turn.pdf")
io.open(out, "wb").write(raw)
print("wrote", out, len(raw), "bytes;", len(OBJECTS), "objects, xref at", startxref)
