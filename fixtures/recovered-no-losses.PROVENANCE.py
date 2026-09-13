# -*- coding: utf-8 -*-
"""Build `fixtures/recovered-no-losses.pdf` — a file whose index is missing
entirely, so pdfcer rebuilds it by scanning, and whose scan keeps **everything
it finds**.

## Why this fixture exists: it is a CONTROL, and it is the whole check

Its sibling `recovered-with-losses.pdf` proves the dropped-object disclosure
appears. On its own that proves very little — a build that pinned the block to
every recovered document, or to every document full stop, would pass it while
announcing losses on files that lost nothing. That is the *cries wolf* failure
R8b rule 4 is most alert to, and it wears the same green tick as the real thing.

So the driven check launches twice, and this is the second launch.

★★★ **The control is a RECOVERED file, not a sound one.** A sound file would
have been the easy choice — `four-pages.pdf` is right there — but it would make
the absence attributable to three different states at once:

  1. the panel never opened,
  2. `Document::recovery()` was `None`, so nothing in the block's neighbourhood
     drew at all,
  3. the dropped-object block is correctly driven by `objects_dropped`.

Only the third is the thing under test, and an assertion satisfied by all three
is not a measurement of which one shipped. This file differs from its sibling in
**exactly one property** — the scan finds nothing it cannot keep — so the
check's two launches isolate that one variable. `properties.recovery` is
required on both launches, which disposes of (1) and (2) and leaves one reading.

## What this file contains

    1  Catalog
    2  Pages
    3  Page      -> MediaBox [0 0 400 200], one visible line of text
    4  Contents  -> ★ plain text with NO `N G obj` sequence anywhere in it
    5  Font      Helvetica

    NO `xref`, NO `trailer`, NO `startxref`  -> RecoveryReason::StartxrefNotFound

That is its sibling with the two traps removed: the content stream no longer
spells an object header inside a string, and there is no truncated object 8 at
the end of the file. Everything else — the header, the object numbering, the
page geometry, the font — is deliberately identical, because a control that
differs in more than one way cannot tell you which difference produced the
result. This project has filed a wrong engine request from exactly that mistake.

⚠ **The text drawn on the page must never contain the bytes `obj` preceded by
two integers.** That is the whole property this fixture asserts, and it is one
careless edit away from being false: a scan that finds a candidate header here
would drop it, the dropped block would draw, and the check's control half would
go red blaming the application for something true of the fixture. The shell's
`the_control_fixture_for_the_dropped_disclosure_really_recovers_and_drops_nothing`
asserts the property through the engine on every `cargo test`, so the breakage
surfaces in the cheap suite rather than in a driven sweep.

## Rebuilding

    python fixtures/recovered-no-losses.PROVENANCE.py

Nothing here is offset-dependent — there is no index to keep in step — so edits
are safe, subject to the warning above.
"""
import io

# ★ No `N G obj` sequence. The sentences are also the explanation, so anyone who
# opens this fixture by hand can see what it is for and what must stay true of
# it.
CONTENT = (
    b"BT /Helv 12 Tf 30 150 Td "
    b"(This file has no index either. pdfcer rebuilt it by scanning.) Tj "
    b"0 -20 Td (Unlike its sibling, the scan found nothing it could not keep.) Tj "
    b"0 -20 Td (So the properties panel must stay silent about losses.) Tj ET\n"
)

assert b"obj" not in CONTENT, (
    "the drawn text spells an object header, which would make the scan drop a "
    "candidate and destroy the one property this control fixture has"
)

objects = [
    (1, b"<< /Type /Catalog /Pages 2 0 R >>"),
    (2, b"<< /Type /Pages /Kids [3 0 R] /Count 1 >>"),
    (
        3,
        b"<< /Type /Page /Parent 2 0 R /MediaBox [0 0 400 200] "
        b"/Resources << /Font << /Helv 5 0 R >> >> /Contents 4 0 R >>",
    ),
    (4, b"<< /Length %d >>\nstream\n" % len(CONTENT) + CONTENT + b"endstream"),
    (
        5,
        b"<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica "
        b"/Encoding /WinAnsiEncoding >>",
    ),
]

out = bytearray(b"%PDF-1.7\n%\xe2\xe3\xcf\xd3\n")
for num, body in objects:
    out += b"%d 0 obj\n" % num
    out += body
    out += b"\nendobj\n"

# ⚠ Ends cleanly after object 5. No truncated tail, which is the second of the
# two differences from the sibling.

path = "fixtures/recovered-no-losses.pdf"
io.open(path, "wb").write(bytes(out))
print("wrote", path, len(out), "bytes")
