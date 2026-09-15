#!/usr/bin/env python3
"""Generate ``fixtures/form-xobject.pdf`` — a drawing wrapped in a form.

===========================================================================
WHY THIS FILE EXISTS
===========================================================================

``a_click_selects_the_whole_drawing_and_a_double_click_goes_inside`` drives
``OPERATOR_REQUESTS.md`` O70: with Smart-select on, a click must select the
**container** and a double-click must go inside it. Both halves need a page
whose visible content is painted from inside a **form XObject** — the only
container a PDF page has.

THE REAL DRAWINGS CANNOT SERVE, AND THE MEASUREMENT IS WHY
---------------------------------------------------------------------------

Two candidates were driven first, and both failed for the same reason in two
different disguises.

``SW41177.pdf`` — the sweep's own fixture — has **no form XObject at all**
(``/Subtype /Form`` appears zero times in 1.8 MB). So every click selects a
page object correctly and there is nothing to enter. A check run against it
would SKIP forever, which is honest and useless.

``ncored-benchmark-cad-drawing.pdf`` has exactly one form, and it is the
interesting shape: **129,758 page objects over 10,256 leaves inside one
form**, the form's own box spanning the whole sheet and beyond
(``-114,-4`` to ``2270,1680``). Three clicks were driven at three points
chosen by asking ``hit_test_point_deep`` directly, and all three selected a
PAGE object — 119703, 1528, 64850 — never a leaf:

  * the probe asked with a **3 pt** tolerance, in page space;
  * the shell asks with ``SELECT_SCREEN_TOLERANCE_PX`` converted at the
    CURRENT ZOOM, and that sheet opens at fit — about 0.39x — so six screen
    pixels is roughly **fifteen points** of page.

At that radius the big page objects win everywhere, because they are
everywhere. The leaves whose centres do survive a 3 pt probe are 4 x 6 pt
glyph strokes: one screen pixel and a half. So the feature is perfectly
reachable by an operator who has zoomed in, and unreachable by a harness
aiming at a page opened to fit — which makes the DOCUMENT the wrong
instrument, not the feature wrong.

That is the fixture lesson again: *a check that drives a document this project
imagined tests the shape this project imagined.* The answer is a fixture with
the shape the feature is about, and
to keep driving the real drawings for the features whose subject IS a real
drawing.

===========================================================================
WHAT IT PRODUCES
===========================================================================

One 400 x 300 pt page. Its whole content stream is::

    q 1 0 0 1 40 40 cm /Fm0 Do Q

so **every visible mark on the page is inside the form** and there is no page
object anywhere for a click to prefer. The form's ``/BBox`` is
``[0 0 320 220]``, and it paints three fat strokes:

    a horizontal bar   (20,110) -> (300,110)   12 pt wide
    a vertical bar     (160, 20) -> (160,200)  12 pt wide
    a diagonal         (40, 40) -> (280,180)   10 pt wide

Fat on purpose. The page is small, so fit-page zoom is around 2x on this
harness's window, and a 12 pt stroke is then about 24 screen pixels — an
order of magnitude past the six-pixel pick tolerance, so a click aimed at the
middle of a bar cannot be ambiguous the way a 4 pt glyph stroke was.

Three strokes rather than one, and crossing rather than parallel:

  * a **click** must resolve to the container, which is one answer however
    many leaves there are — but a fixture with ONE leaf could satisfy that by
    accident on a build that simply reported the only thing it found;
  * a **double-click** must then select a specific leaf, and with three
    crossing strokes a build that entered the container and picked the wrong
    one is visible as a different ``first=leaf:N`` rather than as a pass.

No text, no images, no annotations. Everything on this page exists to
answer one question, and anything else on it would be a second reason a click
could resolve somewhere unexpected.

===========================================================================
HOW
===========================================================================

Hand-assembled COS, like every other generator in this directory: no
dependency, byte offsets computed as the objects are appended, one
cross-reference table written at the end. It is a plain PDF 1.7 file with no
compression, so a reader can open it in a text editor and see the whole thing
— which is the property that matters for a fixture whose job is to be
unsurprising.
"""

from pathlib import Path

# --------------------------------------------------------------------------
# The page, the form, and the marks inside it.
# --------------------------------------------------------------------------

PAGE_W, PAGE_H = 400, 300
FORM_W, FORM_H = 320, 220
PLACE_X, PLACE_Y = 40, 40

# The form's own content: three fat strokes in its own coordinate space.
FORM_STREAM = b"""\
0 0 0 RG
12 w
20 110 m 300 110 l S
160 20 m 160 200 l S
10 w
40 40 m 280 180 l S
"""

# The page's whole content: place the form once, and nothing else.
PAGE_STREAM = b"q 1 0 0 1 %d %d cm /Fm0 Do Q\n" % (PLACE_X, PLACE_Y)


def build() -> bytes:
    """Assemble the file, returning its bytes."""
    objects: list[bytes] = []

    def add(body: bytes) -> int:
        """Append an object and return its 1-based number."""
        objects.append(body)
        return len(objects)

    catalog = add(b"<< /Type /Catalog /Pages 2 0 R >>")
    assert catalog == 1
    pages = add(b"<< /Type /Pages /Kids [3 0 R] /Count 1 >>")
    assert pages == 2

    page = add(
        b"<< /Type /Page /Parent 2 0 R /MediaBox [0 0 %d %d] "
        b"/Resources << /XObject << /Fm0 5 0 R >> >> /Contents 4 0 R >>"
        % (PAGE_W, PAGE_H)
    )
    assert page == 3

    add(b"<< /Length %d >>\nstream\n" % len(PAGE_STREAM) + PAGE_STREAM + b"endstream")
    add(
        b"<< /Type /XObject /Subtype /Form /FormType 1 "
        b"/BBox [0 0 %d %d] /Resources << >> /Length %d >>\nstream\n"
        % (FORM_W, FORM_H, len(FORM_STREAM))
        + FORM_STREAM
        + b"endstream"
    )

    out = bytearray(b"%PDF-1.7\n%\xe2\xe3\xcf\xd3\n")
    offsets = [0]
    for number, body in enumerate(objects, start=1):
        offsets.append(len(out))
        out += b"%d 0 obj\n" % number + body + b"\nendobj\n"

    startxref = len(out)
    out += b"xref\n0 %d\n" % (len(objects) + 1)
    out += b"0000000000 65535 f \n"
    for offset in offsets[1:]:
        out += b"%010d 00000 n \n" % offset
    out += b"trailer\n<< /Size %d /Root 1 0 R >>\nstartxref\n%d\n%%%%EOF\n" % (
        len(objects) + 1,
        startxref,
    )
    return bytes(out)


def main() -> None:
    target = Path(__file__).resolve().parent.parent / "fixtures" / "form-xobject.pdf"
    target.write_bytes(build())
    print(f"wrote {target} ({target.stat().st_size} bytes)")


if __name__ == "__main__":
    main()
