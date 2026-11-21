"""Author `link-named.pdf` — three pages and two named destinations, one in
each §12.3.2.3 namespace, for choosing a named destination in Add link.

    python fixtures/link-named.PROVENANCE.py

Deterministic: no timestamps, no randomness, no producer string, so a re-run in
a clean tree leaves `git status` clean.

Driven by `tools/ui-verify/src/checks/link_named.rs`. Three 400 x 400 pages;
page 1 carries one bordered link, /Rect [100 100 200 140], going to
(appendix). The names:

    (appendix)    in the catalog's /Dests dictionary -> page 3, /Fit
    (chapter-two) in the /Names /Dests name tree     -> page 2, /Fit

so a list that reads only one namespace shows one name, and a list sorted by
namespace rather than by name shows them in the other order.
"""
import io
import os

NL = b"\n"


def stream(data: bytes) -> bytes:
    return b"<< /Length " + str(len(data)).encode() + b" >>\nstream\n" + data + b"endstream"


OBJECTS = [
    # 1 catalog, 2 pages, 3-5 the pages, 6 an empty content stream,
    # 7 the old-style /Dests dictionary, 8 the name tree, 9 the link
    b"<< /Type /Catalog /Pages 2 0 R /Dests 7 0 R /Names << /Dests 8 0 R >> >>",
    b"<< /Type /Pages /Kids [3 0 R 4 0 R 5 0 R] /Count 3 >>",
    b"<< /Type /Page /Parent 2 0 R /MediaBox [0 0 400 400] /Resources << >> /Contents 6 0 R"
    b" /Annots [9 0 R] >>",
    b"<< /Type /Page /Parent 2 0 R /MediaBox [0 0 400 400] /Resources << >> /Contents 6 0 R >>",
    b"<< /Type /Page /Parent 2 0 R /MediaBox [0 0 400 400] /Resources << >> /Contents 6 0 R >>",
    stream(b""),
    b"<< /appendix [5 0 R /Fit] >>",
    b"<< /Limits [(chapter-two) (chapter-two)] /Names [(chapter-two) [4 0 R /Fit]] >>",
    b"<< /Type /Annot /Subtype /Link /Rect [100 100 200 140] /Border [0 0 1] /P 3 0 R"
    b" /Dest (appendix) >>",
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

out = os.path.join(os.path.dirname(os.path.abspath(__file__)), "link-named.pdf")
io.open(out, "wb").write(raw)
print("wrote", out, len(raw), "bytes;", len(OBJECTS), "objects, xref at", startxref)
