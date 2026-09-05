#!/usr/bin/env python3
"""Build ``fixtures/annots-with-everything.pdf``.

WHY THIS FIXTURE EXISTS
=======================

It is the subject of the **annotation clipboard**'s losslessness assertions
(``canvas::annotclip``), and every property below is load-bearing rather than
decorative.  A fixture whose annotations carry only the keys a
``MarkupSpec`` can already express would make "the copy is lossless" pass
against a build that still re-authors from the spec — which is exactly the
vacuous shape this project's falsification discipline forbids.  So every
annotation here carries **``/CA``, ``/T``, ``/M`` and ``/Contents``**, none of
which a ``MarkupSpec`` models, and two of them carry a baked ``/AP`` that no
authoring verb in ``pdfcer-core`` would reproduce.

The three annotations are chosen to land on **three different carriers**
inside ``EditSession::copy_selection`` — a classification the shell reads back
off the clip rather than deriving from a subtype list of its own:

===========  ==========================  =================================
annotation   carrier the engine picks    what it proves
===========  ==========================  =================================
``/Square``  ``ClipAnnotation::Markup``  ``spec_from_dict`` succeeds, so
                                         pdfcer models it and the engine
                                         carries a **spec** — which drops
                                         ``/CA``, ``/T``, ``/M`` and
                                         ``/Contents``.  This is the
                                         annotation that proves the
                                         "lossless" route is *not* lossless
                                         for the kinds pdfcer models, and
                                         therefore the one that justifies
                                         the shell keeping its own
                                         spec-plus-options path for them.
``/Text``    ``ClipAnnotation::Raw``     a sticky note — **the single
                                         most-copied comment in a review
                                         workflow**, and until now the shell
                                         refused to copy it at all
                                         (``spec_from_dict`` has no reader
                                         for it).  Raw carries the whole
                                         dictionary plus its ``/AP``
                                         closure, so this is the annotation
                                         whose copy is genuinely lossless.
``/FreeText`` ``ClipAnnotation::Raw``    a text box, same carrier, plus a
                                         ``/DA`` and a ``/DS`` that only a
                                         dictionary copy preserves.
===========  ==========================  =================================

WHAT IS DELIBERATELY *NOT* HERE
-------------------------------

No ``/Popup`` and no ``/IRT``.  ``pdfcer-core`` strips both on the way to the
clipboard (``CLIP_STRIPPED_ANNOT_KEYS``) because each names a *relationship*
in the source document, and a key-by-key equality assertion over the pasted
dictionary would have to special-case them.  ``fixtures/threaded-comments.pdf``
already carries a ``/Popup`` and an ``/IRT`` and is the fixture for that
subject; keeping them out of this one is what lets the equality assertion be
written **without a hand-maintained exception list**, which is the same defect
shape the whole exercise is about.

``/NM`` is likewise absent: §12.5.2 requires it to be unique within its page,
so the engine strips it and a second paste onto one page would collide.

Run
---

    python tools/gen-annots-with-everything-fixture.py

Idempotent: it rewrites the file byte-for-byte from this source, so a
regenerated fixture is a no-op in ``git status`` unless this script changed.
"""

from __future__ import annotations

import pathlib

# ---------------------------------------------------------------------------
# The objects, in order.  Index i in this list is object number i + 1.
# ---------------------------------------------------------------------------
#
# ★ Written as literal bytes rather than through a PDF library on purpose: the
#   point of a fixture is that a reader can see exactly which keys are present
#   without running anything, and a library would decide some of them for us.

PAGE_CONTENT = b"1 w 0 0 0 RG 60 60 470 720 re S\n"

# The sticky note's baked appearance -- a filled disc.  Small, but it is a real
# `/AP` closure: a stream object referenced from an `/AP << /N ... >>`, which is
# what the raw carrier has to walk and copy by value.  A build that carried the
# dictionary and dropped the closure would paste a note with no appearance,
# which renders as nothing at all and errors nowhere.
NOTE_AP = b"0 0 1 rg 0 0 20 20 re f\n"

# The text box's appearance, likewise.
FREETEXT_AP = b"0.9 0.9 0.2 rg 0 0 160 40 re f\n0 0 0 rg\n"

OBJECTS: list[bytes] = [
    # 1 -- catalog
    b"<< /Type /Catalog /Pages 2 0 R >>",
    # 2 -- page tree
    b"<< /Type /Pages /Kids [3 0 R] /Count 1 >>",
    # 3 -- the page.  /Annots order IS the index space `copy_annotations`
    #      addresses: 0 = the square, 1 = the note, 2 = the text box.
    b"<< /Type /Page /Parent 2 0 R /MediaBox [0 0 595 842] /Resources << >> "
    b"/Contents 4 0 R /Annots [5 0 R 6 0 R 8 0 R] >>",
    # 4 -- page content stream (so the page has one; a clip of annotations
    #      alone must work on a page WITHOUT content too, which
    #      `threaded-comments.pdf` covers)
    b"<< /Length " + str(len(PAGE_CONTENT)).encode() + b" >>\nstream\n" + PAGE_CONTENT + b"endstream",
    # 5 -- /Square.  Modelled by `spec_from_dict` => Markup carrier.
    b"<< /Type /Annot /Subtype /Square /Rect [120 560 320 700] /P 3 0 R /F 4 "
    b"/T (A. Reviewer) /M (D:20260905090000Z) /Contents (Check this dimension.) "
    b"/C [1 0 0] /IC [1 1 0] /CA 0.4 /BS << /W 3 /S /S >> >>",
    # 6 -- /Text, a sticky note.  Not modelled => Raw carrier.
    b"<< /Type /Annot /Subtype /Text /Rect [360 660 380 680] /P 3 0 R /F 4 "
    b"/T (B. Reviewer) /M (D:20260905100000Z) /Contents (Agreed, it is wrong.) "
    b"/Name /Comment /C [1 1 0] /CA 0.4 /AP << /N 7 0 R >> >>",
    # 7 -- the note's appearance stream
    b"<< /Type /XObject /Subtype /Form /BBox [0 0 20 20] /Resources << >> /Length "
    + str(len(NOTE_AP)).encode()
    + b" >>\nstream\n"
    + NOTE_AP
    + b"endstream",
    # 8 -- /FreeText, a text box.  Not modelled => Raw carrier.
    b"<< /Type /Annot /Subtype /FreeText /Rect [120 400 280 440] /P 3 0 R /F 4 "
    b"/T (C. Reviewer) /M (D:20260905110000Z) /Contents (Revise per RFI 12.) "
    b"/DA (0 0 0 rg /Helv 9 Tf) /Q 0 /CA 0.5 /AP << /N 9 0 R >> >>",
    # 9 -- the text box's appearance stream
    b"<< /Type /XObject /Subtype /Form /BBox [0 0 160 40] /Resources << >> /Length "
    + str(len(FREETEXT_AP)).encode()
    + b" >>\nstream\n"
    + FREETEXT_AP
    + b"endstream",
]


def build() -> bytes:
    out = bytearray(b"%PDF-1.7\n%\xe2\xe3\xcf\xd3\n")
    offsets: list[int] = []
    for number, body in enumerate(OBJECTS, start=1):
        offsets.append(len(out))
        out += b"%d 0 obj\n" % number
        out += body
        out += b"\nendobj\n"

    startxref = len(out)
    out += b"xref\n0 %d\n" % (len(OBJECTS) + 1)
    out += b"0000000000 65535 f \n"
    for offset in offsets:
        out += b"%010d 00000 n \n" % offset
    out += b"trailer\n<< /Size %d /Root 1 0 R >>\nstartxref\n%d\n%%%%EOF\n" % (
        len(OBJECTS) + 1,
        startxref,
    )
    return bytes(out)


def main() -> None:
    root = pathlib.Path(__file__).resolve().parent.parent
    target = root / "fixtures" / "annots-with-everything.pdf"
    target.write_bytes(build())
    print(f"wrote {target} ({target.stat().st_size} bytes)")


if __name__ == "__main__":
    main()
