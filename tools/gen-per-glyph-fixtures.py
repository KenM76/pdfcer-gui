#!/usr/bin/env python3
"""Generate the two per-glyph-operator fixtures `OPERATOR_REQUESTS.md` O142 needs.

# Why these fixtures exist

The operator reported, 2026-09-05:

    "on page 2 there is a spelling mistake - clien instead of client.
     if I try to edit the edit is not accepted."

His producer writes **one glyph per show operator**: a thirty-six character
line is thirty-six `Tj`s, positioned by x-only `Td` steps. `pdfcer-core`'s
`Pass 256.0` can match a `find` across such operators when they share a font
resource, size, `Tc`/`Tw`/`Tz`, MCID and text-space row -- but its contract
carries the clause that governs everything here:

    "A pinned request never spans."

So a request carrying BOTH a `find` and a `pinned_span` is confined to the one
operator the pin names, which on his line holds a single character, and a
thirty-six character `find` can never match inside it.  The shell's fix is to
drop the pin -- and the pin is the only disambiguator `EditRequest` has, so it
may only be dropped where the text occurs **once** on the page.

That is two behaviours, and each needs its own document:

| fixture | shape | what it proves |
|---|---|---|
| `per-glyph-operators.pdf` | one per-glyph run, unique on the page | the pin comes off and the edit LANDS |
| `per-glyph-twice.pdf` | the SAME per-glyph run twice | the pin stays on and the edit is REFUSED by name |

# * Why the second one cannot be omitted

Without it the guard is untestable in the only direction that matters.  A build
that dropped the pin unconditionally would pass every assertion made against the
first fixture, for ever, while silently editing the wrong occurrence of a word on
a signed quotation -- which is the defect this whole route is shaped around.  A
check that cannot fail on the dangerous build is not a check.

# ** Why the content streams are UNCOMPRESSED

So that `grep` over the fixture answers "how many show operators hold this
line?" directly.  Every one of these files is small enough that the compression
buys nothing, and a fixture whose defining property can only be read back by
running the program it is testing is a fixture that goes wrong quietly.

# The fonts

Helvetica, a standard-14 name with no embedded program, so the file stays tiny
and no licence question arises.  The glyph advances below are eyeballed rather
than metric -- these documents are read by an editor, never printed, and a
wrong advance changes where a letter sits and nothing this tests depends on.
* The `Td` steps ARE x-only and the rows ARE shared, because those two ARE
load-bearing: they are exactly the conditions `Pass 256.0` requires before it
will span, and a fixture that got them wrong would refuse for the right reason
by accident and prove nothing.

Run:  python tools/gen-per-glyph-fixtures.py
"""

from pathlib import Path

FIXTURES = Path(__file__).resolve().parent.parent / "fixtures"

# One glyph per show operator, stepped along x by each glyph's OWN advance on one
# row -- the shape `Pass 256.0` spans and the shape his producer emits.
SIZE = 12.0

# *** THE STEPS MUST BE THE FONT'S REAL ADVANCES, AND A UNIFORM STEP IS A BUG IN
# THE FIXTURE THAT PRESENTS AS A BUG IN THE PROGRAM.
#
# This started life as a flat `ADVANCE = 7.0` for every character, which looks
# harmless and is not.  When `edit_text` spans operators it puts the replacement
# into the one holding the match's END, empties the earlier ones, and re-spaces
# the tail by the net advance -- and it computes that advance from the font's
# /Widths.  With Td steps of 7.0 against Helvetica's real widths the two
# disagree, and the corrected line lands **2.008 pt** left of where it started.
#
# That is the fixture's arithmetic, not the engine's: on the operator's own file,
# whose producer steps by the true advances, the same edit moves the line by
# 0.042 pt.  A geometry assertion written against the flat-step fixture would
# have been measuring this generator and reporting the program.
#
# Helvetica, /Widths in 1000ths of an em (ISO 32000-1 Annex D / the AFM):
HELVETICA_WIDTHS = {"A": 667, "B": 667, "C": 722, "D": 722}


def per_glyph_line(text: str, x: float, y: float) -> str:
    """One text object drawing `text` at one `Tj` per character."""
    out = ["BT", f"/F1 {SIZE:g} Tf", f"1 0 0 1 {x} {y} Tm"]
    for i, ch in enumerate(text):
        if i:
            # The advance of the PREVIOUS glyph -- which is what a producer
            # emitting one operator per glyph writes, and what the engine's
            # re-spacing arithmetic assumes.
            step = HELVETICA_WIDTHS[text[i - 1]] / 1000.0 * SIZE
            out.append(f"{step:.4f} 0 Td")
        out.append(f"({ch}) Tj")
    out.append("ET")
    return "\n".join(out)


def build(content: str) -> bytes:
    """A one-page PDF with `content` as its uncompressed content stream."""
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
    # The unique case.  One per-glyph run; nothing else on the page repeats it.
    unique = per_glyph_line("ABC", 72, 700)
    (FIXTURES / "per-glyph-operators.pdf").write_bytes(build(unique))

    # ** The ambiguous case.  The SAME string, drawn twice, in two separate text
    # objects far enough apart in y that no matcher could join them -- so the
    # page genuinely holds two candidates and the shell genuinely cannot choose.
    twice = per_glyph_line("ABC", 72, 700) + "\n" + per_glyph_line("ABC", 72, 600)
    (FIXTURES / "per-glyph-twice.pdf").write_bytes(build(twice))

    for name in ("per-glyph-operators.pdf", "per-glyph-twice.pdf"):
        p = FIXTURES / name
        print(f"{name}: {p.stat().st_size} bytes")


if __name__ == "__main__":
    main()
