# -*- coding: utf-8 -*-
"""Build `fixtures/action-names-field.pdf` — a form whose button names its
target **by name string**, which is the only shape the engine's action-target
counters can see.

## Why this fixture had to be hand-authored

`FieldRename::action_targets_retargeted` and
`FieldDeletion::action_targets_orphaned` traverse `/ResetForm` and
`/SubmitForm` `/Fields` arrays and `/Hide` `/T` values looking for
**fully-qualified name strings**. ISO 32000-1 §12.7.5.3 allows that array to
hold *either* indirect references to field dictionaries *or* text strings
naming them, and the two are not interchangeable for this purpose: a rename
cannot break an indirect reference (the object still exists), so the engine's
traversal is structurally blind to it and correctly reports zero.

`fixtures/submit-button.pdf` — the only other fixture in this repository
carrying a form action at all — uses `/Fields [4 0 R]`, an indirect reference.
So it reports zero for both counters and cannot exercise either disclosure.
That is not a defect in that fixture; it is a different shape.

## What this file contains

    1  Catalog       -> /AcroForm { /Fields [4 0 R, 5 0 R] }
    2  Pages
    3  Page          -> /Annots [4 0 R, 5 0 R]
    4  Widget+Field  /FT /Tx  /T (Amount)          -- the target
    5  Widget+Field  /FT /Btn /T (ResetIt)         -- the button
                     /A << /S /ResetForm /Fields [(Amount)] >>
    6  Font          Helvetica

★ `/Fields [(Amount)]` — a **string**, not a reference. That single pair of
parentheses is the entire point of the fixture, and a well-meaning tidy-up that
"fixed" it into `[4 0 R]` would make both disclosures untestable again while
every test still compiled.

★ The button is a real terminal field with its own `/T`, not a bare annotation,
so `delete_field("Amount")` leaves it behind to be counted rather than removing
it as part of the same subtree.

## Rebuilding

    python fixtures/action-names-field.PROVENANCE.py

Offsets are computed, so edits are safe — unlike `autosize-field.PROVENANCE.py`,
which preserves them by construction because it edits a shipped file in place.
"""
import io

objects = [
    # 1 — catalog
    b"<< /Type /Catalog /Pages 2 0 R /AcroForm << /Fields [4 0 R 5 0 R] "
    b"/DA (/Helv 0 Tf 0 g) /DR << /Font << /Helv 6 0 R >> >> >> >>",
    # 2 — page tree
    b"<< /Type /Pages /Kids [3 0 R] /Count 1 >>",
    # 3 — page
    b"<< /Type /Page /Parent 2 0 R /MediaBox [0 0 300 200] "
    b"/Resources << /Font << /Helv 6 0 R >> >> /Annots [4 0 R 5 0 R] >>",
    # 4 — the text field the button names
    b"<< /Type /Annot /Subtype /Widget /FT /Tx /T (Amount) /Rect [40 120 260 145] "
    b"/F 4 /P 3 0 R /DA (/Helv 10 Tf 0 g) /V (100) >>",
    # 5 — the button, naming its target BY NAME STRING
    b"<< /Type /Annot /Subtype /Widget /FT /Btn /Ff 65536 /T (ResetIt) "
    b"/Rect [40 40 160 70] /F 4 /P 3 0 R /DA (/Helv 10 Tf 0 g) "
    b"/MK << /CA (Reset) >> "
    b"/A << /S /ResetForm /Fields [(Amount)] >> >>",
    # 6 — font
    b"<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica /Encoding /WinAnsiEncoding >>",
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

path = "fixtures/action-names-field.pdf"
io.open(path, "wb").write(bytes(out))
print("wrote", path, len(out), "bytes")
