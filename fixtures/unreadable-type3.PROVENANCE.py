# -*- coding: utf-8 -*-
"""Build `fixtures/unreadable-type3.pdf` — a page carrying BOTH a pattern a
redaction can find AND a font whose text no search can ever reach.

## Why this fixture exists

`OPERATOR_REQUESTS.md` has no row for this; it came out of the request-channel
archival sweep of 2026-09-11, which is worth saying because it is the kind of
defect that is only ever found by reading.

`EditSession::mark_redactions_by_pattern_styled` returns `Vec<ObjId>` where its
literal twin `search_and_mark_redactions_styled` returns `RedactionMarking` —
the same struct, plus `diagnostics`. Both call the same `author_text_matches`,
so the pattern route COMPUTES the diagnostics and then discards them one line
later at `.map(|m| m.created)`. This shell consumed the literal route's
diagnostics on 2026-08-25 the day Pass 127.1 shipped them, and on the pattern
route wrote `map_or(0, ...)` over a `None` — so a pattern redaction reported
**zero unreadable fonts** on every document, including documents full of them.

★★★ **Zero is not "unknown" on that field.** `panels::redact` prints the
warning only `if doc.last_redaction_unreadable_fonts > 0`, so the operator was
not told nothing — they were told there is nothing to tell. And because the
field lives on the document and is written unconditionally, a pattern pass run
after a literal pass **overwrote a true warning with a false zero and the
warning disappeared off the screen.** A second, more thorough redaction made
the honest disclosure vanish.

## What this file contains, and why each piece is here

    1  Catalog
    2  Pages
    3  Page      -> /Resources /Font << /Helv 6 0 R  /T3 7 0 R >>
    4  Contents  -> ONE line in Helvetica, ONE line in the Type 3 font
    5  CharProc  -> the single glyph's content stream
    6  Font      Helvetica, Type 1, WinAnsiEncoding        (READABLE)
    7  Font      Type 3, /CharProcs << /g27 5 0 R >>, NO /ToUnicode   ★ THE POINT

★★ **The Helvetica line contains `12-3456`, which the pattern `##-####`
matches.** That is the whole design of this fixture and it is the opposite of
the obvious one. A fixture where the pattern found NOTHING would light the
warning too, but it would prove only the already-known zero-hit case — and a
check written against it could pass on a build that printed the warning
whenever `created == 0`, which is a different and wrong rule. Here the
redaction **succeeds**, marks a real hit, reports success, and STILL owes the
disclosure, because a font on the same page could be hiding further
occurrences that no scan can see. That is the sentence the operator needs and
it is only reachable from a fixture that matches something.

★ **The Type 3 glyph is named `/g27`**, not `/square` or `/a`. ISO 32000-1
§9.10.2's glyph-name route (the "glyph-name extension" pdfcer counts
separately) can recover Unicode from a name that is in the Adobe Glyph List;
`g27` is in no list, so there is no accidental second route and the font is
unreadable for the reason claimed rather than by luck.

★ **The Type 3 font is actually SELECTED and DRAWN**, not merely present in
`/Resources`. `text_extract::page` raises `FontNote::Type3NoToUnicode` while
walking the content stream, so a font nothing sets would never be visited and
the counter would stay at zero — a fixture that looked right and asserted
nothing.

⚠ **The file is otherwise completely well-formed** — sound xref, correct
`/Length`, correct offsets. `Document::load_anomalies()` must return empty on
it. If this file also carried a structural fault, a check could go green on
the wrong disclosure: the status bar would have something to say either way.

## The engine's own words on why this population is invisible

`text_extract/page.rs`, the note raised by this exact construction:

    font <name> is a Type 3 font with NO /ToUnicode -- its glyphs are content
    streams named by arbitrary /CharProcs keys (ISO 32000-1 section 9.6.5), so
    section 9.10.2 leaves no sourced route to Unicode: text set in it RENDERS
    correctly but cannot be searched, copied or extracted. Acrobat is gated on
    the same entry

**Renders correctly.** Nothing on the page looks unredacted. That is the whole
hazard, and it is why this is a confidentiality defect rather than a polish one.

## Rebuilding

    python fixtures/unreadable-type3.PROVENANCE.py

Offsets are computed, so edits to the strings above are safe.
"""
import io

# The Type 3 glyph: a solid box, 750/1000 em square. `d1` (not `d0`) declares
# the glyph as shape-only with a bounding box, which is what lets a consumer
# colour it with the text fill colour -- the ordinary choice for a text glyph.
CHARPROC = b"750 0 0 0 750 750 d1\n0 0 750 750 re\nf\n"

# ONE readable line carrying the pattern, ONE unreadable line.
#
# The unreadable line is `aaaa` in the source bytes because the Type 3
# /Encoding maps code 97 to /g27; what it DRAWS is four boxes, and what any
# extractor can recover from it is nothing.
CONTENT = (
    b"BT /Helv 14 Tf 40 150 Td (Drawing 12-3456 rev B) Tj ET\n"
    b"BT /T3 28 Tf 40 90 Td (aaaa) Tj ET\n"
)

objects = [
    # 1 - catalog
    b"<< /Type /Catalog /Pages 2 0 R >>",
    # 2 - page tree
    b"<< /Type /Pages /Kids [3 0 R] /Count 1 >>",
    # 3 - page
    b"<< /Type /Page /Parent 2 0 R /MediaBox [0 0 400 220] "
    b"/Resources << /Font << /Helv 6 0 R /T3 7 0 R >> >> /Contents 4 0 R >>",
    # 4 - contents
    b"<< /Length %d >>\nstream\n" % len(CONTENT) + CONTENT + b"endstream",
    # 5 - the single /CharProcs stream
    b"<< /Length %d >>\nstream\n" % len(CHARPROC) + CHARPROC + b"endstream",
    # 6 - readable font
    b"<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica "
    b"/Encoding /WinAnsiEncoding >>",
    # 7 - THE POINT: Type 3, arbitrary /CharProcs key, and NO /ToUnicode.
    b"<< /Type /Font /Subtype /Type3 /FontBBox [0 0 750 750] "
    b"/FontMatrix [0.001 0 0 0.001 0 0] "
    b"/CharProcs << /g27 5 0 R >> "
    b"/Encoding << /Type /Encoding /Differences [97 /g27] >> "
    b"/FirstChar 97 /LastChar 97 /Widths [750] "
    b"/Resources << >> >>",
]

out = bytearray(b"%PDF-1.7\n%\xe2\xe3\xcf\xd3\n")
offsets = []
for i, body in enumerate(objects, start=1):
    offsets.append(len(out))
    out += b"%d 0 obj\n" % i
    out += body
    out += b"\nendobj\n"

xref_at = len(out)
n = len(objects) + 1
out += b"xref\n0 %d\n" % n
out += b"0000000000 65535 f \n"
for off in offsets:
    out += b"%010d 00000 n \n" % off
out += b"trailer\n<< /Size %d /Root 1 0 R >>\nstartxref\n%d\n%%%%EOF\n" % (n, xref_at)

path = "fixtures/unreadable-type3.pdf"
io.open(path, "wb").write(bytes(out))
print("wrote", path, len(out), "bytes")
