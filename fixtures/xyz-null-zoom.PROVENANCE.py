"""Author `xyz-null-zoom.pdf` — two links whose destinations name a point and no
magnification, differing from each other in exactly one number.

    python fixtures/xyz-null-zoom.PROVENANCE.py

Deterministic: no timestamps, no randomness, no producer string, so a re-run in
a clean tree leaves `git status` clean.

Driven by `tools/ui-verify/src/checks/point_destination.rs`, for
`OPERATOR_REQUESTS.md` O200 and `DEFECTS.md` D47:

    "…links in word documents saved as pdfs such as table of contents …
     should just jump the position on the page pointed to without changing
     the zoom. If the position jumped to is visible on the page with the
     current horizontal position of the page, the horizontal position
     shouldn't be changed."

Why not `goto-actions.pdf`, which already carries a `/XYZ 72 720 null`: its
pages are US Letter and fit the window, so there is no horizontal scroll range
and `off_x` is pinned at the same number whatever the shell decides. A check
written against it is green against a build that recentres the horizontal on
every destination — the defect the second clause is about. The pages here are
1,224 pt wide, twice Letter, so holding the horizontal still is distinguishable
from moving it.

The two links differ in exactly one property:

    link 0   rect [36 700 300 730]   ->  page 2, /XYZ 168 500 null
    link 1   rect [36 640 300 670]   ->  page 2, /XYZ 1150 500 null

Same target page, same `top`, same null zoom, same rectangle size. Only the
destination's `left` differs, which is the one input the second clause turns on,
so a difference in outcome between the two drives cannot be attributed to
anything else.

168 is the x of link 0's own rectangle centre. The harness zooms in with
Ctrl+wheel on that centre, and an anchored zoom holds the point under the
pointer fixed, so content x = 168 is on screen when the click lands without the
check having to predict a window size, a fit zoom or a notch step. The
destination is visible by construction and the clause says the horizontal must
not move.

1150 is 74 pt from the right edge. From a view anchored near x = 168 it is off
the right of the viewport at any zoom where horizontal range exists at all, so
the same clause says the horizontal must move. Without that witness, "the
horizontal did not move" is also what a shell ignoring the destination entirely
would produce.

Both destinations are on page 2 because `canvas::strip::page_scroll_offset`
brings the named page into view on the frame the page turns and centres it
horizontally while doing so. A destination solved afterwards that reads the live
scroll offset is reading that centring, not the operator.
`OpenDoc::dest_origin_x` exists for this, and a same-page fixture would never
exercise it. `top = 500` on a 792 pt page is far enough from the head of the
sheet that the page turn alone lands somewhere measurably different.

Structure:

    1  /Catalog   -> /Pages 2
    2  /Pages     -> [3, 4], /Count 2
    3  /Page 1    /MediaBox [0 0 1224 792], /Annots [6, 7], /Contents 5
    4  /Page 2    /MediaBox [0 0 1224 792], /Contents 8
    5  stream     page 1: two labelled rows under the link rectangles
    6  /Annot     /Link -> /GoTo [4 0 R /XYZ 168 500 null]
    7  /Annot     /Link -> /GoTo [4 0 R /XYZ 1150 500 null]
    8  stream     page 2: marks at both destinations and a line along the top

The destinations are direct arrays rather than names through a name tree: the
name-tree path is `goto-actions.pdf`'s business and covered there, and here it
would put a resolver step between the bytes and the behaviour under test, so a
failure could be either one. A cross-reference table rather than an xref stream,
so byte offsets are auditable against a hex dump; the offsets are computed from
the assembled body, never typed.
"""

import io
import os

NL = chr(10)

# The numbers `tools/ui-verify/src/checks/point_destination.rs` also states.
# Changing one here without changing the check breaks it in a way that reads as
# a product defect.
PAGE_W, PAGE_H = 1224, 792
NEAR_RECT = (36, 700, 300, 730)
FAR_RECT = (36, 640, 300, 670)
NEAR_LEFT = (NEAR_RECT[0] + NEAR_RECT[2]) // 2
FAR_LEFT = PAGE_W - 74
DEST_TOP = 500

assert NEAR_LEFT == 168, NEAR_LEFT
assert FAR_LEFT == 1150, FAR_LEFT
# The one property that differs, and it has to differ by most of a page or the
# far destination could be on screen after all.
assert FAR_LEFT - NEAR_LEFT > PAGE_W // 2, (NEAR_LEFT, FAR_LEFT)

PAGE1 = (
    "BT /Helv 14 Tf 44 710 Td (Near: jump to a point already across the page.) Tj ET" + NL
    + "BT /Helv 14 Tf 44 650 Td (Far: jump to a point off the right of the view.) Tj ET" + NL
    + "0.6 G 1 w " + NL
    + f"{NEAR_RECT[0]} {NEAR_RECT[1]} {NEAR_RECT[2] - NEAR_RECT[0]} {NEAR_RECT[3] - NEAR_RECT[1]} re S" + NL
    + f"{FAR_RECT[0]} {FAR_RECT[1]} {FAR_RECT[2] - FAR_RECT[0]} {FAR_RECT[3] - FAR_RECT[1]} re S" + NL
).encode()

# A line along the top so "scrolled to the head of the sheet" is visibly
# different from "scrolled to the destination", and a mark at each destination.
PAGE2 = (
    "BT /Helv 14 Tf 44 760 Td (Top of page two.) Tj ET" + NL
    + "0 G 2 w " + NL
    + f"{NEAR_LEFT - 20} {DEST_TOP} m {NEAR_LEFT + 20} {DEST_TOP} l S" + NL
    + f"{NEAR_LEFT} {DEST_TOP - 20} m {NEAR_LEFT} {DEST_TOP + 20} l S" + NL
    + f"BT /Helv 14 Tf {NEAR_LEFT + 26} {DEST_TOP - 5} Td (near destination) Tj ET" + NL
    + f"{FAR_LEFT - 20} {DEST_TOP} m {FAR_LEFT + 20} {DEST_TOP} l S" + NL
    + f"{FAR_LEFT} {DEST_TOP - 20} m {FAR_LEFT} {DEST_TOP + 20} l S" + NL
    + f"BT /Helv 14 Tf {FAR_LEFT - 150} {DEST_TOP - 5} Td (far destination) Tj ET" + NL
).encode()

FONT = (
    b"/Resources << /Font << /Helv << /Type /Font /Subtype /Type1 "
    b"/BaseFont /Helvetica >> >> >> "
)


def stream(body: bytes) -> bytes:
    return (
        b"<< /Length " + str(len(body)).encode() + b" >>" + NL.encode()
        + b"stream" + NL.encode() + body + b"endstream"
    )


def link(rect, left) -> bytes:
    """One `/Link` whose action names a point and no magnification.

    The null zoom is the subject: §12.3.2.2 Table 151 lets each of `left`, `top`
    and `zoom` be null, meaning leave this one as it is, and states the
    0-means-null equivalence for `zoom` alone. A viewer that magnifies here is
    inventing a number the file went out of its way not to give it.

    `/Border [0 0 0]` so the link draws nothing of its own: the rectangles the
    page content strokes are the only cue, and a viewer-drawn border on top of
    them would make a screenshot ambiguous about which is which.
    """
    return (
        b"<< /Type /Annot /Subtype /Link /Border [0 0 0] "
        + f"/Rect [{rect[0]} {rect[1]} {rect[2]} {rect[3]}] ".encode()
        + b"/A << /Type /Action /S /GoTo /D "
        + f"[4 0 R /XYZ {left} {DEST_TOP} null] ".encode()
        + b">> >>"
    )


OBJECTS = [
    b"<< /Type /Catalog /Pages 2 0 R >>",
    b"<< /Type /Pages /Kids [3 0 R 4 0 R] /Count 2 >>",
    b"<< /Type /Page /Parent 2 0 R "
    + f"/MediaBox [0 0 {PAGE_W} {PAGE_H}] ".encode()
    + FONT
    + b"/Annots [6 0 R 7 0 R] /Contents 5 0 R >>",
    b"<< /Type /Page /Parent 2 0 R "
    + f"/MediaBox [0 0 {PAGE_W} {PAGE_H}] ".encode()
    + FONT
    + b"/Contents 8 0 R >>",
    stream(PAGE1),
    link(NEAR_RECT, NEAR_LEFT),
    link(FAR_RECT, FAR_LEFT),
    stream(PAGE2),
]

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
    # Exactly 20 bytes per entry — §7.5.4 is strict, and a 19-byte entry breaks
    # every reader that seeks by multiplication.
    buf.write(str(off).zfill(10).encode() + b" 00000 n " + NL.encode())
buf.write(b"trailer" + NL.encode())
buf.write(b"<< /Size " + str(len(OBJECTS) + 1).encode() + b" /Root 1 0 R >>" + NL.encode())
buf.write(b"startxref" + NL.encode())
buf.write(str(startxref).encode() + NL.encode())
buf.write(b"%%EOF" + NL.encode())

raw = buf.getvalue()

# A fixture whose xref is wrong fails a test for a reason that has nothing to do
# with the test, and the message will not say so.
for n, off in enumerate(offsets, start=1):
    header = str(n).encode() + b" 0 obj"
    assert raw[off:off + len(header)] == header, (n, off, raw[off:off + 16])
assert raw[startxref:startxref + 4] == b"xref", raw[startxref:startxref + 16]
assert raw.count(b"null]") == 2, raw.count(b"null]")

out = os.path.join(os.path.dirname(os.path.abspath(__file__)), "xyz-null-zoom.pdf")
io.open(out, "wb").write(raw)
print("wrote", out, len(raw), "bytes;", len(OBJECTS), "objects, xref at", startxref)
