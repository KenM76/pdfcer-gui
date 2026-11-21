"""Author `annot-facts.pdf` — one annotation neither restyle verb reads, for
the Properties read of `/C` and `/Name` on any subtype.

    python fixtures/annot-facts.PROVENANCE.py

Deterministic: no timestamps, no randomness, no producer string, so a re-run in
a clean tree leaves `git status` clean.

Driven by `tools/ui-verify/src/checks/annot_facts.rs`. One 400 x 400 page
carrying a /FreeText at /Rect [100 100 200 140] with no /DA, so
`text_spec_from_dict` declines it and `spec_from_dict` has no arm for it. It
carries /C [1 0 0] and /Name /Comment, which only `annot::Annotation::color`
and `annot::Annotation::icon` report.
"""
import io
import os

NL = b"\n"


def stream(data: bytes) -> bytes:
    return b"<< /Length " + str(len(data)).encode() + b" >>\nstream\n" + data + b"endstream"


OBJECTS = [
    # 1 catalog, 2 pages, 3 the page, 4 an empty content stream, 5 the mark
    b"<< /Type /Catalog /Pages 2 0 R >>",
    b"<< /Type /Pages /Kids [3 0 R] /Count 1 >>",
    b"<< /Type /Page /Parent 2 0 R /MediaBox [0 0 400 400] /Resources << >> /Contents 4 0 R"
    b" /Annots [5 0 R] >>",
    stream(b""),
    b"<< /Type /Annot /Subtype /FreeText /Rect [100 100 200 140] /P 3 0 R /C [1 0 0]"
    b" /Name /Comment /Contents (no appearance string) >>",
]
buf = io.BytesIO()
buf.write(b"%PDF-1.7" + NL + b"%\xe2\xe3\xcf\xd3" + NL)
offsets = []
for n, body in enumerate(OBJECTS, start=1):
    offsets.append(buf.tell())
    buf.write(str(n).encode() + b" 0 obj" + NL + body + NL + b"endobj" + NL)
startxref = buf.tell()
buf.write(b"xref" + NL + b"0 " + str(len(OBJECTS) + 1).encode() + NL)
buf.write(b"0000000000 65535 f " + NL)
for off in offsets:
    buf.write(b"%010d 00000 n " % off + NL)
buf.write(b"trailer" + NL)
buf.write(b"<< /Size " + str(len(OBJECTS) + 1).encode() + b" /Root 1 0 R >>" + NL)
buf.write(b"startxref" + NL + str(startxref).encode() + NL + b"%%EOF" + NL)

raw = buf.getvalue()
for n, off in enumerate(offsets, start=1):
    header = str(n).encode() + b" 0 obj"
    assert raw[off:off + len(header)] == header, (n, off)
assert raw[startxref:startxref + 4] == b"xref"

out = os.path.join(os.path.dirname(os.path.abspath(__file__)), "annot-facts.pdf")
io.open(out, "wb").write(raw)
print("wrote", out, len(raw), "bytes;", len(OBJECTS), "objects, xref at", startxref)
