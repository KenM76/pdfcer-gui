"""Author `blank-overhang.pdf` — an oversize sheet whose ink stops well short of
the paper's printable area, so the part that gets cropped is empty paper.

    python fixtures/blank-overhang.PROVENANCE.py

Deterministic: no timestamps, no randomness, no producer string, so a re-run in
a clean tree leaves `git status` clean.

Driven by `tools/ui-verify/src/checks/print_clip_claim.rs`, for
`OPERATOR_REQUESTS.md` O113: the print preview's clip hatch became ink-aware,
so on a 1:1 sheet whose overhang is empty paper nothing is hatched and the
caption says the overhang is blank — while the commit button went on counting
that same sheet as clipped, because `Job::clipped()` is geometric. Two true
sentences reading as a contradiction.

# Why no existing fixture will do

The check needs a page with TWO properties at once, and `fixtures/` had nothing
that carried the second:

1. Its page box must exceed a common printable area at Actual size, or nothing
   is cropped and there is no claim to correct. `a1-titleblock.pdf` does this.
2. Its content must stop short of the cropped band, or the ink test correctly
   reports `overhang=losing`, assertion 3 is vacuous, and the check reports
   SKIPPED. `a1-titleblock.pdf` fails this: it is a full-bleed CAD sheet whose
   border and title block run to the edge of the paper, so every band the
   printable area leaves outside carries ink.

Every other oversize fixture in the corpus is the same shape — drawn to fill
its sheet — because before O113 no surface cared what was in the margin.

So this sheet is 1,000 x 800 pt with all of its ink inside a 440 x 340 box in
the upper-left corner. The numbers come from the two printable areas a Letter
printer offers, which is what a developer machine has: 775 x 595 pt in
landscape and 595 x 775 in portrait, each inset 8 pt from the sheet. The page
is larger than both in both axes, so it crops whichever way the paper turns,
and the ink sits inside 595 x 595 — the intersection of the two — with at least
95 pt of blank paper between it and the nearest crop line either way.

The placement assumed is the one the engine chooses with no operator offset:
flush to the corner of the printable area, overhanging right and bottom. That
is asserted separately, by `print_position.rs` pressing Reset and reading
`edges=.r.b`; a centred placement would overhang on all four edges and this
geometry would not hold, which is why the placement has its own instrument
rather than being taken on trust here.

# Why the clearance is 95 pt and not 5

`dialogs::print::ink` quantises the page into 256 cells on its long side —
3.9 pt per cell here — and `InkMask::ink_extent` snaps an extent OUT to whole
cells deliberately, so ink within one cell of a crop line reads as ink in the
band. 95 pt is twenty-four cells.

# ★ The page paints NO background rectangle, and that is load-bearing

`dialogs::print::ink`'s header records the measurement: a real CAD export lays
a near-white background rectangle over the whole sheet — `(249, 249, 249)` in
the file that was measured — and under a naive "anything not pure white is ink"
test 97% of that sheet reads as ink. `INK_MAX_LEVEL` is 246 for that reason.

A fixture that painted a page-wide background would therefore be ink
everywhere, every band would report `losing`, and the result would read as a
defect in the correction rather than as a property of the fixture. The margin
here is unpainted — the `(255, 255, 255)` a blank template renders — and
nothing may be added to this page that covers it.

# What is drawn

A drawing frame with a title block in its lower-right, two dimension-style
leaders with end bars, and three labels. Enough that the inked region is
unmistakably a drawing rather than a stray mark, and enough that a capture of
the preview shows a picture in one corner of a large blank sheet, which is the
operator's case in one glance.

Structure:

    1  /Catalog   -> /Pages 2
    2  /Pages     -> [3], /Count 1
    3  /Page      /MediaBox [0 0 1000 800], /Contents 4
    4  stream     the frame, the title block, the leaders and the labels

A cross-reference table rather than an xref stream, so byte offsets are
auditable against a hex dump; the offsets are computed from the assembled body,
never typed.
"""

import io
import os

NL = chr(10)

# The sheet, and the box the ink is confined to. Both stated in SCREEN
# orientation — y down from the top of the sheet — because that is the frame
# the printable area and the overhang bands are expressed in. Converting once,
# here, is cheaper to audit than converting at every drawing call.
PAGE_W, PAGE_H = 1000, 800
L, T, R, B = 60, 60, 500, 400

# The two printable areas a Letter printer offers, inset 8 pt. The fixture's
# claim is made against both, so neither orientation can make it vacuous.
PRINTABLE = ((775, 595), (595, 775))

for pw, ph in PRINTABLE:
    # Property 1: the sheet crops on both axes, so the overhang is a corner
    # rather than a strip — which is what makes `edges` two letters.
    assert PAGE_W > pw and PAGE_H > ph, (PAGE_W, PAGE_H, pw, ph)
    # Property 2: the ink is inside, with room to spare. The clearance is
    # checked rather than containment alone, because containment by one point
    # is containment within a single mask cell and reads as ink in the band.
    assert pw - R >= 95, (pw, R)
    assert ph - B >= 95, (ph, B)

CELL_PT = PAGE_W / 256.0
assert (PRINTABLE[0][0] - R) / CELL_PT >= 20, CELL_PT


def y(top_down):
    """Screen y, measured down from the sheet top, to PDF user-space y."""
    return PAGE_H - top_down


def inside(px, py):
    """Refuse a drawing coordinate outside the ink box.

    One mistyped offset below would put a stroke in the overhang, and the check
    that consumes this file would then report a defect in the application with
    a sentence naming code that is correct. Cheaper to refuse here.
    """
    assert L <= px <= R, (px, L, R)
    assert T <= py <= B, (py, T, B)
    return px


def line(x0, y0, x1, y1):
    inside(x0, y0)
    inside(x1, y1)
    return f"{x0} {y(y0)} m {x1} {y(y1)} l S"


def box(left, top, right, bottom):
    inside(left, top)
    inside(right, bottom)
    return f"{left} {y(bottom)} {right - left} {bottom - top} re S"


def text(left, top, size, body):
    # Helvetica's average advance is near 0.6 em across mixed case, so this
    # over-estimates a lower-case string and is about right for the upper-case
    # ones. It is a guard against a label running into the margin, not a
    # metrics calculation: a real width needs the font's widths array, and
    # being wrong by a few points here cannot matter against 95 pt of blank.
    inside(left, top)
    inside(left + 0.6 * size * len(body), top - size)
    return f"BT /Helv {size} Tf {left} {y(top)} Td ({body}) Tj ET"


TITLE_W, TITLE_H = 200, 70
TITLE_L, TITLE_T = R - 8 - TITLE_W, B - 8 - TITLE_H

CONTENT = (
    # The frame: the outer line on the ink box and an inner rule inset from it,
    # the way a plotted sheet is drawn.
    "0 G 1.5 w" + NL
    + box(L, T, R, B) + NL
    + "0.5 w" + NL
    + box(L + 8, T + 8, R - 8, B - 8) + NL
    # The title block, lower-right inside the frame, with one divider.
    + "1 w" + NL
    + box(TITLE_L, TITLE_T, R - 8, B - 8) + NL
    + line(TITLE_L, TITLE_T + 24, R - 8, TITLE_T + 24) + NL
    + text(TITLE_L + 6, TITLE_T + 17, 9, "BLANK-OVERHANG TEST SHEET") + NL
    + text(TITLE_L + 6, TITLE_T + 40, 8, "SHEET 1000 x 800   SCALE 1:1") + NL
    + text(TITLE_L + 6, TITLE_T + 56, 8, "ink stops 95 pt short of the crop") + NL
    # Two leaders with a bar at each end.
    + "0.8 w" + NL
    + line(L + 40, T + 60, L + 300, T + 60) + NL
    + line(L + 40, T + 52, L + 40, T + 68) + NL
    + line(L + 300, T + 52, L + 300, T + 68) + NL
    + line(L + 40, T + 90, L + 40, T + 260) + NL
    + line(L + 32, T + 90, L + 48, T + 90) + NL
    + line(L + 32, T + 260, L + 48, T + 260) + NL
    + text(L + 130, T + 52, 10, "260") + NL
    + text(L + 52, T + 180, 10, "170") + NL
    + text(L + 40, T + 34, 12, "DRAWN INSIDE THE PRINTABLE AREA") + NL
).encode()

OBJECTS = [
    b"<< /Type /Catalog /Pages 2 0 R >>",
    b"<< /Type /Pages /Kids [3 0 R] /Count 1 >>",
    b"<< /Type /Page /Parent 2 0 R "
    + f"/MediaBox [0 0 {PAGE_W} {PAGE_H}] ".encode()
    + b"/Resources << /Font << /Helv << /Type /Font /Subtype /Type1 "
    + b"/BaseFont /Helvetica >> >> >> "
    + b"/Contents 4 0 R >>",
    b"<< /Length " + str(len(CONTENT)).encode() + b" >>" + NL.encode()
    + b"stream" + NL.encode() + CONTENT + b"endstream",
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

out = os.path.join(os.path.dirname(os.path.abspath(__file__)), "blank-overhang.pdf")
io.open(out, "wb").write(raw)
print("wrote", out, len(raw), "bytes;", len(OBJECTS), "objects, xref at", startxref)
