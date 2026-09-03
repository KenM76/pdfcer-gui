#!/usr/bin/env python3
"""Generate ``fixtures/tail-alignment.pdf`` — the fixture ``DEFECTS.md`` D4b needs.

===========================================================================
WHY THIS FILE EXISTS
===========================================================================

``DEFECTS.md`` D4's closing section, *"Why the tests do not catch any of
this"*, is an inventory of what the project's fixtures do not contain:

    No fixture has a paragraph split across many runs, mixed sizes or fonts
    in a block, ROTATED TEXT, or words separated by positioning rather than
    space glyphs. Every condition that fails in the field is absent by
    construction.

Two of D4b's cases are *wrong on commit*, and both of them are invisible on
every fixture this repository holds:

  1. a **right-aligned** block, whose tail must not move when a line is
     edited;
  2. **rotated** text, whose tail must not be displaced along user-space x
     because its baseline does not run that way.

A check that edited left-aligned upright text would pass against the broken
build — which is the whole point of `HANDOFF.md` §2's grid lesson — so the
fixture has to contain the two shapes the defect is about, and nothing else
in this repository does.

===========================================================================
WHAT IT PRODUCES, AND WHY EACH DECISION IS THE WAY IT IS
===========================================================================

One US-Letter page (612 x 792 pt) carrying three text objects:

  A. **A right-aligned three-line block**, Helvetica 12, whose lines' right
     edges are flush at ``RIGHT_EDGE`` = 500 pt and whose left edges are
     ragged by tens of points. That is what makes
     ``ReflowEngine::infer_alignment`` answer ``Right`` with source
     ``Detected``: it measures the spread of the lines' left edges against
     the spread of their right edges, with a tolerance of
     ``max(2.0, 0.5 x size)`` = 6 pt, and reports ``Right`` when the rights
     are flush and the lefts are not.

     **All three lines live inside ONE ``BT`` / ``ET``**, and that is
     load-bearing rather than tidy. The engine's reflow branch walks every
     record after the anchor until it meets a boundary and adds the advance
     delta to each absolute ``Tm`` it passes. So an edit to line 1 shifts
     lines 2 AND 3 under ``Reflow`` and neither under ``Pin`` — which turns
     the defect into a byte-level yes/no about a string that is either still
     in the file verbatim or is not.

  B. **A rotated line**, 90 degrees anticlockwise, in its own ``BT`` / ``ET``
     with an anchor run and a follower ``Tm``. Its text matrix is
     ``[0 1 -1 0 e f]``, which is what ``disposition::is_upright`` refuses.

  C. **A left-aligned upright control block**, two lines, one ``BT`` / ``ET``.
     It is here so the check can prove the fix is *selective*: a build that
     pinned everything would satisfy A and B and is not the fix. Editing this
     one MUST still reflow its follower.

**The content stream is written uncompressed.** Normally a fixture would be
Flate-compressed to keep the repository small; here the whole oracle is a
byte scan for a ``Tm`` operand triple in the file the application wrote, and
a compressed stream would make that scan answer "absent" for a correct build
and a broken one alike — a false pass in the direction that matters. The page
is small enough that the cost is a few hundred bytes.

**Every word is unique across the page.** ``EditRequest::find`` locates its
anchor by matching text within one show operator, and although the shell pins
the operator by provenance, a fixture in which two runs read the same is a
fixture where a *failure* is ambiguous. Distinct strings mean a byte scan can
name which run it found.

===========================================================================
USAGE
===========================================================================

    python tools/gen-textedit-fixtures.py

Writes ``fixtures/tail-alignment.pdf`` relative to the repository root,
overwriting it. Deterministic: same bytes every run, so re-running it never
shows up as a diff.

**``.gitattributes`` already covers this.** ``core.autocrlf`` is true
globally on this machine and a PDF's cross-reference table stores absolute
byte offsets, so a normalized ``\\r\\n`` would corrupt the file at ``git add``
time. ``HANDOFF.md`` §10 records why that file predates the first commit; the
``*.pdf binary`` rule in it covers this fixture with no change.
"""

import os

# --------------------------------------------------------------------------
# Helvetica advance widths, 1000 units per em, for the characters used below.
#
# Taken from the Adobe Core-14 AFM metrics, which is the same source
# `pdfcer-core` falls back to for a non-embedded standard face — so the x
# positions computed here and the glyph boxes the engine extracts agree. Only
# the characters this fixture actually shows are listed: a fuller table would
# be data nothing reads, and a missing character raises `KeyError` at
# generation time rather than silently mis-placing a line.
# --------------------------------------------------------------------------
W = {
    " ": 278, "0": 556, "1": 556, "2": 556, "3": 556, "4": 556, "5": 556,
    "6": 556, "7": 556, "8": 556, "9": 556, "-": 333, ".": 278, ":": 278,
    "A": 667, "B": 667, "C": 722, "D": 722, "E": 667, "F": 611, "G": 778,
    "H": 722, "I": 278, "J": 500, "K": 722, "L": 611, "M": 833, "N": 722,
    "O": 778, "P": 667, "Q": 778, "R": 722, "S": 667, "T": 611, "U": 722,
    "V": 667, "W": 944, "X": 667, "Y": 667, "Z": 611,
}

SIZE = 12.0
#: Where every line of block A ends. Flush to within far less than the
#: engine's 6 pt tolerance, because they are placed by measurement.
RIGHT_EDGE = 500.0


def advance(text: str) -> float:
    """The §9.4.4 advance of ``text`` at :data:`SIZE`, in points.

    No ``Tc`` / ``Tw`` / ``Tz`` terms because the fixture sets none: the
    formula reduces to ``sum(w0) * Tfs``, and every operator this file emits
    leaves the text state at its Table 105 defaults so the extraction's
    arithmetic and this function's cannot disagree.
    """
    return sum(W[c] for c in text) * SIZE / 1000.0


def show(text: str) -> str:
    """One ``(literal) Tj``, with §7.3.4.2's three escapes applied."""
    esc = text.replace("\\", r"\\").replace("(", r"\(").replace(")", r"\)")
    return f"({esc}) Tj\n"


def right_aligned_line(text: str, y: float) -> str:
    """A line of block A: an absolute ``Tm`` placing ``text`` to end at
    :data:`RIGHT_EDGE`, then the show operator.

    The ``Tm`` is absolute rather than a relative ``Td`` deliberately. The
    engine's follower walk only repositions ``Rec::Tm`` records — an
    advance-relative follower moves for free and would prove nothing about
    which disposition ran — so every line this fixture wants to watch has to
    carry one.
    """
    x = RIGHT_EDGE - advance(text)
    return f"1 0 0 1 {x:.2f} {y:.2f} Tm\n" + show(text)


def build_content() -> bytes:
    """The page's whole content stream, uncompressed.

    Three text objects, described in the module docstring. The ``/F1 12 Tf``
    sits inside each ``BT`` so no object depends on another having run.
    """
    out = []

    # --- A. right-aligned block, three lines, ONE BT/ET -------------------
    #
    # The left edges land at 500-129.4, 500-96.7 and 500-166.7 — a spread of
    # 70 pt against a 6 pt tolerance, which is what makes the block
    # unambiguously right-aligned rather than "flush both margins", the case
    # `infer_alignment` reports as `AmbiguousDefault`.
    out.append("BT\n/F1 12 Tf\n")
    out.append(right_aligned_line("REVISION B", 700.0))
    out.append(right_aligned_line("SHEET 2", 684.0))
    out.append(right_aligned_line("SCALE 1:5 NTS", 668.0))
    out.append("ET\n")

    # --- B. rotated line: anchor + follower, ONE BT/ET --------------------
    #
    # [0 1 -1 0 e f] is a quarter turn anticlockwise: the baseline runs UP
    # the page. Under `Reflow` the engine would add the advance delta to `e`
    # — sliding the follower across the sheet — and under `Pin` it writes no
    # `Tm` at all.
    out.append("BT\n/F1 12 Tf\n")
    out.append("0 1 -1 0 90.00 300.00 Tm\n")
    out.append(show("TITLE VERTICAL"))
    out.append("0 1 -1 0 90.00 420.00 Tm\n")
    out.append(show("PINNED TAIL"))
    out.append("ET\n")

    # --- C. left-aligned upright control, two lines, ONE BT/ET ------------
    #
    # The selectivity control. A build that pinned unconditionally would pass
    # every assertion about A and B; only this one tells the fix from a
    # blanket `Pin`.
    out.append("BT\n/F1 12 Tf\n")
    out.append("1 0 0 1 72.00 200.00 Tm\n")
    out.append(show("PLAIN LEFT ONE"))
    out.append("1 0 0 1 72.00 184.00 Tm\n")
    out.append(show("MOVING FOLLOWER"))
    out.append("ET\n")

    # --- D. TWO RUNS ON ONE BASELINE, ONE BT/ET ---------------------------
    #
    # ★★ Added 2026-08-20, and it is the only block in this fixture that
    # reflow still moves.
    #
    # The engine's `Pass 121.1` narrowed the reflow walk: a following ``Tm``
    # continues the edited line **only if it differs in ``e`` alone** — same
    # orientation, same scale, same baseline. Before that it walked forward
    # shifting every absolute ``Tm`` until a ``Td``/``TD``/``T*`` boundary,
    # and a CAD stream positions everything with ``Tm`` and never emits
    # ``Td``, so one four-character edit on the operator's real drawing moved
    # **1,676 labels** and changed 34,059 pixels across the whole sheet.
    #
    # ★ That fix made blocks A, B and C stop being falsifiable. Every
    # follower in them sits on a DIFFERENT baseline (or a different
    # orientation), so reflow no longer reaches any of them — which is
    # exactly right, and which left this fixture with no case where the
    # engine's default moves anything. A fixture that cannot exhibit the
    # hazard cannot prove a rule that prevents it: the two falsifying
    # assertions in ``proof.rs`` went quiet, and a quiet falsifier is a test
    # that has stopped measuring.
    #
    # So: two runs at the SAME ``f``, differing in ``e`` alone. That is a
    # single visual line drawn as two show operators — one table cell beside
    # another, a title-block field beside its label — which is the
    # overwhelmingly common shape on this operator's documents and the one
    # case where "the rest of the line" genuinely is the rest of a line.
    #
    # The second run is placed past the first's advance so the two do not
    # overlap; the exact x is computed rather than guessed for the reason
    # every other coordinate here is.
    same_line_x = 72.0 + advance("CELL ONE") + 12.0
    out.append("BT\n/F1 12 Tf\n")
    out.append("1 0 0 1 72.00 140.00 Tm\n")
    out.append(show("CELL ONE"))
    out.append(f"1 0 0 1 {same_line_x:.2f} 140.00 Tm\n")
    out.append(show("CELL TWO"))
    out.append("ET\n")

    return "".join(out).encode("latin-1")


def build_pdf() -> bytes:
    """Assemble a minimal, valid, uncompressed one-page PDF.

    Five objects and a classic cross-reference **table** (not a stream):
    §7.5.4's table form is what a hand-written generator can offset-check by
    eye, and the engine reads both.
    """
    content = build_content()
    objects = [
        b"<< /Type /Catalog /Pages 2 0 R >>",
        b"<< /Type /Pages /Kids [3 0 R] /Count 1 >>",
        b"<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] "
        b"/Resources << /Font << /F1 5 0 R >> >> /Contents 4 0 R >>",
        b"<< /Length " + str(len(content)).encode() + b" >>\nstream\n" + content + b"endstream",
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
    target = os.path.join(root, "fixtures", "tail-alignment.pdf")
    data = build_pdf()
    with open(target, "wb") as fh:
        fh.write(data)

    # The numbers a check has to know, printed rather than duplicated in two
    # files: a check that hard-codes a coordinate this script computes is a
    # check that silently aims at blank paper the day a width changes.
    print(f"wrote {target} ({len(data)} bytes)")
    for label, text in (
        ("A line 1", "REVISION B"),
        ("A line 2", "SHEET 2"),
        ("A line 3", "SCALE 1:5 NTS"),
    ):
        x = RIGHT_EDGE - advance(text)
        print(f"  {label}: x={x:.2f} width={advance(text):.2f} text={text!r}")
    # ★ Block D's second run, which `proof.rs` scans for by its exact operand
    # triple. Printed rather than duplicated in two files, for the reason the
    # three above are: a check that hard-codes a coordinate this script
    # computes is a check that silently aims at blank paper the day a width
    # changes.
    print(
        f"  D follower: 1 0 0 1 {72.0 + advance('CELL ONE') + 12.0:.2f} 140.00 Tm"
        "   (the ONLY Tm in this fixture that reflow still moves)"
    )


if __name__ == "__main__":
    main()
