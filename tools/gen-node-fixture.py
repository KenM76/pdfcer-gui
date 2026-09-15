#!/usr/bin/env python3
"""Generate ``fixtures/polyline-nodes.pdf`` — the fixture the multi-node move needs.

===========================================================================
WHY THIS FILE EXISTS
===========================================================================

``multi_node_move_moves_every_picked_anchor`` drives the Node rung: it clicks
a shape, descends twice, picks one anchor, Shift-picks a second, drags both,
and asserts that ONE ``move_nodes`` command carried both.

Every step of that needs anchors that are:

  1. **several per subpath** — the check descends into a subpath and picks two
     of its anchors, so a subpath with one or two coincident anchors gives it
     nothing to work with;
  2. **far apart on screen** — an anchor mark is six points wide, so two
     anchors ten points apart at fit-page zoom are one mark as far as a click
     is concerned;
  3. **on a page small enough that fit-page zoom is near 1** — the operator's
     own drawings are ISO A1 sheets shown at 0.38x, where sixty points of page
     space is twenty-three pixels of screen.

``SW41177.pdf`` fails (1) and (3) at once. Driven against it, the Node rung
reports::

    canvas-anchors total=2 selected=0 unselected_drawn=2

a two-anchor line whose two published anchor marks land at the same screen x
once the descent has entered a subpath. There is no second anchor to pick, so
the check SKIPS — correctly, and uselessly.

**That is not a reason to weaken the check.** It is the fixture lesson: *a
check that drives a document this project imagined tests the shape this project
imagined.* The right answer is a fixture with the shape the feature is about —
and, separately, to keep driving the real drawing for the features whose
subject IS a real drawing (text editing, the page cache), which is what
``--doc-point`` exists for.

===========================================================================
WHAT IT PRODUCES
===========================================================================

One US-Letter page (612 x 792 pt) carrying **one path object with one
subpath**: an open six-anchor polyline whose vertices are at::

    (100, 200) (200, 320) (300, 200) (400, 320) (500, 200) (560, 300)

Six because the overlay publishes at most six anchor regions
(``canvas::overlay::PUBLISHED_ANCHORS``) and a fixture that offered more would
be testing the cap rather than the move. A hundred points between neighbours,
which at US-Letter fit-page zoom (~0.9x on a 900-pixel-tall canvas) is about
ninety screen pixels — an order of magnitude past the six-pixel mark, so a
click aimed at one anchor cannot land on another.

Zig-zag rather than a straight line, deliberately: a horizontal polyline's
anchors differ only in x, so a check that confused two of them would still
produce a plausible-looking drag. The alternating y makes a wrong pick
visible in the committed coordinates.

**One subpath, and one object on the page.** The Node rung is reached by
descending Object -> Part -> Node, and each descent is a double click at the
same point; a page with two objects near the aim point would make which one is
entered depend on paint order, which is a fact about the fixture that the
check would silently inherit.

Stroked, not filled: a fill of an open path is closed implicitly by the
renderer, which would put an anchor-free segment between the last vertex and
the first and make the drawn marks disagree with the visible shape.

===========================================================================
HOW TO RUN
===========================================================================

    python tools/gen-node-fixture.py

Writes ``fixtures/polyline-nodes.pdf``. Idempotent: same bytes every run, so
it can be regenerated without producing a diff. No dependencies beyond the
standard library — the same rule ``gen-textedit-fixtures.py`` follows, and for
the same reason: a fixture generator that needs a PDF library is a fixture
generator that stops working when that library moves.
"""

from __future__ import annotations

import pathlib

# ---------------------------------------------------------------------------
# The page and the shape
# ---------------------------------------------------------------------------

PAGE_W, PAGE_H = 612, 792

#: The polyline's vertices, in PDF user space (y up).
#:
#: Six of them — see the module docstring for why six and not more. The
#: alternating y is what makes a mis-picked anchor visible in the committed
#: coordinates rather than merely plausible.
VERTICES = [
    (100, 200),
    (200, 320),
    (300, 200),
    (400, 320),
]

#: Where the two cubic segments end.
#:
#: Not fed to :data:`VERTICES` - :func:`content_stream` spells the curves out,
#: because a cubic carries two control points that a vertex list cannot express
#: - but listed here so a reader sees the whole shape in one place. The path's
#: anchors are ``VERTICES + CURVE_ENDS``, six in all.
CURVE_ENDS = [(500, 320), (580, 320)]

#: Line width, points.
#:
#: Three, which is fat enough that the canvas hit test finds the path from a
#: click that is a point or two off the mathematical line. A hairline would
#: make the check's FIRST click — the one that selects the object at all — a
#: coin toss, and the failure would read as "the hit test is broken".
LINE_WIDTH = 3


def content_stream() -> bytes:
    """The page's content: one stroked path, part polyline and part curve.

    ``m``, then straight ``l`` segments to the first few vertices, then **two
    ``c`` (cubic Bezier) segments**, then ``S``. No ``h`` (close), because a
    closed subpath's last segment has no anchor of its own and the drawn marks
    would then disagree with the visible shape at exactly one vertex - which is
    the kind of off-by-one a driven check would report as a defect in the
    overlay.

    * **The curved tail is why this fixture earns its keep twice over.** The
    multi-node move needs anchors far apart, which the straight run gives it;
    the Bezier handle drag needs anchors whose neighbouring segment is a CURVE,
    because ``EditSession::move_handle`` refuses a straight one by name
    (``NoHandleHere``) and this shell agrees by drawing no handle there. A
    fixture of straight lines would make the handle check SKIP forever while
    reporting nothing wrong - which is the worst outcome available, because it
    reads as "not applicable" rather than as "untested".

    ``v`` and ``y`` are deliberately NOT used. They omit a control point, so a
    drag on one makes the engine re-spell the segment as ``c`` and return a
    disclosure; that path deserves its own fixture and its own check, and mixing
    it in here would make a passing run ambiguous about which path it took.
    """
    x0, y0 = VERTICES[0]
    ops = [f"{LINE_WIDTH} w", f"{x0} {y0} m"]
    for x, y in VERTICES[1:]:
        ops.append(f"{x} {y} l")
    # Two cubics off the last straight vertex. The control points are pulled
    # sixty points off the chord, so a handle mark is nowhere near an anchor
    # mark and a click aimed at one cannot land on the other.
    ops.append("440 380 480 380 500 320 c")
    ops.append("530 260 560 260 580 320 c")
    ops.append("S")
    return "\n".join(ops).encode("ascii")


# ---------------------------------------------------------------------------
# The smallest correct PDF that carries it
# ---------------------------------------------------------------------------


def build() -> bytes:
    """Assemble a five-object PDF with a correct cross-reference table.

    Written by hand rather than with a library for the reason in the module
    docstring. The structure is the minimum ISO 32000-1 §7.5 requires:

      1  ``/Catalog``   -> the page tree
      2  ``/Pages``     -> one kid
      3  ``/Page``      -> the media box and the content stream
      4  the content stream itself

    Offsets are recorded as each object is emitted, so the ``xref`` table
    cannot drift from the body — the failure mode of a hand-written PDF is
    almost always a stale offset, and computing them from the buffer's own
    length makes that unrepresentable.
    """
    content = content_stream()
    objects: list[bytes] = [
        b"<< /Type /Catalog /Pages 2 0 R >>",
        b"<< /Type /Pages /Kids [3 0 R] /Count 1 >>",
        (
            f"<< /Type /Page /Parent 2 0 R /MediaBox [0 0 {PAGE_W} {PAGE_H}] "
            f"/Contents 4 0 R /Resources << >> >>"
        ).encode("ascii"),
        b"<< /Length " + str(len(content)).encode("ascii") + b" >>\nstream\n" + content + b"\nendstream",
    ]

    out = bytearray(b"%PDF-1.7\n")
    # A binary comment, so every tool treats the file as binary rather than
    # trying to normalise its line endings — which would invalidate the xref
    # offsets below the moment the file crossed a platform.
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
    target = root / "fixtures" / "polyline-nodes.pdf"
    target.parent.mkdir(parents=True, exist_ok=True)
    target.write_bytes(build())
    print(f"wrote {target} ({target.stat().st_size} bytes)")
    print(f"  one path, one subpath, {len(VERTICES) + len(CURVE_ENDS)} anchors")
    print(f"  the last {len(CURVE_ENDS)} arrive on CURVES, so they carry Bezier handles")
    print(f"  aim --doc-point at 0,{VERTICES[1][0]},{VERTICES[1][1]}")


if __name__ == "__main__":
    main()
