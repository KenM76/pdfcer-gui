#!/usr/bin/env python3
"""Generate ``fixtures/comment-note.pdf`` — a sheet carrying a REAL review thread.

Why this fixture exists
=======================

``tools/ui-verify``'s note pop-up checks assert that clicking a comment on the
page opens a window showing **the author, the date and the words**.  Every one
of those assertions is *vacuous* on a fixture whose annotation carries none of
them:

* a note with no ``/Contents`` makes "the pop-up shows the words" pass on a
  build that shows nothing;
* a note with no ``/T`` makes "the pop-up names the author" pass on a build
  that draws no byline at all;
* a note with no ``/Popup /Open true`` makes "a note the file authored open
  opens with the document" pass on a build that ignores ``/Open`` entirely —
  which is the exact defect `canvas::notepopup::model::read_open` was written
  to avoid, and the one an implementation reaches for by default.

`RESUME.md`'s falsification discipline names this class in as many words: *"an
absence check is vacuous if the run already stands where the defect lands."*
So this fixture is built to stand on the **opposite** side of every assertion.

What it contains
================

===============================  ================================================
object                           what it proves can be read
===============================  ================================================
``5 0 R``  ``/Text``             ``/Contents``, ``/T``, ``/M``, ``/Open true``,
                                 ``/C``, ``/Name /Note``, and a ``/Popup`` link
``6 0 R``  ``/Popup``            a pop-up with **its own** ``/Rect`` — placed
                                 deliberately away from the note, so a build
                                 that ignores the file's placement and always
                                 draws "beside the note" is visibly wrong
``7 0 R``  ``/Text`` + ``/IRT``  a reply: the thread's second entry, a different
                                 author, ``/RT /R``
``8 0 R``  ``/Text``             a **closed** note (``/Open false``) with words,
                                 so "the file's state is honoured" has a
                                 negative case as well as a positive one
===============================  ================================================

★ The second note matters as much as the first.  A build that opened *every*
pop-up on load would pass every assertion about the open one.  Object ``8`` is
what makes that fail.

Page geometry
=============

US Letter, 612 x 792 pt, one page, with a visible frame and a caption so a
screenshot of a failing run is legible.  The annotation rectangles are chosen
to sit well inside the sheet and away from each other:

===========  ====================  ==============================
object       ``/Rect``             aim point (PDF user space)
===========  ====================  ==============================
``5`` note   ``[100 600 120 620]``  ``0,110,610``
``6`` popup  ``[300 520 450 620]``  (not aimed at)
``8`` note   ``[100 300 120 320]``  ``0,110,310``
===========  ====================  ==============================

Regenerate with::

    python tools/gen-comment-note-fixture.py

It is deterministic — no clock is read, no random ids — so re-running it
produces byte-identical output and the fixture can be committed.
"""

from __future__ import annotations

import pathlib
import sys

# --- the document ----------------------------------------------------------
#
# Written by hand rather than through a library, for the reason every fixture
# generator in this tree is: the point of a fixture is to be *exactly* one
# shape, and a library's convenience layer decides things (a /Popup you did not
# ask for, an /AP you cannot control, a /CreationDate from the clock) that a
# check may then be asserting about by accident.

PAGE_W, PAGE_H = 612, 792

CONTENT = b"""q
0.2 0.2 0.2 RG
2 w
36 36 540 720 re
S
BT /F1 14 Tf 60 730 Td (pdfcer review fixture - two notes and one reply) Tj ET
BT /F1 10 Tf 130 606 Td (<- open note, with words, an author and a date) Tj ET
BT /F1 10 Tf 130 306 Td (<- closed note, with words) Tj ET
Q
"""


def obj(num: int, body: bytes) -> bytes:
    return b"%d 0 obj\n" % num + body + b"\nendobj\n"


def build() -> bytes:
    objects: dict[int, bytes] = {}

    objects[1] = b"<< /Type /Catalog /Pages 2 0 R >>"
    objects[2] = b"<< /Type /Pages /Kids [3 0 R] /Count 1 >>"
    objects[3] = (
        b"<< /Type /Page /Parent 2 0 R /MediaBox [0 0 %d %d] "
        b"/Resources << /Font << /F1 9 0 R >> >> "
        b"/Contents 4 0 R "
        b"/Annots [5 0 R 6 0 R 7 0 R 8 0 R] >>" % (PAGE_W, PAGE_H)
    )
    objects[4] = b"<< /Length %d >>\nstream\n" % len(CONTENT) + CONTENT + b"endstream"

    # 5 - the open note.  Every key a reviewer UI reads is present and
    # non-empty, which is what stops the checks being vacuous.
    objects[5] = (
        b"<< /Type /Annot /Subtype /Text /Rect [100 600 120 620] "
        b"/Name /Note /F 4 "
        b"/Contents (Check this weld before rev C) "
        b"/T (Ken Mantle) "
        b"/M (D:20260905143000Z) "
        b"/C [1 0.82 0.2] "
        b"/Open true "
        b"/Popup 6 0 R >>"
    )
    # 6 - its pop-up, placed AWAY from the note on purpose.  See the module
    # docstring: a build that always draws beside the note is visibly wrong
    # here, and identical to a correct one on a fixture whose /Popup sits where
    # "beside" would have put it anyway.
    objects[6] = (
        b"<< /Type /Annot /Subtype /Popup /Rect [300 520 450 620] "
        b"/Parent 5 0 R /Open true >>"
    )
    # 7 - a reply.  Different author, so a thread drawn with the parent's
    # byline repeated is distinguishable from one drawn correctly.
    objects[7] = (
        b"<< /Type /Annot /Subtype /Text /Rect [104 604 124 624] "
        b"/Name /Comment /F 4 "
        b"/Contents (Done - rev C issued 5 Sep) "
        b"/T (Jo Smith) "
        b"/M (D:20260905161500Z) "
        b"/IRT 5 0 R /RT /R "
        b"/Open false >>"
    )
    # 8 - a CLOSED note with words.  The negative case: a build that opens
    # every pop-up on load passes every assertion about object 5 and fails
    # here.
    objects[8] = (
        b"<< /Type /Annot /Subtype /Text /Rect [100 300 120 320] "
        b"/Name /Note /F 4 "
        b"/Contents (Dimension missing on this view) "
        b"/T (Ken Mantle) "
        b"/M (D:20260905143500Z) "
        b"/Open false >>"
    )
    objects[9] = b"<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>"

    out = bytearray(b"%PDF-1.7\n")
    offsets: dict[int, int] = {}
    for num in sorted(objects):
        offsets[num] = len(out)
        out += obj(num, objects[num])

    startxref = len(out)
    count = max(objects) + 1
    out += b"xref\n0 %d\n" % count
    out += b"0000000000 65535 f \n"
    for num in range(1, count):
        out += b"%010d 00000 n \n" % offsets[num]
    out += b"trailer\n<< /Size %d /Root 1 0 R >>\nstartxref\n%d\n%%%%EOF\n" % (
        count,
        startxref,
    )
    return bytes(out)


def main() -> int:
    root = pathlib.Path(__file__).resolve().parent.parent
    target = root / "fixtures" / "comment-note.pdf"
    target.parent.mkdir(parents=True, exist_ok=True)
    data = build()
    target.write_bytes(data)
    print(f"wrote {target} ({len(data)} bytes)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
