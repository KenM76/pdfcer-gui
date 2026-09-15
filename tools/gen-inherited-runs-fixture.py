#!/usr/bin/env python3
"""Generate `fixtures/inherited-runs.pdf` -- the document that can REFUSE a
line move, which `OPERATOR_REQUESTS.md` O188's move half cannot be verified
without.

# Why this fixture exists, and why `paragraph.pdf` could not be used

`pdfcer-core` shipped `move_text_run` / `move_text_run_in_form` on 2026-09-14
(`G017`).  They do not always succeed, and the two ways they refuse are the
thing this document exists to produce:

| refusal | when |
|---|---|
| `TextRunHasNoPositionOfItsOwn` | THIS line's origin is inherited from where the previous line finished, so there is no operand to rewrite |
| `MoveWouldMoveNextRun`         | the NEXT line's origin is inherited from where THIS one finishes, so moving this one would carry that one along |

Both come from ISO 32000-1 9.4.2: a show operator that is not preceded by a
text-positioning operator (`Td`, `TD`, `Tm`, `T*`, `'`, `"`) starts wherever
the previous show operator's advance left the pen.

`fixtures/paragraph.pdf` -- the pin every other line-of-text check uses -- has
**six** show operators and a `Tm` in front of every single one of them.  Its
runs are all `RunPositioning::Explicit`, so `text_run_move_refusal` answers
`None` six times out of six and NEITHER refusal can be reached on it.  That is
the right shape for asserting that a line moves, and it is structurally
incapable of asserting that a line refuses.

*** A check written against `paragraph.pdf` alone would therefore pass on a
build that had deleted the pre-check entirely, for ever.  It would also pass on
a build that raised the WRONG one of the two sentences, since it would never
raise either.  A check that cannot fail on the dangerous build is not a check,
and that is the whole justification for a second fixture rather than a second
aim point on the first.

# The shape, and why it is exactly three runs

One `BT`...`ET` block, one font, three show operators, arranged so that a
single document yields all three of the engine's answers:

| run | content stream | `text_run_move_refusal` | what a drag on it must do |
|---|---|---|---|
| 0 | `1 0 0 1 72 700 Tm (Alpha) Tj` | `MoveWouldMoveNextRun` | refuse, saying the NEXT line would be dragged |
| 1 | `(Beta follows Alpha) Tj`      | `TextRunHasNoPositionOfItsOwn` | refuse, saying THIS line has no position |
| 2 | `1 0 0 1 72 660 Tm (Gamma stands alone) Tj` | `None` | MOVE |

Run 1 is the only one with no positioning operator in front of it.  That single
omission is what makes run 1 unmovable and run 0 unmovable-for-the-other-reason,
which is why three runs is the minimum: two would give one refusal and no
control, and a fourth would add nothing that is not already distinguishable.

** Run 2 is the CONTROL and is not decoration.  Without it, a build that
refused every line move -- the state of the program before 2026-09-14 -- would
satisfy every assertion made against runs 0 and 1.  The fixture has to be able
to tell "refuses for the right reason" from "refuses always", and one document
that does both is the only way to do that without trusting two separate runs of
the harness to have driven the same build.

*** Run 2 is deliberately placed at y=660, forty points BELOW run 0's baseline,
rather than continuing the inherited chain.  A `Tm` immediately after run 1 is
what makes run 1's successor explicit, which is what keeps run 1's refusal
`TextRunHasNoPositionOfItsOwn` and NOT `MoveWouldMoveNextRun` as well.  The
engine checks its own reasons in that order, so the two would be
indistinguishable from outside if run 2 inherited too -- and the check would be
asserting a sentence it had not actually isolated.

# ** Why the content stream is UNCOMPRESSED

So that `python -c "print(open(...,'rb').read())"` -- or a human with an
editor -- can confirm by eye that run 1 has no positioning operator in front of
it.  The single fact this whole fixture encodes is an ABSENCE, and an absence
inside a Flate stream is not reviewable.  Byte-for-byte reviewability beats
half a kilobyte.

# ** Why Helvetica and not an embedded font

Because the refusal has nothing to do with glyphs.  The engine's guard reads
the operator sequence and never touches the font program, so an embedded font
would add a failure mode (a broken `/Widths`, a bad `FontFile`) that could make
this fixture fail for a reason that has no bearing on its subject.  Base-14
Helvetica is resolved by every viewer and by `pdfcer-render`'s own fallback.

* The advance of run 0 therefore depends on the viewer's Helvetica metrics,
which is fine: nothing here asserts WHERE run 1 lands, only that it inherits.
The measured spans a driven check aims at are recorded in
`fixtures/inherited-runs.PROVENANCE.md` and are re-measured, never assumed.

# Regenerating

    python tools/gen-inherited-runs-fixture.py

Deterministic: same bytes every time, no timestamps, no ids.  Committed to the
repository, so a missing file is a broken checkout and a driven check must
report its absence as a FAILURE rather than a skip.
"""

from __future__ import annotations

import pathlib

FIXTURES = pathlib.Path(__file__).resolve().parent.parent / "fixtures"

# The three lines, spelled out rather than generated, because the ONE fact this
# fixture encodes is the absence of a positioning operator on the middle line
# and a loop would hide it behind a conditional.
CONTENT = """BT
/F1 12 Tf
1 0 0 1 72 700 Tm
(Alpha) Tj
(Beta follows Alpha) Tj
1 0 0 1 72 660 Tm
(Gamma stands alone) Tj
ET
"""


def build(content: str) -> bytes:
    """A one-page PDF with `content` as its uncompressed content stream.

    Lifted verbatim from `tools/gen-per-glyph-fixtures.py`, which carries the
    argument for every choice in it: five objects, a classic cross-reference
    table rather than a stream, `/MediaBox [0 0 612 792]`, and one base-14
    font resource named `/F1`.

    * Kept as a copy rather than imported.  These generators are run by hand,
      years apart, by whoever needs a document with one specific shape; a
      shared module would make each of them unreadable on its own and would
      couple two fixtures that have nothing to do with each other.  The cost of
      the duplication is twenty lines that have not changed since they were
      written.
    """
    stream = content.encode("latin-1")
    objects = [
        b"<< /Type /Catalog /Pages 2 0 R >>",
        b"<< /Type /Pages /Kids [3 0 R] /Count 1 >>",
        b"<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] "
        b"/Resources << /Font << /F1 5 0 R >> >> /Contents 4 0 R >>",
        b"<< /Length " + str(len(stream)).encode() + b" >>\nstream\n" + stream + b"\nendstream",
        b"<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>",
    ]
    out = bytearray(b"%PDF-1.7\n")
    offsets = [0]
    for n, body in enumerate(objects, start=1):
        offsets.append(len(out))
        out += str(n).encode() + b" 0 obj\n" + body + b"\nendobj\n"
    xref = len(out)
    out += b"xref\n0 " + str(len(objects) + 1).encode() + b"\n"
    out += b"0000000000 65535 f \n"
    for off in offsets[1:]:
        out += f"{off:010d} 00000 n \n".encode()
    out += (
        b"trailer\n<< /Size " + str(len(objects) + 1).encode() + b" /Root 1 0 R >>\n"
        b"startxref\n" + str(xref).encode() + b"\n%%EOF\n"
    )
    return bytes(out)


def main() -> None:
    path = FIXTURES / "inherited-runs.pdf"
    path.write_bytes(build(CONTENT))
    print(f"inherited-runs.pdf: {path.stat().st_size} bytes")


if __name__ == "__main__":
    main()
