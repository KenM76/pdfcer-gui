#!/usr/bin/env python3
"""Generate ``fixtures/paragraph.pdf`` — a paragraph that reflow can change.

===========================================================================
WHY THIS FILE EXISTS
===========================================================================

``edit.reflow_block`` re-wraps a paragraph so its lines fill their box again.
Proving it works means driving the running binary, and driving it needs a
document with **a paragraph the recogniser sees as one block and whose lines
are visibly under-filled** — so that a correct reflow produces *fewer lines*
and a broken one does not.

**No fixture in this repository has one.** The inventory, as of 2026-08-28:

  ``a1-titleblock.pdf``      isolated cells — a CAD title block, which is
                             precisely the shape that has NO paragraph
  ``tail-alignment.pdf``     three blocks built to exhibit alignment and
                             rotation, every line placed by measurement to a
                             flush right edge — a reflow of a block whose
                             lines are already flush has nothing to do
  ``rotated-text.pdf``       one rotated line
  ``four-pages.pdf``         page-count work
  ``polyline-nodes.pdf``,
  ``reclaimable.pdf``,
  ``synthetic-image-only.pdf``,
  ``embedded-font.pdf``      not text-shaped at all

★★★ **A check driven against a fixture with no reflowable paragraph would
report the feature broken about a build whose reflow works perfectly**, and
this project has already made that exact mistake twice — a harness aiming at
the wrong point, and a tool left armed so a click drew instead of selecting.
The fixture is part of the oracle, not scenery around it.

===========================================================================
WHAT IT PRODUCES, AND WHY EACH DECISION IS THE WAY IT IS
===========================================================================

One US-Letter page (612 x 792 pt) carrying **one left-aligned block of six
lines**, Helvetica 12, at ``LEFT`` = 72 pt, first baseline at ``TOP`` = 700
pt, stepping down by ``LEADING`` = 16 pt.

**The lines are deliberately ragged and SHORT.** The longest reaches about
310 pt of advance; the shortest about half that. The engine's default wrap
width is *the block box width*, i.e. the widest line — so a correct reflow
packs the short lines up into the long ones and the block loses lines. That
difference is the check's oracle:

    lines_before=6  lines_after=4     ← a reflow happened
    lines_before=6  lines_after=6     ← nothing moved

★★ **Why raggedness is not enough on its own, and the leading matters.**
``BlockRecognitionOptions::default()`` groups lines into a block by baseline
gap and by left-edge agreement. A leading far from the font size splits the
block into single-line blocks, and a single-line block reflows to itself —
which looks exactly like a broken reflow. 16 pt against a 12 pt face is a
1.33 ratio, ordinary body-text leading, comfortably inside the default.

★★ **Every line starts at the SAME x.** A ragged *left* edge is what makes
``infer_alignment`` answer ``Right``, and a right-aligned block reflows
differently. This fixture is about the ordinary case, so the lefts are flush
to the point and the rights are ragged — which is what left-aligned prose is.

★ **One ``BT`` / ``ET`` for the whole block**, with an absolute ``Tm`` per
line. Same reason ``gen-textedit-fixtures.py`` gives: the engine's walk acts
on ``Tm`` records, so every line this fixture wants moved has to carry one.

★ **The content stream is uncompressed**, so a failure can be diagnosed by
reading the file rather than by writing a decoder first. The page is small
enough that the cost is a few hundred bytes.

★ **Every line's text is distinct and its words are English prose.** Prose
because the recogniser's word-gap heuristics are tuned for it and a fixture
of ``AAA BBB`` would exercise a shape no document has; distinct because a
failure that names the line it found is worth more than one that cannot.

===========================================================================
USAGE
===========================================================================

    python tools/gen-reflow-fixture.py

Writes ``fixtures/paragraph.pdf`` relative to the repository root,
overwriting it. Deterministic: same bytes every run, so re-running it never
shows up as a diff. ``.gitattributes``'s ``*.pdf binary`` rule already covers
it — ``core.autocrlf`` is true on this machine and a normalised ``\\r\\n``
would corrupt the cross-reference table's absolute byte offsets.

It prints the geometry a check needs — the block's rectangle in page points
— rather than leaving a check to hard-code numbers this script computes. A
check that hard-codes a coordinate is a check that silently aims at blank
paper the day a width changes.
"""

import os

# --------------------------------------------------------------------------
# Helvetica advance widths, 1000 units per em.
#
# The Adobe Core-14 AFM metrics, which is what `pdfcer-core` falls back to for
# a non-embedded standard face — so the x positions computed here and the
# glyph boxes the engine extracts agree. Only the characters this fixture
# shows are listed: a missing one raises `KeyError` at generation time rather
# than silently mis-measuring a line.
# --------------------------------------------------------------------------
W = {
    " ": 278, ",": 278, ".": 278, "-": 333,
    "a": 556, "b": 556, "c": 500, "d": 556, "e": 556, "f": 278, "g": 556,
    "h": 556, "i": 222, "j": 222, "k": 500, "l": 222, "m": 833, "n": 556,
    "o": 556, "p": 556, "q": 556, "r": 333, "s": 500, "t": 278, "u": 556,
    "v": 500, "w": 722, "x": 500, "y": 500, "z": 500,
    "A": 667, "B": 667, "C": 722, "D": 722, "E": 667, "F": 611, "G": 778,
    "H": 722, "I": 278, "J": 500, "K": 722, "L": 611, "M": 833, "N": 722,
    "O": 778, "P": 667, "Q": 778, "R": 722, "S": 667, "T": 611, "U": 722,
    "V": 667, "W": 944, "X": 667, "Y": 667, "Z": 611,
}

SIZE = 12.0
#: Every line's left edge. Flush to the point — see the header on why.
LEFT = 72.0
#: The first line's baseline.
TOP = 700.0
#: The baseline step. 1.33 x the size: ordinary body-text leading, and well
#: inside what `BlockRecognitionOptions::default()` will group.
LEADING = 16.0

#: The six lines, deliberately ragged and short. Line 1 is the longest and
#: therefore sets the block box width the reflow wraps to.
LINES = [
    "The drawing office keeps every revision of a sheet",
    "in one file, and the notes beside",
    "the title block are edited far more often",
    "than the geometry is. A note that has been",
    "retyped twice no longer fills",
    "its box.",
]


def advance(text: str) -> float:
    """The §9.4.4 advance of ``text`` at :data:`SIZE`, in points.

    No ``Tc`` / ``Tw`` / ``Tz`` terms because the fixture sets none: the
    formula reduces to ``sum(w0) * Tfs``, so the extraction's arithmetic and
    this function's cannot disagree.
    """
    return sum(W[c] for c in text) * SIZE / 1000.0


def show(text: str) -> str:
    """One ``(literal) Tj``, with §7.3.4.2's three escapes applied."""
    esc = text.replace("\\", r"\\").replace("(", r"\(").replace(")", r"\)")
    return f"({esc}) Tj\n"


def build_content() -> bytes:
    """The page's whole content stream, uncompressed.

    One ``BT`` / ``ET`` holding six absolutely-positioned lines. The font is
    selected once, before the first ``Tm``, because a ``Tf`` between lines
    would give the recogniser a reason to split the block and the split is
    the failure this fixture exists to avoid.
    """
    out = ["BT\n", f"/F1 {SIZE:g} Tf\n"]
    for i, text in enumerate(LINES):
        y = TOP - i * LEADING
        out.append(f"1 0 0 1 {LEFT:.2f} {y:.2f} Tm\n")
        out.append(show(text))
    out.append("ET\n")
    return "".join(out).encode("ascii")


def build_pdf() -> bytes:
    """The whole file: five objects, a classic cross-reference table, EOF.

    No object streams and no compression — see the header. The xref offsets
    are computed from the bytes actually written rather than predicted, so a
    change to any object body cannot desynchronise the table.
    """
    content = build_content()
    objects = [
        b"<< /Type /Catalog /Pages 2 0 R >>",
        b"<< /Type /Pages /Kids [3 0 R] /Count 1 >>",
        b"<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] "
        b"/Resources << /Font << /F1 5 0 R >> >> /Contents 4 0 R >>",
        b"<< /Length " + str(len(content)).encode() + b" >>\nstream\n"
        + content + b"endstream",
        b"<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica "
        b"/Encoding /WinAnsiEncoding >>",
    ]

    out = bytearray(b"%PDF-1.7\n%\xe2\xe3\xcf\xd3\n")
    offsets = []
    for i, body in enumerate(objects, start=1):
        offsets.append(len(out))
        out += f"{i} 0 obj\n".encode() + body + b"\nendobj\n"

    xref_at = len(out)
    out += f"xref\n0 {len(objects) + 1}\n".encode()
    out += b"0000000000 65535 f \n"
    for off in offsets:
        out += f"{off:010d} 00000 n \n".encode()
    out += (
        f"trailer\n<< /Size {len(objects) + 1} /Root 1 0 R >>\n"
        f"startxref\n{xref_at}\n%%EOF\n"
    ).encode()
    return bytes(out)


def main() -> None:
    root = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
    target = os.path.join(root, "fixtures", "paragraph.pdf")
    data = build_pdf()
    with open(target, "wb") as fh:
        fh.write(data)

    print(f"wrote {target} ({len(data)} bytes)")
    widest = max(advance(t) for t in LINES)
    total = sum(advance(t) for t in LINES)
    print(f"  block left  x = {LEFT:.2f}")
    print(f"  block right x = {LEFT + widest:.2f}  (wrap width {widest:.2f} pt)")
    print(f"  baselines     = {TOP:.2f} down to {TOP - (len(LINES) - 1) * LEADING:.2f}")
    print(f"  total advance = {total:.2f} pt over {len(LINES)} lines")
    # ★ The number a check should expect, printed rather than asserted here:
    # this script does not know the engine's word-breaking, and a prediction
    # baked into a fixture generator is a second implementation of the thing
    # under test. What it CAN say is the floor — the block cannot need more
    # lines than it has, and packing to the widest line cannot need more than
    # ceil(total / widest).
    print(f"  a correct reflow needs at least {int(total // widest) + 1} line(s)")
    for i, text in enumerate(LINES):
        print(f"    line {i}: y={TOP - i * LEADING:7.2f}  w={advance(text):6.2f}  {text!r}")


if __name__ == "__main__":
    main()
