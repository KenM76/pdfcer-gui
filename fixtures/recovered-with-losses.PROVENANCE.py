# -*- coding: utf-8 -*-
"""Build `fixtures/recovered-with-losses.pdf` — a file whose index is missing
entirely, so pdfcer rebuilds it by scanning, and whose scan finds two things it
cannot keep.

## Why this fixture exists

Engine `fb6e004`, owed item 18, `decision 145`. Recovery used to hand back a
shorter document and no explanation: on a real file (`Annotations_output.pdf`,
a PDFsharp writer bug that points `startxref` 134 bytes short of its own
`xref`) it recovered 10 of 11 objects and **dropped the page's content stream
with nothing recorded anywhere**. `RecoveryReport::objects_dropped` now names
every object the scan found and recovery could not keep, with a reason on each.

★★★ **Nothing else in `fixtures/` can reach that field.** Every other file here
either has a sound cross-reference table — in which case `Document::recovery()`
is `None` and there is no report at all — or, like `contradicts-itself.pdf`,
deliberately avoids the recovery path so that it can test the *anomaly* path
without lighting this one. A check driven against any of them would be
asserting the absence of a disclosure on a document that has nothing to
disclose: a check that cannot fail, which is a shape this project has filed by
accident more than once.

## What this file contains

    1  Catalog
    2  Pages
    3  Page      -> MediaBox [0 0 400 200], one visible line of text
    4  Contents  -> ★ the stream DATA contains the bytes `9 0 obj`
    5  Font      Helvetica
    8  ★ a truncated object: `8 0 obj` followed by `<< /Type` and nothing else

    NO `xref`, NO `trailer`, NO `startxref`  -> RecoveryReason::StartxrefNotFound

## ★★ The two drops are deliberately the two DIFFERENT stories that share one
## reason code, because that is the pair the disclosure has to keep apart

`DropReason::Unparseable` covers both of these, and the engine's own doc says
why that is not sloppiness — from outside the crate they are the same event:

- **Object 9 does not exist.** The bytes `9 0 obj` sit inside the content
  stream's own text, as part of a string being drawn on the page. The scan is
  obliged to try every byte sequence that spells `N G obj`, so it finds this
  one; parsing from there hits `<< /Type ) Tj` and fails. **Nothing is lost.**
  This is the overwhelmingly common case on real damaged files, and it is why
  the disclosure must not be written in alarming language.
- **Object 8 is real and is gone.** Its body starts and stops mid-dictionary,
  exactly as a truncated download or a crashed writer leaves one. Losing it is
  the correct outcome — keeping it would hand the caller bytes that cannot be
  read — but losing it in silence is the gap this fixture exists to prove is
  closed.

⚠ **The document still opens, with a page you can see.** That is load-bearing
for a driven check: the disclosure lives in Document properties, and a panel
cannot be read on a document that failed to open. Objects 1–5 are well-formed
and the page renders normally, which is also the honest picture of the class of
file this happens to — it looks completely fine on screen.

⚠ **`DropReason::IdMismatch` is NOT produced by this fixture, and that is
stated rather than left to be discovered.** That arm fires when an object
parses but its own id disagrees with the header the scan matched at that
offset, and the two readings come from the same bytes, so it cannot be forced
from a hand-written file without a parser/scanner divergence to exploit. The
`id_mismatch` half of `crate::text::panels::docprops::dropped_summary` is
therefore covered by its unit tests and not by a driven check. Saying so here
is the point: an untested branch that nobody has written down is indistinguish-
able from a tested one.

## Why there is no `xref` at all rather than a subtly wrong one

A wrong `startxref` offset is the more realistic damage and is what the real
file had. It is also a second variable: a *nearly* right offset can land
recovery in a different `RecoveryReason`, and the reason is printed in the
panel above the lines this fixture is for. Omitting the trailer entirely gives
exactly one trigger (`StartxrefNotFound`) and makes the fixture's own behaviour
a fact rather than a byte-offset coincidence.

## Rebuilding

    python fixtures/recovered-with-losses.PROVENANCE.py

Nothing here is offset-dependent — there is no index to keep in step — so edits
are safe.
"""
import io

# The parenthesised text is drawn on the page AND is the false-positive object
# header. `9 0 obj` must be followed by something that cannot start an object;
# `<< /Type ) Tj` is a dictionary that ends at a `)`, which no lenient policy
# accepts. The visible sentence is also the explanation, so anyone who opens
# this fixture by hand can see what it is for.
CONTENT = (
    b"BT /Helv 12 Tf 30 150 Td "
    b"(This file has no index. pdfcer rebuilt it by scanning.) Tj "
    b"0 -20 Td (The next line is a trap for that scan: 9 0 obj << /Type ) Tj "
    b"0 -20 Td (...and object 8 below is really truncated.) Tj ET\n"
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

# Object 8: a real definition that stops mid-dictionary. It is LAST so that
# nothing after it can accidentally terminate it into something parseable.
out += b"8 0 obj\n<< /Type \n"

path = "fixtures/recovered-with-losses.pdf"
io.open(path, "wb").write(bytes(out))
print("wrote", path, len(out), "bytes")
