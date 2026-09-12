"""Author `orphan-widget.pdf` — one page, one widget annotation NO field owns.

Run from the repository root:

    python fixtures/orphan-widget.PROVENANCE.py

It rewrites `fixtures/orphan-widget.pdf` byte-for-byte deterministically (no
timestamps, no randomness, no producer string), so a re-run in a clean tree
leaves `git status` clean. That is a property worth having: a fixture whose
bytes move on every run cannot be reviewed in a diff.

===========================================================================
WHAT IT IS FOR
===========================================================================

`EditSession::adopt_widget` registers an unclaimed `/Widget` annotation as a
form field. On 2026-09-12 the engine added a guard (`Pass 298.0`, `93f329b`) so
that a **dotted** name is refused rather than authoring a field nobody can
address: `/T` is the ONE segment a field contributes to its fully-qualified
name (§12.7.3.2), so `Text.2` at the `/Fields` root becomes an FQN that every
resolver splits on `.` before looking anything up — it finds the real terminal
`Text`, stops, and the field renders, accepts a click, and can never be filled.

pdfcer-gui asked for that refusal, and the surface that can provoke it is the
per-widget name box in `panels::forms::tab_order::register` — free text, gated
only on non-empty. So the shell needs a test that drives `actions::forms::adopt`
with a dotted name and asserts the operator gets a sentence rather than the
funnel's generic shrug.

That test needs an **unowned widget**, and there was no fixture with one.

===========================================================================
★★★ WHY IT IS HAND-AUTHORED HERE AND NOT PRODUCED BY pdfcer
===========================================================================

Two reasons, and the second is the one that matters.

**1. pdfcer cannot make this shape.** Every verb that creates a widget
(`add_text_field` and its four siblings, `paste_field`) registers it in
`/AcroForm /Fields` in the same commit — that is the whole point of them. An
*un*registered widget is what a **damaged or third-party** file looks like:
a form flattened by a tool that dropped `/AcroForm` but left the annotations, or
a page extracted from a form without its field tree. There is no sequence of
pdfcer edits that produces it, which is exactly why the adopt verb exists.

**2. A fixture produced by the code under test is not independent.** If pdfcer
wrote this file, a test reading it would be measuring pdfcer's agreement with
itself, and a shared misreading would pass. Written from ISO 32000-1 by hand,
the round trip is a measurement: the bytes say what the spec says, and pdfcer
either reads them that way or does not.

===========================================================================
THE SHAPE, AND EVERY PRECONDITION IT HAS TO MEET
===========================================================================

`adopt_plan` (engine `edit.rs:41004`) refuses in five ways before it reaches the
dotted-name guard. The fixture has to clear all five, so each is listed with the
byte in this file that satisfies it:

| refusal | what it checks | how this file clears it |
|---|---|---|
| `DocumentEncrypted` | `/Encrypt` in the trailer | there is none |
| certification | a certifying signature | there is none |
| `NotAWidget` | the object resolves to a dict with `/Subtype /Widget` | object 4 is exactly that |
| `WidgetAlreadyOwned` | the widget appears under a parsed `/AcroForm` field | **the catalog has no `/AcroForm` at all** |
| `FieldNameEmpty` / `WidgetHasNoFieldIdentity` | the supplied name, else the widget's own `/T` | the widget carries `/T (Orphan)`, so `None` also works |

★ `/T (Orphan)` is deliberate and is not decoration. It makes this the **merged
field-widget** shape — a widget that IS its own field and was simply never
registered — which is the recoverable case `adopt_widget` was written for, and
the one whose `AdoptOutcome::name` the preview shows the operator *before* the
press. A bare kid widget with no `/T` is the unrecoverable case and would refuse
with `WidgetHasNoFieldIdentity` before any name check, which would make a
dotted-name test pass for the wrong reason.

`/FT /Tx` likewise: without it `AdoptOutcome::field_type` is `None` and the
registration succeeds into a typeless, unfillable field. Present here so the
fixture exercises the *ordinary* path and a test asserting the dotted refusal
cannot be confused with a test about inheritance.

===========================================================================
PDF STRUCTURE
===========================================================================

    1  /Catalog   -> /Pages 2            (★ NO /AcroForm — see the table)
    2  /Pages     -> [3], /Count 1
    3  /Page      /MediaBox [0 0 612 792], /Annots [4], /Contents 5
    4  /Annot     /Subtype /Widget /FT /Tx /T (Orphan) /Rect [72 680 300 704]
    5  stream     one line of content so the page is not blank on screen

Written with a classic cross-reference **table** rather than an xref stream, on
purpose: byte offsets in a table are auditable by eye against a hex dump, and a
fixture nobody can audit is a fixture nobody trusts. Offsets are computed from
the assembled body below, never typed — a hand-typed offset is the single most
common way a minimal PDF is subtly broken, and the one a reader cannot see.
"""

import io
import os

NL = chr(10)

# ---------------------------------------------------------------------------
# The objects, as literal bytes. Object 5's stream length is patched in below
# rather than counted by hand, for the same reason the xref offsets are.
# ---------------------------------------------------------------------------
CONTENT = b"BT /Helv 12 Tf 72 720 Td (A widget no field owns.) Tj ET" + NL.encode()

OBJECTS = [
    # 1 — the catalog. ★ No /AcroForm: that absence is the whole fixture.
    b"<< /Type /Catalog /Pages 2 0 R >>",
    # 2 — the page tree.
    b"<< /Type /Pages /Kids [3 0 R] /Count 1 >>",
    # 3 — the page. /Annots names the widget; nothing else does.
    b"<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] "
    b"/Resources << /Font << /Helv << /Type /Font /Subtype /Type1 "
    b"/BaseFont /Helvetica >> >> >> "
    b"/Annots [4 0 R] /Contents 5 0 R >>",
    # 4 — the unowned widget. A merged field-widget: it carries its own /T and
    # /FT, so it is the recoverable shape, and no /Parent, so nothing claims it.
    b"<< /Type /Annot /Subtype /Widget /FT /Tx /T (Orphan) /F 4 "
    b"/Rect [72 680 300 704] "
    b"/MK << /BC [0 0 0] /BG [1 1 1] >> /DA (/Helv 12 Tf 0 g) >>",
    # 5 — page content, so the page is not blank.
    b"<< /Length " + str(len(CONTENT)).encode() + b" >>" + NL.encode()
    + b"stream" + NL.encode() + CONTENT + b"endstream",
]

# ---------------------------------------------------------------------------
# Assemble, recording each object's byte offset as it is written. The xref
# table is built FROM these offsets, never from typed numbers.
# ---------------------------------------------------------------------------
buf = io.BytesIO()
# %PDF-1.7 plus a binary comment line, which tells transfer tools the file is
# not text. §7.5.2 recommends it and several readers rely on it.
buf.write(b"%PDF-1.7" + NL.encode())
buf.write(b"%" + bytes([0xE2, 0xE3, 0xCF, 0xD3]) + NL.encode())

offsets = []
for n, body in enumerate(OBJECTS, start=1):
    offsets.append(buf.tell())
    buf.write(str(n).encode() + b" 0 obj" + NL.encode())
    buf.write(body + NL.encode())
    buf.write(b"endobj" + NL.encode())

startxref = buf.tell()
buf.write(b"xref" + NL.encode())
buf.write(b"0 " + str(len(OBJECTS) + 1).encode() + NL.encode())
# Entry 0 is the head of the free list and is always exactly this.
buf.write(b"0000000000 65535 f " + NL.encode())
for off in offsets:
    # Each entry is EXACTLY 20 bytes: 10-digit offset, space, 5-digit
    # generation, space, keyword, two-byte EOL. §7.5.4 is strict about this and
    # a 19-byte entry breaks every reader that seeks by multiplication.
    buf.write(str(off).zfill(10).encode() + b" 00000 n " + NL.encode())
buf.write(b"trailer" + NL.encode())
buf.write(b"<< /Size " + str(len(OBJECTS) + 1).encode() + b" /Root 1 0 R >>" + NL.encode())
buf.write(b"startxref" + NL.encode())
buf.write(str(startxref).encode() + NL.encode())
buf.write(b"%%EOF" + NL.encode())

raw = buf.getvalue()

# Self-check before writing: every xref entry must be 20 bytes, and the offset
# the table gives for each object must actually land on that object's header.
# A fixture whose xref is wrong fails a test for a reason that has nothing to do
# with the test, and the failure message will not say so.
for n, off in enumerate(offsets, start=1):
    header = str(n).encode() + b" 0 obj"
    assert raw[off:off + len(header)] == header, (n, off, raw[off:off + 16])
assert raw[startxref:startxref + 4] == b"xref", raw[startxref:startxref + 16]

out = os.path.join(os.path.dirname(os.path.abspath(__file__)), "orphan-widget.pdf")
io.open(out, "wb").write(raw)
print("wrote", out, len(raw), "bytes;", len(OBJECTS), "objects, xref at", startxref)
