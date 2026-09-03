#!/usr/bin/env python3
"""Generate the ONE fixture that is **certified AND nested at the same time**.

    fixtures/certified-nested-form.pdf  — `/Perms /DocMDP` at `/P 2`, over a
                                          two-level field-name tree
                                          (`Personal.Name`,
                                          `Personal.Address.City`,
                                          `Personal.Address.Zip`)

===========================================================================
WHY THIS FILE EXISTS
===========================================================================

``tools/ui-verify/src/checks/form_groups.rs``'s check
``structural_refusals_are_sentences_not_controls`` carries an assertion that,
until this fixture, **had nothing to run on**: that on a certified document the
Forms panel's *Field groups* section lists its grouping nodes, says in prose why
they cannot be removed, and **draws no Delete-group control at all** (R9 — a
permanently-refused capability renders *nothing*, not a greyed button).

Running that assertion needs a document that is **both** of two things, and no
file in either corpus was both:

  * ``fixtures/certified-comments.pdf`` — certified (this repository's
    ``gen-certified-fixture.py``), but its fields are **flat**: one ``/Sig``
    field, no interior in the name tree, so ``AcroForm::groups`` is empty and
    ``groups::section`` takes its early return before any arm control could be
    withheld.
  * ``D:/Dev/pdfcer/fixtures/synthetic/forms/certified-p2-form.pdf`` — certified,
    and **also flat** (``FullName`` text + ``Subscribe`` check box).
  * ``D:/Dev/pdfcer/fixtures/synthetic/forms/nested-form.pdf`` — the only file in
    either corpus with grouping nodes at all, and **not certified**: no
    ``/Perms``, no signature, every gate open.

⇒ Phase F of that check therefore reported, honestly, that its arm-withholding
half *did not run* — an absence with nothing behind it. This file is the
missing intersection, and phase F now points at it.

★★★ **The failure mode this script is written against.** A fixture that loads
and whose ``AcroForm::groups`` is EMPTY makes the check **pass while testing
nothing** — the section early-returns, no arm control is drawn, and "no arm
control was drawn" is exactly what the assertion looks for. That is strictly
worse than the SKIP it replaces, because a SKIP is legible in the report and a
vacuous pass is not. Hence the four-point verification recorded below, and
hence the unit test in ``crates/pdfcer-gui/src/app/actions/forms/delete.rs``
that pins all four *from inside the crate* so a regression in the engine's
walk cannot quietly re-empty the list.

===========================================================================
WHY A NEW SCRIPT RATHER THAN A THIRD OUTPUT OF `gen-certified-fixture.py`
===========================================================================

``gen-certified-fixture.py``'s entire thesis — stated at length in its own
header — is that its two outputs **are one document**, differing in exactly one
dictionary, so that any behavioural difference the harness sees between them is
caused by the certification and by nothing else. A third output built here would
share none of that: different page count, different page size, no annotations,
a field tree instead of a flat pair. It would be a second subject wearing the
first one's title, and the "they differ in exactly one dictionary" sentence in
that file would become false the moment a reader took it to cover all three.

So: separate script, separate subject, and the shape, the docstring conventions,
the ``/Contents`` filler and the computed-``/ByteRange`` machinery are copied
from it deliberately — a reader who has read one can read the other.

===========================================================================
THE FIELD TREE, AND WHY IT IS A COPY OF `nested-form.pdf`'S
===========================================================================

``pdfcer-core``'s ``AcroForm::groups`` collects the field-name tree's
**interior** — the *pure* non-terminals, the nodes with child fields and no
widget kids of their own. ``forms::walk_field`` records one at the early return
it takes when ``!child_fields.is_empty() && widget_kids.is_empty()``, and
nowhere else. Two consequences decide this file's shape:

  1. **A grouping node must have at least two terminal fields under it** for the
     shell's *Delete group…* to be describing a cascade rather than a rename of
     a single field. ``Personal`` has three.
  2. **Two levels, not one.** ``Personal`` holds ``Address`` (itself a pure
     non-terminal) and ``Name`` (a terminal, one level shallower). So
     ``groups`` is **two** entries, and deleting ``Personal`` also empties
     ``Personal.Address`` — a node the operator never named. That cascade is
     the thing the Field-groups section exists to disclose, and a one-level
     tree cannot exhibit it.

★ ``Name`` sits one level shallower than ``City``/``Zip`` **on purpose**, which
is ``nested-form.pdf``'s own recorded reason (engine ``PROVENANCE.md``): a walk
carrying a single shared depth counter gets exactly one of the two wrong, and a
tree of uniform depth would let that bug through.

★ The tree is byte-for-byte the same *shape* as ``nested-form.pdf``'s — same
partial names, same nesting, same ``/DA`` inheritance from ``Address`` and same
``/FT /Tx`` inheritance from ``Personal`` — so a difference between a run on
this file and a run on that one is caused by the certification. It is not the
same *document* (this one is A4, carries a signature and a content stream), so
the pair is a weaker control than ``gen-certified-fixture.py``'s; the strong
control here is **inside** the file, and that is the next section.

===========================================================================
`/P 2`, AND WHY NOT `/P 1` OR `/P 3`
===========================================================================

The certification is ``/P 2`` — §12.8.2.2 Table 254's *"filling in forms,
instantiating page templates, and signing are permitted"*.

**Not ``/P 1``.** The engine's own ``PROVENANCE.md`` for
``certified-p2-form.pdf`` records why, and it is the R162 argument — an
assertion that cannot come out false is not an assertion:

    That file is ``/P 1``, "no changes permitted", and refuses *everything*.
    ``EditSession::fill_refusal`` and ``EditSession::deletion_refusal``
    deliberately use different gates — filling takes the ``/P``-aware one,
    deletion takes the strict one — so a test written against ``/P 1`` passes
    whether or not those gates differ at all, and would keep passing if
    someone collapsed one into the other.

Concretely: ``check_certification_for_fill`` refuses **only** at
``certification_permission == Some(1)``, whereas ``structural_form_refusal``
takes ``check_certification``, whose ``forbids_structural_change()`` is
``perms_enforced && signatures > 0`` and is ``/P``-blind. At ``/P 1`` both
refuse, the Forms panel's fill controls vanish too, and a build that had
disabled the *whole panel* would be indistinguishable from a correct one.

⇒ **``/P 2`` is what puts the control inside the document**: on this one file
the fill gate is open and the structural gate is shut. A check that finds no
Delete-group arm has therefore found a *withheld* control, not an absent
panel — because the fill controls beside it are still there.

**Not ``/P 3`` either**, though for a different reason and it is worth stating
so nobody "upgrades" it later. ``/P 3`` would also refuse here — the structural
gate does not read ``/P`` at all — so it would test the same thing while no
longer matching the value every other certified fixture in this project and in
the engine's corpus carries. ``/P 2`` is additionally the ordinary real-world
case: a certified fillable form is what a bank or a government sends out.

===========================================================================
NO CRYPTOGRAPHY, AND NONE CLAIMED
===========================================================================

``/Contents`` is filler zeros and the ``/ByteRange`` is computed from the
laid-out bytes, exactly as in ``gen-certified-fixture.py`` and
``gen-signed-fixture.py``. ``pdfcer-core``'s signature module opens *"This
module verifies nothing"*, so a real certificate would exercise nothing these
bytes do not, and would drag a private key nobody can rotate into a repository.

**What this fixture can and cannot support:** it CAN support *"pdfcer found an
enforced certification, listed the grouping nodes, and withheld the
Delete-group controls while leaving the fill controls alone"* — which is the
whole of the feature. It CANNOT support any claim about signature validity, and
nothing may be written that reads as though it does.

Three structural things are needed for the census to see a certification, and
all three are here (the same triple ``gen-certified-fixture.py`` documents):

  1. a signature dictionary (``/Type /Sig`` with a computed ``/ByteRange``) —
     ``signature::census`` counts those and nothing else;
  2. a ``/Reference`` array carrying ``/TransformMethod /DocMDP``, which is what
     makes it a **certification** rather than an approval and is where
     ``certification_permission`` is read from;
  3. the catalog's ``/Perms << /DocMDP 11 0 R >>`` — the enforcement switch,
     Table 258's *"consumer applications shall enforce the permissions"*, and
     the thing ``perms_enforced`` actually reads.

===========================================================================
VERIFIED, WITH THE ENGINE, NOT BY EYE
===========================================================================

The four properties this fixture exists to have, each asserted by
``crates/pdfcer-gui/src/app/actions/forms/delete.rs``'s
``the_certified_nested_fixture_is_both_certified_and_nested``:

  1. it **loads** — ``Document::load`` succeeds and the page tree parses;
  2. ``EditSession::deletion_refusal()`` answers ``Some`` — the point;
  3. ``AcroForm::groups`` is **non-empty** (2 nodes: ``Personal.Address``,
     ``Personal`` — post-order, deepest first) — the other half, and the half
     every previous attempt at this fixture got wrong;
  4. ``EditSession::fill_refusal()`` answers ``None`` — a fill verb is still
     permitted, so the fixture can tell the two gates apart.

===========================================================================
PROVENANCE
===========================================================================

Wholly synthetic, byte-authored by this script. No downloaded PDF is involved
and no PDF library produced these bytes, so the fixture cannot inherit a bug or
a normalisation from the code it exercises.

Run from the repository root::

    python tools/gen-certified-nested-fixture.py
"""

import pathlib

OUT = pathlib.Path("fixtures/certified-nested-form.pdf")

#: Width of the ``/Contents`` placeholder, in hex digits. Even (hex pairs) and
#: CONSTANT, because the byte-range arithmetic is measured from the laid-out
#: file rather than derived from this value — see ``gen-signed-fixture.py``.
CONTENTS_HEX_DIGITS = 512

#: A4 in points, matching this repository's other generated fixtures and the
#: shell's own blank-document template, so a driven check's canvas coordinates
#: read against a page size the rest of the suite already uses.
#:
#: ★ Deliberately NOT ``nested-form.pdf``'s 300×260. That file is a core unit
#: fixture and is never opened in a window; this one is driven, and a 300-point
#: page in a 1400-point viewport is scaled far enough that a click computed from
#: a ``/Rect`` lands imprecisely.
MEDIA_BOX = "[0 0 595 842]"

#: The three terminal widgets, in `/Rect` order, top to bottom.
#:
#: Well apart (38 points of clear space between rows) and well away from the
#: signature widget below them: a driven check clicks the centre of one of
#: these, and a click that landed on a neighbour would select the wrong field
#: and report the gate as broken when what actually happened is that the click
#: missed.
ZIP_RECT = b"[60 700 300 722]"
CITY_RECT = b"[60 660 300 682]"
NAME_RECT = b"[60 620 360 642]"
#: The certification's own widget. Far below the form rows, for the same reason
#: ``gen-certified-fixture.py`` keeps its square away from its widget.
SIG_RECT = b"[60 300 300 360]"


def build() -> bytes:
    """Assemble the fixture and return its bytes.

    Object numbering, and why it reads the way it does:

    ==== ===========================================================
    obj  what
    ==== ===========================================================
    1    catalog — ``/AcroForm`` + the ``/Perms`` enforcement switch
    2    page tree
    3    the single page
    4    ``Personal``   — pure grouping node (child fields 5 and 8)
    5    ``Address``    — pure grouping node (child fields 6 and 7)
    6    ``Personal.Address.Zip``   — terminal, merged with its widget
    7    ``Personal.Address.City``  — terminal, merged with its widget
    8    ``Personal.Name``          — terminal, merged with its widget
    9    ``Certifier`` — the ``/FT /Sig`` field
    10   its widget (no ``/T``, so it is a widget kid and not a field)
    11   the signature dictionary — ``/Type /Sig``, ``/DocMDP``, ``/P 2``
    12   the page's content stream
    ==== ===========================================================

    ★ 4–8 are the same numbers, in the same order, that ``nested-form.pdf``
    uses for the same nodes. Free to keep and worth keeping: a reader diffing
    the two files against each other sees the certification and nothing else
    move.
    """
    contents = b"0" * CONTENTS_HEX_DIGITS
    # Ten digits per number is enough for any file this script produces, and a
    # constant width means overwriting the placeholder cannot change an offset.
    br_placeholder = b"[0000000000 0000000000 0000000000 0000000000]"

    objs: dict[int, bytes] = {}

    # The catalog. `/Perms << /DocMDP 11 0 R >>` is the enforcement switch —
    # `signature::census` reads `perms_enforced` from exactly this entry, and
    # `forbids_structural_change()` is `perms_enforced && signatures > 0`. The
    # `/DR` font resource is `nested-form.pdf`'s, carried over so a field whose
    # appearance is regenerated has a font to name.
    objs[1] = (
        b"<< /Type /Catalog /Pages 2 0 R /Perms << /DocMDP 11 0 R >> "
        b"/AcroForm << /Fields [4 0 R 9 0 R] /SigFlags 3 "
        b"/DA (/Helv 0 Tf 0 g) /DR << /Font << /Helv << /Type /Font "
        b"/Subtype /Type1 /BaseFont /Helvetica /Encoding /WinAnsiEncoding "
        b">> >> >> >> >>"
    )
    # ONE page, unlike `certified-comments.pdf`'s two. That file needed a page
    # to spare because a check might combine an annotation assertion with a
    # page delete; this one's subject is the field tree, every widget is on
    # page 1, and a second empty page would only give a driven check somewhere
    # wrong to look.
    objs[2] = b"<< /Type /Pages /Kids [3 0 R] /Count 1 >>"
    objs[3] = (
        b"<< /Type /Page /Parent 2 0 R /MediaBox "
        + MEDIA_BOX.encode()
        + b" /Resources << >> /Contents 12 0 R "
        b"/Annots [6 0 R 7 0 R 8 0 R 10 0 R] >>"
    )

    # ---- the field tree ------------------------------------------------
    #
    # `Personal` carries `/FT /Tx` for its subtree to inherit (Table 220: a
    # field's type may come from an ancestor) and NO `/Kids` that are widgets,
    # which is precisely what makes `walk_field` take its pure-non-terminal
    # early return and push a `FieldGroupNode`. Add a bare widget to this dict
    # and `AcroForm::groups` loses this entry — that is the mixed-kids case,
    # and it is the trap this fixture must not fall into.
    objs[4] = b"<< /T (Personal) /FT /Tx /Kids [5 0 R 8 0 R] >>"
    # `Address` is the SECOND level, and the reason the cascade is observable:
    # deleting `Personal` empties this node too, and nobody named it.
    # `/DA` here rather than on the terminals so the walk's `/DA` inheritance
    # is exercised at a different depth from `/FT`'s.
    objs[5] = (
        b"<< /T (Address) /DA (/Helv 8 Tf 0 0 1 rg) /Kids [6 0 R 7 0 R] "
        b"/Parent 4 0 R >>"
    )
    # The three terminals, each merged with its own widget (§12.5.6.19): one
    # dictionary that is both the field and the annotation. `/F 4` is Print.
    for num, partial, parent, rect, value in (
        (6, b"Zip", b"5 0 R", ZIP_RECT, b"(K7L 1A1)"),
        (7, b"City", b"5 0 R", CITY_RECT, b"(Kingston)"),
        (8, b"Name", b"4 0 R", NAME_RECT, b"(A. Operator)"),
    ):
        objs[num] = (
            b"<< /T (" + partial + b") /Parent " + parent + b" /V " + value + b" "
            b"/Type /Annot /Subtype /Widget /Rect " + rect + b" /P 3 0 R /F 4 "
            b"/MK << /BC [0 0 0] >> >>"
        )

    # ---- the certification ---------------------------------------------
    #
    # The `/Sig` FIELD, with a separate widget kid rather than merged. Separate
    # because the widget carries `/F 132` (Print | Locked) and the field
    # carries `/V`, and keeping them apart makes it obvious which entry the
    # census reads (the field's `/V`, resolved to object 11) and which the
    # canvas draws.
    #
    # ★ `Certifier` is a TERMINAL, not a grouping node: object 10 has no `/T`,
    # so `walk_field` classifies it as a widget kid and object 9 falls through
    # to the terminal arm. If it ever became a grouping node, `AcroForm::groups`
    # would gain a third entry and the driven check's node count would move.
    objs[9] = b"<< /FT /Sig /T (Certifier) /V 11 0 R /Kids [10 0 R] >>"
    objs[10] = (
        b"<< /Parent 9 0 R /Subtype /Widget /Rect " + SIG_RECT + b" /P 3 0 R /F 132 >>"
    )
    # The signature dictionary. The `/Reference` array is what the census reads
    # `certifications` and `certification_permission` from; without it the same
    # `/Type /Sig` is an ordinary approval signature and every gate stays open,
    # which is `fixtures/signed-two-pages.pdf`'s subject and deliberately a
    # different fixture.
    #
    # `/P 2` is written explicitly even though Table 254 makes `/P` optional
    # with default 2. Absence lands in the same place, and writing it is what
    # makes the fixture SAY what it is testing — a reader should not have to
    # know a default to know why the file refuses. See the module header for
    # why 2 and not 1 or 3.
    objs[11] = (
        b"<< /Type /Sig /Filter /Adobe.PPKLite "
        b"/SubFilter /adbe.pkcs7.detached "
        b"/Name (A. Certifier) /M (D:20260829120000Z) "
        b"/Reference [ << /Type /SigRef /TransformMethod /DocMDP "
        b"/TransformParams << /Type /TransformParams /P 2 /V /1.2 >> >> ] "
        b"/ByteRange " + br_placeholder + b" /Contents <" + contents + b"> >>"
    )

    # A stroked box per widget rectangle, so the page is not blank and a human
    # looking at a failed run can see the form the check is talking about. The
    # boxes are drawn UNDER the widgets, at the same rectangles, which is what
    # a flattened form would look like.
    stream = (
        b"0.5 w 0 0 0 RG\n"
        b"60 700 240 22 re S\n"
        b"60 660 240 22 re S\n"
        b"60 620 300 22 re S\n"
        b"60 300 240 60 re S\n"
    )
    objs[12] = b"<< /Length %d >>\nstream\n" % len(stream) + stream + b"endstream"

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
    """Write the fixture."""
    data = build()
    OUT.parent.mkdir(parents=True, exist_ok=True)
    OUT.write_bytes(data)
    print(
        f"wrote {OUT} {len(data)} bytes  certified=yes /P=2  pages=1  "
        f"groups=Personal,Personal.Address  "
        f"terminals=Personal.Name,Personal.Address.City,Personal.Address.Zip,Certifier"
    )


if __name__ == "__main__":
    main()
