#!/usr/bin/env python3
"""Generate the two fixtures the **annotation-delete gate** needs.

    fixtures/certified-comments.pdf   — a certified document (`/Perms /DocMDP`,
                                        `/P 2`) carrying a markup comment
    fixtures/threaded-comments.pdf    — the SAME document with the certification
                                        removed

===========================================================================
WHY THIS FILE EXISTS
===========================================================================

``EditSession::annotation_deletion_refusal`` and
``annotation_deletion_preview`` are consumed by
``crate::panels::properties::annotdelete``, which decides two things:

  1. whether ``format.delete`` is drawn at all (R83), and
  2. what sentence is shown in its place, or what collateral is disclosed
     before the press.

R1 says a feature is not done until it has been asserted by **driving the
running binary**, and no fixture in this repository can drive either branch:

  * ``fixtures/signed-two-pages.pdf`` is deliberately an **approval**
    signature — its own generator's point (2) — so it carries no
    ``/Reference`` and no ``/Perms``. ``forbids_structural_change()`` is
    false on it and the gate is **open**, which is the branch that needed
    no new fixture.
  * nothing in ``fixtures/`` carries a markup annotation at all, so the
    collateral branch has nothing to select.

===========================================================================
THE TWO FIXTURES ARE ONE DOCUMENT, AND THAT IS THE POINT
===========================================================================

They differ in **exactly one dictionary**: the catalog's ``/Perms``. Same
pages, same annotations, same object numbers, same geometry.

That is not tidiness. A check that asserted "the control is absent on the
certified file" against one document and "the control is present on the
ordinary file" against a *different* document would be comparing two
variables at once, and a failure could not say which. Holding everything
but the certification constant makes the pair a controlled experiment: any
difference the harness sees between them is caused by ``/Perms``, because
nothing else differs.

===========================================================================
WHAT MAKES THE CERTIFIED ONE REFUSE, PRECISELY
===========================================================================

``signature::census`` reports ``forbids_structural_change()`` as
``perms_enforced && signatures > 0`` — and ``perms_enforced`` is the
**catalog's** ``/Perms → /DocMDP`` entry, Table 258's *"consumer
applications shall enforce the permissions"*. So three things are needed
and all three are here:

  1. a real signature dictionary (``/Type /Sig`` with a computed
     ``/ByteRange``), because the census counts those and nothing else;
  2. a ``/Reference`` array on it carrying ``/TransformMethod /DocMDP``,
     which is what makes it a **certification** rather than an approval and
     is where ``certification_permission`` is read from;
  3. the catalog's ``/Perms << /DocMDP 12 0 R >>``, which is the
     enforcement switch.

``/P 2`` is written explicitly even though Table 254 makes ``/P`` optional
with default 2. Absence would land in the same place, and writing it is
what makes the fixture *say what it is testing* — a reader should not have
to know a default to know why the file refuses.

★ ``/P 2``, not ``/P 1``. Both refuse, and 2 is the one worth fixturing:
it is the value a real certified form carries, and it is the boundary
``annotation_deletion_refusal`` exists to sit on. §12.8.2.2 Table 254 puts
annotation *creation, deletion and modification* on the ``P = 3`` line, so
``P = 2`` refuses and ``P = 3`` would allow — and a fixture at 1 would pass
a gate that had been written to compare against the wrong number.

===========================================================================
THE ANNOTATION, AND WHY IT HAS COMPANY
===========================================================================

Page 1 carries a ``/Square`` markup with:

  * a ``/Popup`` companion (§12.5.6.14 — the pop-up *"shall not appear
    alone"*, so deleting the parent must take it), and
  * one **reply**: a ``/Text`` annotation whose ``/IRT`` points at the
    square, with no ``/RT``. Table 170's default for an absent ``/RT`` is
    ``R``, so the engine classifies it as a reply rather than as a group
    subordinate — which is the ordinary case and therefore the one to
    fixture.

⇒ ``annotation_deletion_preview`` on the square therefore answers
``popup_removed: true`` and ``replies_orphaned: 1``, which is a collateral
sentence with **two** clauses. One clause would not prove the joining is
right; two does.

★ ``group_members_promoted`` is deliberately left at zero. A ``/RT /Group``
subordinate is the exotic branch, and adding one would make the sentence
three clauses without exercising anything the second clause does not — the
same argument the engine's own preview makes for reporting
``appearance_streams_removed`` as 0.

===========================================================================
NO CRYPTOGRAPHY, AND NONE CLAIMED
===========================================================================

``/Contents`` is filler zeros, exactly as in ``gen-signed-fixture.py``.
``pdfcer-core``'s signature module opens *"This module verifies nothing"*, so
a real certificate would test nothing these do and would drag a private key
into a repository.

**What these fixtures can and cannot support:** they CAN support *"pdfcer
found an enforced certification and withheld the control"*, which is the
whole of the feature. They CANNOT support any claim about signature
validity, and nothing should be written that reads as though they do.

===========================================================================
PROVENANCE
===========================================================================

Wholly synthetic, byte-authored by this script. No downloaded PDF is
involved and no PDF library produced these bytes, so neither fixture can
inherit a bug or a normalisation from the code it exercises.

Run from the repository root::

    python tools/gen-certified-fixture.py
"""

import pathlib

CERTIFIED = pathlib.Path("fixtures/certified-comments.pdf")
ORDINARY = pathlib.Path("fixtures/threaded-comments.pdf")

#: Width of the ``/Contents`` placeholder, in hex digits. Even (hex pairs) and
#: CONSTANT, because the byte-range arithmetic is measured from the laid-out
#: file rather than derived from this value — see ``gen-signed-fixture.py``.
CONTENTS_HEX_DIGITS = 512

#: A4 in points, matching the shell's own blank-document template so a driven
#: check's canvas coordinates read against a page size the rest of the suite
#: already uses.
MEDIA_BOX = "[0 0 595 842]"

#: The square's ``/Rect``, in PDF user space.
#:
#: Well inside the page and well away from the signature widget at
#: ``[60 60 300 120]``: a driven check clicks the centre of this rectangle, and
#: a click that landed on the widget instead would select a form field, take the
#: form surface's branch, and report the annotation gate as broken when what
#: actually happened is that the click missed.
SQUARE_RECT = "[120 560 320 700]"


def build(certified: bool) -> bytes:
    """Assemble one of the two fixtures and return its bytes.

    ``certified`` decides the single dictionary that differs: with it, the
    catalog carries ``/Perms << /DocMDP 12 0 R >>`` and the signature carries a
    ``/Reference`` naming ``/DocMDP`` with ``/P 2``. Without it, the same
    signature is an ordinary approval signature and the catalog has no
    ``/Perms`` — so ``forbids_structural_change()`` is false and every gate is
    open.
    """
    contents = b"0" * CONTENTS_HEX_DIGITS
    # Ten digits per number is enough for any file this script produces, and a
    # constant width means overwriting the placeholder cannot change an offset.
    br_placeholder = b"[0000000000 0000000000 0000000000 0000000000]"

    perms = b" /Perms << /DocMDP 12 0 R >>" if certified else b""
    # The `/Reference` array is what the census reads `certifications` and
    # `certification_permission` from. Without it the same `/Type /Sig` is an
    # approval signature, which is `fixtures/signed-two-pages.pdf`'s whole
    # subject and deliberately a different fixture.
    reference = (
        b" /Reference [ << /Type /SigRef /TransformMethod /DocMDP "
        b"/TransformParams << /Type /TransformParams /P 2 /V /1.2 >> >> ]"
        if certified
        else b""
    )

    objs: dict[int, bytes] = {}
    objs[1] = (
        b"<< /Type /Catalog /Pages 2 0 R /AcroForm << /Fields [10 0 R] "
        b"/SigFlags 3 >>" + perms + b" >>"
    )
    # TWO pages, matching `signed-two-pages.pdf`: a one-page document refuses a
    # page delete, and a check that wanted to combine an annotation assertion
    # with a structural one would have no page to spare.
    objs[2] = b"<< /Type /Pages /Kids [3 0 R 4 0 R] /Count 2 >>"
    objs[3] = (
        b"<< /Type /Page /Parent 2 0 R /MediaBox "
        + MEDIA_BOX.encode()
        + b" /Resources << >> /Contents 5 0 R /Annots [11 0 R 20 0 R 21 0 R 22 0 R] >>"
    )
    objs[4] = (
        b"<< /Type /Page /Parent 2 0 R /MediaBox "
        + MEDIA_BOX.encode()
        + b" /Resources << >> /Contents 6 0 R >>"
    )
    # A stroked rectangle per page, so the canvas has something to draw and a
    # human looking at a failed run can see which page is which.
    for num, rect in ((5, b"80 500 200 250"), (6, b"80 300 400 120")):
        stream = b"1 w 0 0 0 RG " + rect + b" re S\n"
        objs[num] = b"<< /Length %d >>\nstream\n" % len(stream) + stream + b"endstream"
    # The signature FIELD, merged with its widget (§12.5.6.19), on page 1.
    objs[10] = b"<< /FT /Sig /T (Certifier) /V 12 0 R /Kids [11 0 R] >>"
    objs[11] = (
        b"<< /Parent 10 0 R /Subtype /Widget /Rect [60 60 300 120] "
        b"/P 3 0 R /F 132 >>"
    )
    objs[12] = (
        b"<< /Type /Sig /Filter /Adobe.PPKLite "
        b"/SubFilter /adbe.pkcs7.detached "
        b"/Name (A. Certifier) /M (D:20260829120000Z)" + reference + b" "
        b"/ByteRange " + br_placeholder + b" /Contents <" + contents + b"> >>"
    )

    # ---- the markup, its pop-up and its reply ---------------------------
    #
    # ★ `/Square` rather than `/Highlight`: a highlight needs `/QuadPoints`
    # over real glyphs to be selectable where the operator sees it, and these
    # pages carry no text. A square's `/Rect` IS its geometry, so the click
    # target and the annotation agree by construction.
    objs[20] = (
        b"<< /Type /Annot /Subtype /Square /Rect " + SQUARE_RECT.encode() + b" "
        b"/P 3 0 R /F 4 /T (A. Reviewer) /M (D:20260829090000Z) "
        b"/Contents (Check this dimension.) /C [1 0 0] /CA 1 "
        b"/Popup 21 0 R /NM (square-under-test) >>"
    )
    # The pop-up. §12.5.6.14: it "shall not appear alone but is associated with
    # a markup annotation, its parent annotation" — so deleting 20 must take
    # this with it, and `popup_removed` is how the engine says so.
    objs[21] = (
        b"<< /Type /Annot /Subtype /Popup /Rect [330 560 530 700] "
        b"/P 3 0 R /Parent 20 0 R /Open false >>"
    )
    # The reply. NO `/RT`, deliberately: Table 170's default is `R`, so the
    # engine classifies this as a reply rather than a group subordinate — the
    # ordinary case, and therefore the one to fixture. Its `/IRT` names 20,
    # which is what makes `replies_orphaned` 1 rather than 0.
    objs[22] = (
        b"<< /Type /Annot /Subtype /Text /Rect [120 520 140 540] "
        b"/P 3 0 R /F 4 /T (B. Reviewer) /M (D:20260829100000Z) "
        b"/Contents (Agreed, it is wrong.) /IRT 20 0 R /Name /Comment >>"
    )

    buf = bytearray(b"%PDF-1.7\n%\xe2\xe3\xcf\xd3\n")
    off: dict[int, int] = {}
    for n in sorted(objs):
        off[n] = len(buf)
        buf += b"%d 0 obj\n" % n + objs[n] + b"\nendobj\n"

    xref_at = len(buf)
    size = max(objs) + 1
    buf += b"xref\n0 %d\n0000000000 65535 f \n" % size
    for n in range(1, size):
        if n in off:
            buf += b"%010d 00000 n \n" % off[n]
        else:
            buf += b"0000000000 65535 f \n"
    buf += b"trailer\n<< /Size %d /Root 1 0 R >>\nstartxref\n%d\n%%%%EOF\n" % (
        size,
        xref_at,
    )

    # The file is laid out, so the hole's real position is known. `+1` and the
    # trailing arithmetic step over the `<` and `>` delimiters: the digest
    # excludes the signature VALUE, not the string syntax around it.
    lt = buf.index(b"<" + contents)
    gt = lt + 1 + len(contents)
    total = len(buf)
    first = (0, lt + 1)
    second = (gt, total - gt)

    real = b"[%d %d %d %d]" % (first[0], first[1], second[0], second[1])
    if len(real) > len(br_placeholder):
        raise AssertionError(f"byte range {real!r} exceeds the reserved width")
    # Pad INSIDE the array so no offset shifts. If this ever failed to fit, the
    # arithmetic above is wrong and silence would hide it — hence the raise.
    real = real[:-1] + b" " * (len(br_placeholder) - len(real)) + b"]"
    i = buf.index(br_placeholder)
    buf[i : i + len(br_placeholder)] = real
    return bytes(buf)


def main() -> None:
    """Write both fixtures."""
    for path, certified in ((CERTIFIED, True), (ORDINARY, False)):
        data = build(certified)
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(data)
        print(
            f"wrote {path} {len(data)} bytes  "
            f"certified={'yes' if certified else 'no'}  pages=2  "
            f"annots=square+popup+reply"
        )


if __name__ == "__main__":
    main()
