"""Author `cropped-sheets.pdf` — two sheets that each carry an explicit visible
area (`/CropBox`), one matching its paper and one cropped to part of it.

    python fixtures/cropped-sheets.PROVENANCE.py

Deterministic: no timestamps, no randomness, no producer string.

Driven by `tools/ui-verify/src/checks/page_size.rs` for `OPERATOR_REQUESTS.md`
O250. A resize that rewrites `/MediaBox` and leaves `/CropBox` alone looks like
it did nothing, because every viewer frames the page by the visible area. No
other fixture carries a `/CropBox`, so on them a stale visible area cannot be
told from a moved one.

Page 0 is smaller than A6, so the check GROWS it. Shrinking cannot show the
defect: the visible area is clipped to the sheet, so a stale one shrinks too.
Page 1 is smaller than page 0 so that, in continuous view, page 0 has the most
area on screen and is the page the resize acts on.

    page 0  /MediaBox [0 0 200 280]   /CropBox equal   - the sheet the check resizes;
                                                         its visible area must follow
    page 1  /MediaBox [0 0 180 250]   /CropBox inset   - the negative control
"""

import io
import os

NL = chr(10)

SHEETS = [
    # (media w, media h, crop box, label)
    (200, 280, (0, 0, 200, 280), "SHEET 0  CROP = PAPER"),
    (180, 250, (20, 20, 160, 230), "SHEET 1  CROP INSET"),
]


def content(w, h, crop, label):
    llx, lly, urx, ury = crop
    return (
        "0 G 1 w" + NL
        + f"{llx + 10} {lly + 10} {urx - llx - 20} {ury - lly - 20} re S" + NL
        + f"BT /Helv 9 Tf {llx + 14} {ury - 24} Td ({label}) Tj ET" + NL
    ).encode()


OBJECTS = [
    b"<< /Type /Catalog /Pages 2 0 R >>",
    b"<< /Type /Pages /Kids ["
    + b" ".join(f"{3 + 2 * i} 0 R".encode() for i in range(len(SHEETS)))
    + b"] /Count " + str(len(SHEETS)).encode() + b" >>",
]
for i, (w, h, crop, label) in enumerate(SHEETS):
    body = content(w, h, crop, label)
    OBJECTS.append(
        b"<< /Type /Page /Parent 2 0 R "
        + f"/MediaBox [0 0 {w} {h}] /CropBox [{' '.join(map(str, crop))}] ".encode()
        + b"/Resources << /Font << /Helv << /Type /Font /Subtype /Type1 "
        + b"/BaseFont /Helvetica >> >> >> "
        + f"/Contents {4 + 2 * i} 0 R >>".encode()
    )
    OBJECTS.append(
        b"<< /Length " + str(len(body)).encode() + b" >>" + NL.encode()
        + b"stream" + NL.encode() + body + b"endstream"
    )

buf = io.BytesIO()
buf.write(b"%PDF-1.7" + NL.encode())
buf.write(b"%" + bytes([0xE2, 0xE3, 0xCF, 0xD3]) + NL.encode())

offsets = []
for n, body in enumerate(OBJECTS, start=1):
    offsets.append(buf.tell())
    buf.write(str(n).encode() + b" 0 obj" + NL.encode())
    buf.write(body + NL.encode())
    buf.write(b"endobj" + NL.encode())

startxref = buf.tell()
buf.write(b"xref" + NL.encode())
buf.write(b"0 " + str(len(OBJECTS) + 1).encode() + NL.encode())
buf.write(b"0000000000 65535 f " + NL.encode())
for off in offsets:
    buf.write(str(off).zfill(10).encode() + b" 00000 n " + NL.encode())
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

out = os.path.join(os.path.dirname(os.path.abspath(__file__)), "cropped-sheets.pdf")
io.open(out, "wb").write(raw)
print("wrote", out, len(raw), "bytes;", len(OBJECTS), "objects, xref at", startxref)
