#!/usr/bin/env python3
"""Generate ``fixtures/hole-in-a-big-object.pdf`` — the shape of O105.

===========================================================================
WHY THIS FILE EXISTS
===========================================================================

The operator, 2026-09-03:

    "can you check our radius/diameter dimensioning tool? selecting a point
    sometimes makes a big circle, and selecting more points around a hole
    doesn't always get it to narrow down to the size of the hole."

The cause was measured rather than guessed: the tool hit-tested for a **PDF
path object** and fed *every anchor of every subpath of that object* to the
circle fit. On his own drawing (``D:/Dev/pdfTests/SW41177/SW41177.pdf``,
page 1, read with ``pdfcer object-list``) three objects carry **4,405**,
**4,972** and **6,681** anchors, the largest of them holding 1,194 subpaths
across a 550 x 500 pt region — half the sheet. One click anywhere on that
object handed the fit six thousand points scattered across the drawing, and
the best-fit circle through them is enormous.

★★★ **THE FIXTURE HAS TO CONTAIN THAT SHAPE, or the check is vacuous.** A page
carrying one tidy circle in its own object would pass on the broken build and
on the fixed one: a click would contribute that circle's own anchors, the fit
would be the circle, and the defect the operator reported could not occur.
That is the fixture lesson this project keeps re-learning — *a fixture that
defeats a default does not defeat a starting state; plant it.*

So this page holds **one path object** containing:

  * a **small circle** — the "hole" — of radius 30 pt centred at (306, 500);
  * **forty unrelated line segments** scattered across the rest of the page,
    in the SAME object, as separate subpaths.

Clicking the hole under the old build therefore contributes the hole's four
anchors *and* eighty far-flung line ends, and the fitted radius comes out
around an order of magnitude too large. Under the new build a click is one
point, three clicks on the rim give radius 30, and the check can tell the two
builds apart by a number rather than by a crash.

===========================================================================
WHY THE CIRCLE IS DRAWN AS FOUR CUBICS
===========================================================================

Because that is what every CAD exporter emits — PDF has no arc operator (ISO
32000-1 §8.5.2 lists only ``m``/``l``/``c``/``v``/``y``/``h``/``re``), so a
circle is four Beziers with the classic 0.5523 control offset. Its **anchors**
are therefore the four quadrant points, which is exactly what the snap engine
offers a click on the rim, and exactly the geometry three clicks land on.

A polygon approximation would have given the fit dozens of anchors and made
the check pass for the wrong reason: with thirty anchors on the rim, even a
sloppy pick lands on the circle.

===========================================================================
WHY THE HOLE IS AT THE PAGE CENTRE AND THE CLUTTER IS NOT
===========================================================================

So a check can aim at the rim with a coordinate that is nowhere near a
distractor. The forty segments are kept at least 120 pt from the hole's centre
— four times its radius — so a click on the rim can never snap to one of them,
and a failure therefore means the tool, not the aim.

===========================================================================
HOW TO RUN
===========================================================================

    python tools/gen-hole-in-a-big-object-fixture.py

Writes ``fixtures/hole-in-a-big-object.pdf``. Idempotent: same bytes every
run, so it can be regenerated without producing a diff. No dependencies beyond
the standard library — the rule every fixture generator here follows, because
a fixture generator that needs a PDF library stops working when that library
moves.
"""

from __future__ import annotations

import pathlib

# ---------------------------------------------------------------------------
# The page, the hole, and the clutter that hides it
# ---------------------------------------------------------------------------

PAGE_W, PAGE_H = 612, 792

#: The hole's centre and radius, PDF user space (y up).
#:
#: 30 pt is small enough to be a hole rather than a page feature and large
#: enough that its four quadrant anchors are tens of screen pixels apart at
#: fit-page zoom — the same requirement ``gen-node-fixture.py`` documents, and
#: for the same reason: an anchor mark is six points wide.
HOLE_CX, HOLE_CY, HOLE_R = 306.0, 500.0, 30.0

#: The cubic control-point offset that turns four Beziers into a circle.
#:
#: 4/3 * (sqrt(2) - 1). The maximum radial error is about 0.02 % of the radius,
#: which is three orders of magnitude below anything a click can express — so a
#: check asserting the fitted radius to a tenth of a point is asserting about
#: the tool, not about this approximation.
KAPPA = 0.5522847498307936

#: How many unrelated segments share the hole's object.
#:
#: Forty, giving eighty extra anchors. Not thousands: the point is to make the
#: broken build's answer *wrong by an order of magnitude*, which forty achieves,
#: and a fixture with six thousand anchors would be slow to decompose in every
#: run of every check that opens it.
CLUTTER_SEGMENTS = 40

#: How far every clutter endpoint is kept from the hole's centre.
#:
#: Four hole radii. Inside that distance a click on the rim could snap to a
#: clutter endpoint instead, and the check would fail for a reason that has
#: nothing to do with the tool.
CLUTTER_KEEP_OUT = HOLE_R * 4.0

#: Line width, points. Fat enough that a click a point or two off the
#: mathematical rim still finds the path.
LINE_WIDTH = 2


def clutter_points() -> list[tuple[float, float, float, float]]:
    """Forty segments spread over the page, none within :data:`CLUTTER_KEEP_OUT`.

    Generated from a fixed integer recurrence rather than ``random``, because a
    fixture generator must produce the same bytes every run — a fixture whose
    contents move is a fixture that makes every check that reads it
    irreproducible.

    The recurrence is a plain linear congruential step; its statistical quality
    is irrelevant, only its determinism and its spread.
    """
    out: list[tuple[float, float, float, float]] = []
    state = 12345
    def nxt(lo: float, hi: float) -> float:
        nonlocal state
        state = (state * 1103515245 + 12345) & 0x7FFFFFFF
        return lo + (state / 0x7FFFFFFF) * (hi - lo)

    while len(out) < CLUTTER_SEGMENTS:
        x0 = nxt(40.0, PAGE_W - 40.0)
        y0 = nxt(40.0, PAGE_H - 40.0)
        x1 = x0 + nxt(-60.0, 60.0)
        y1 = y0 + nxt(-60.0, 60.0)
        far = all(
            ((x - HOLE_CX) ** 2 + (y - HOLE_CY) ** 2) ** 0.5 > CLUTTER_KEEP_OUT
            for x, y in ((x0, y0), (x1, y1))
        )
        inside = all(20.0 < x < PAGE_W - 20.0 and 20.0 < y < PAGE_H - 20.0 for x, y in ((x0, y0), (x1, y1)))
        if far and inside:
            out.append((x0, y0, x1, y1))
    return out


def content_stream() -> bytes:
    """One stroked path object: the hole, then forty unrelated segments.

    ★ **One ``S`` at the end, not one per shape.** That is what makes this a
    single path *object* with forty-one subpaths, which is the whole subject of
    the fixture — a stream that stroked each shape separately would produce
    forty-one objects and the defect could not occur.
    """
    k = KAPPA * HOLE_R
    cx, cy, r = HOLE_CX, HOLE_CY, HOLE_R
    ops = [f"{LINE_WIDTH} w"]
    # The hole: start at the east quadrant, four cubics anticlockwise, close.
    ops.append(f"{cx + r:.4f} {cy:.4f} m")
    ops.append(f"{cx + r:.4f} {cy + k:.4f} {cx + k:.4f} {cy + r:.4f} {cx:.4f} {cy + r:.4f} c")
    ops.append(f"{cx - k:.4f} {cy + r:.4f} {cx - r:.4f} {cy + k:.4f} {cx - r:.4f} {cy:.4f} c")
    ops.append(f"{cx - r:.4f} {cy - k:.4f} {cx - k:.4f} {cy - r:.4f} {cx:.4f} {cy - r:.4f} c")
    ops.append(f"{cx + k:.4f} {cy - r:.4f} {cx + r:.4f} {cy - k:.4f} {cx + r:.4f} {cy:.4f} c")
    ops.append("h")
    # …and the clutter, in the same object.
    for x0, y0, x1, y1 in clutter_points():
        ops.append(f"{x0:.3f} {y0:.3f} m")
        ops.append(f"{x1:.3f} {y1:.3f} l")
    ops.append("S")
    return "\n".join(ops).encode("ascii")


def build() -> bytes:
    """Assemble a four-object PDF with a correct cross-reference table.

    The same hand-written minimum ``gen-node-fixture.py`` uses, and for the
    same reason. Offsets are computed from the buffer's own length as each
    object is emitted, so a stale offset — the usual failure of a hand-written
    PDF — is unrepresentable.
    """
    content = content_stream()
    objects: list[bytes] = [
        b"<< /Type /Catalog /Pages 2 0 R >>",
        b"<< /Type /Pages /Kids [3 0 R] /Count 1 >>",
        (
            f"<< /Type /Page /Parent 2 0 R /MediaBox [0 0 {PAGE_W} {PAGE_H}] "
            f"/Contents 4 0 R /Resources << >> >>"
        ).encode("ascii"),
        b"<< /Length "
        + str(len(content)).encode("ascii")
        + b" >>\nstream\n"
        + content
        + b"\nendstream",
    ]

    out = bytearray(b"%PDF-1.7\n")
    out += b"%\xe2\xe3\xcf\xd3\n"

    offsets: list[int] = []
    for n, body in enumerate(objects, start=1):
        offsets.append(len(out))
        out += f"{n} 0 obj\n".encode("ascii") + body + b"\nendobj\n"

    xref_at = len(out)
    out += f"xref\n0 {len(objects) + 1}\n".encode("ascii")
    out += b"0000000000 65535 f \n"
    for off in offsets:
        out += f"{off:010d} 00000 n \n".encode("ascii")
    out += (
        f"trailer\n<< /Size {len(objects) + 1} /Root 1 0 R >>\n"
        f"startxref\n{xref_at}\n%%EOF\n"
    ).encode("ascii")
    return bytes(out)


def main() -> None:
    root = pathlib.Path(__file__).resolve().parent.parent
    target = root / "fixtures" / "hole-in-a-big-object.pdf"
    target.parent.mkdir(parents=True, exist_ok=True)
    target.write_bytes(build())
    print(f"wrote {target} ({target.stat().st_size} bytes)")
    print(
        f"  ONE path object: a circle r={HOLE_R:.0f} at ({HOLE_CX:.0f}, {HOLE_CY:.0f}) "
        f"plus {CLUTTER_SEGMENTS} unrelated segments"
    )
    print(f"  anchors in that object: 4 on the hole + {CLUTTER_SEGMENTS * 2} elsewhere")
    print(f"  aim --doc-point at 0,{HOLE_CX:.0f},{HOLE_CY + HOLE_R:.0f} (the north quadrant)")


if __name__ == "__main__":
    main()
