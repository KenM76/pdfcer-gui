# -*- coding: utf-8 -*-
"""Build `fixtures/layer-folders.pdf`: six painted layers whose `/D /Order`
nests them under two labelled folders and one parent layer.

Used by `tools/ui-verify/src/checks/layer_folders.rs`
(`layer_folders_show_and_reorganise`). The tree it declares, as
`pdfcer_core::layers::Layers::order` reads it:

    [0]   folder  "Sheet"      -> [0,0] Title Block, [0,1] Grid Lines
    [1]   layer   Dimensions   -> [1,0] Dimensions Reference (a sublayer)
    [2]   folder  "Services"   -> [2,0] Electrical (off by default)
    [3]   layer   Notes

Every element form Table 101 (ISO 32000-1 §8.11.4.3) permits for `/Order`
appears: a labelled array (a folder), a group followed by an unlabelled array
(sublayers), and plain group references. Each layer paints a distinct mark
inside its own `/OC` span, so a visibility change shows on the page.

Rebuild: `python fixtures/layer-folders.PROVENANCE.py`. Offsets are computed.
"""

import io

LAYERS = [
    (b"Title Block", True, b"0 0 0 RG 3 w 40 40 2304 1604 re S\n"),
    (b"Grid Lines", True, b"0.8 0.8 0.8 RG 0.5 w 140 40 m 140 1644 l S 40 140 m 2344 140 l S\n"),
    (b"Dimensions", True, b"0 0 0.8 RG 1 w 300 1300 m 900 1300 l S\n"),
    (b"Dimensions Reference", True, b"0.4 0.4 1 RG 1 w 300 1000 m 1500 1000 l S\n"),
    (b"Electrical", False, b"0.85 0.1 0.1 rg 300 200 500 300 re f\n"),
    (b"Notes", True, b"BT /Helv 20 Tf 0 0.5 0 rg 140 1500 Td (NOTES) Tj ET\n"),
]
FIRST_OCG = 6
IDS = [FIRST_OCG + i for i in range(len(LAYERS))]


def ref(i):
    return b"%d 0 R" % IDS[i]


ORDER = (
    b"[[(Sheet) " + ref(0) + b" " + ref(1) + b"] "
    + ref(2) + b" [" + ref(3) + b"] "
    + b"[(Services) " + ref(4) + b"] "
    + ref(5) + b"]"
)

content = b"".join(
    b"/OC /OC%d BDC\nq\n" % (i + 1) + paint + b"Q\nEMC\n"
    for i, (_n, _on, paint) in enumerate(LAYERS)
)
props = b" ".join(b"/OC%d %s" % (i + 1, ref(i)) for i in range(len(LAYERS)))
all_ocgs = b" ".join(ref(i) for i in range(len(LAYERS)))
on = b" ".join(ref(i) for i, (_n, v, _p) in enumerate(LAYERS) if v)
off = b" ".join(ref(i) for i, (_n, v, _p) in enumerate(LAYERS) if not v)

objects = [
    b"<< /Type /Catalog /Pages 2 0 R /PageMode /UseOC /OCProperties << /OCGs ["
    + all_ocgs + b"] /D << /Name (Default) /BaseState /ON /Order " + ORDER
    + b" /ON [" + on + b"] /OFF [" + off + b"] >> >> >>",
    b"<< /Type /Pages /Kids [3 0 R] /Count 1 >>",
    b"<< /Type /Page /Parent 2 0 R /MediaBox [0 0 2384 1684] /Resources << "
    b"/Font << /Helv 5 0 R >> /Properties << " + props + b" >> >> /Contents 4 0 R >>",
    b"<< /Length %d >>\nstream\n" % len(content) + content + b"endstream",
    b"<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica /Encoding /WinAnsiEncoding >>",
] + [b"<< /Type /OCG /Name (" + n + b") /Intent /View >>" for n, _v, _p in LAYERS]

out = bytearray(b"%PDF-1.7\n%\xe2\xe3\xcf\xd3\n")
offsets = []
for i, body in enumerate(objects, start=1):
    offsets.append(len(out))
    out += b"%d 0 obj\n" % i + body + b"\nendobj\n"
xref_at = len(out)
n = len(objects) + 1
out += b"xref\n0 %d\n0000000000 65535 f \n" % n
for off_ in offsets:
    out += b"%010d 00000 n \n" % off_
out += b"trailer\n<< /Size %d /Root 1 0 R >>\nstartxref\n%d\n%%%%EOF\n" % (n, xref_at)

path = "fixtures/layer-folders.pdf"
io.open(path, "wb").write(bytes(out))
print("wrote %s  %d bytes" % (path, len(out)))
