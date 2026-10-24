# -*- coding: utf-8 -*-
"""Build `fixtures/layer-hiding.pdf`: five layers, a sublayer, a locked
sublayer, a layer span nested inside another, and one square on no layer.

Used by `tools/ui-verify/src/checks/layer_hiding.rs`. `/D /Order`:

    Structure -> [Steel, Survey (locked)]
    Bolts
    Notes

Each object is a 160 x 120 pt filled square on an 800 x 800 page, painted
back to front in this order (lower-left corner, in page points):

    Structure  (60, 600)
    Steel      (300, 600)  its span also encloses the Bolts span below
    Bolts      (300, 400)  inside Steel's span: hidden when Steel is hidden
    Notes      (560, 600)
    Survey     (60, 400)   /Locked in /D
    no layer   (560, 400)

Bolts is top level in `/Order` but nested in Steel's content span, so
Structure's switch does not reach it while hiding Steel hides its square.

Rebuild: `python fixtures/layer-hiding.PROVENANCE.py`. Offsets are computed.
"""

import io

NAMES = [b"Structure", b"Steel", b"Bolts", b"Notes", b"Survey"]
FIRST_OCG = 5
IDS = [FIRST_OCG + i for i in range(len(NAMES))]


def ref(i):
    return b"%d 0 R" % IDS[i]


def square(x, y, rgb):
    return b"%s rg %d %d 160 120 re f\n" % (rgb, x, y)


content = (
    b"/OC /OC1 BDC\n" + square(60, 600, b"0.2 0.2 0.8") + b"EMC\n"
    + b"/OC /OC2 BDC\n" + square(300, 600, b"0.6 0.6 0.6")
    + b"/OC /OC3 BDC\n" + square(300, 400, b"0.8 0.4 0.1") + b"EMC\n"
    + b"EMC\n"
    + b"/OC /OC4 BDC\n" + square(560, 600, b"0.1 0.6 0.2") + b"EMC\n"
    + b"/OC /OC5 BDC\n" + square(60, 400, b"0.6 0.1 0.6") + b"EMC\n"
    + square(560, 400, b"0.1 0.1 0.1")
)
ORDER = b"[" + ref(0) + b" [" + ref(1) + b" " + ref(4) + b"] " + ref(2) + b" " + ref(3) + b"]"
props = b" ".join(b"/OC%d %s" % (i + 1, ref(i)) for i in range(len(NAMES)))
all_ocgs = b" ".join(ref(i) for i in range(len(NAMES)))

objects = [
    b"<< /Type /Catalog /Pages 2 0 R /PageMode /UseOC /OCProperties << /OCGs ["
    + all_ocgs + b"] /D << /Name (Default) /BaseState /ON /Order " + ORDER
    + b" /ON [" + all_ocgs + b"] /Locked [" + ref(4) + b"] >> >> >>",
    b"<< /Type /Pages /Kids [3 0 R] /Count 1 >>",
    b"<< /Type /Page /Parent 2 0 R /MediaBox [0 0 800 800] /Resources << "
    b"/Properties << " + props + b" >> >> /Contents 4 0 R >>",
    b"<< /Length %d >>\nstream\n" % len(content) + content + b"endstream",
] + [b"<< /Type /OCG /Name (" + n + b") /Intent /View >>" for n in NAMES]

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

path = "fixtures/layer-hiding.pdf"
io.open(path, "wb").write(bytes(out))
print("wrote %s  %d bytes" % (path, len(out)))
