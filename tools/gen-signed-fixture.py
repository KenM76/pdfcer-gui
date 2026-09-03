#!/usr/bin/env python3
"""Generate ``fixtures/signed-two-pages.pdf`` — the fixture the signature
warning needs.

===========================================================================
WHY THIS FILE EXISTS
===========================================================================

``crate::dialogs::signature`` puts a window in front of any save that
``pdfcer-core`` reports as invalidating a digital signature. R1 says a feature
is not done until it has been asserted by **driving the running binary**, and
``tools/ui-verify``'s ``signature_save`` check does that: it launches on a
signed document, deletes a page, presses Save-a-copy, and asserts that the
window appeared and held the write.

Every one of those steps needs a document that is:

  1. **actually signed** — a signature dictionary with a ``/Type /Sig`` and a
     real ``/ByteRange``, because ``signature::census`` counts those and
     nothing else. An AcroForm ``/SigFlags`` declaration is NOT a signature,
     deliberately: the engine's census documents that an unsigned form which
     merely announces it expects signatures *"must not make pdfcer warn about
     destroying something that does not exist"*;
  2. **signed by an APPROVAL signature, not a certification one** — the
     signature carries no ``/Reference``, so ``census.certifications`` is 0 and
     ``SignatureImpact::documentation_basis`` answers ``ConservativeReport``.
     That is the arm whose copy is the hardest to get right (ISO 32000-1 is
     silent, and pdfcer reports the cautious answer under rule 4), so it is the
     arm the driven check should exercise;
  3. **more than one page** — the check makes the save structural by deleting a
     page, and ``pages.delete`` over the only page of a one-page document is a
     refusal rather than an edit.

===========================================================================
WHY NOT THE ENGINE'S OWN SIGNATURE FIXTURES
===========================================================================

``D:\\Dev\\pdfcer\\fixtures\\synthetic\\signature\\`` has three, and they were
read before this was written. All three are **one page**, which fails (3)
above: they were built for ``signature::byte_range_coverage``, which is
arithmetic over byte offsets and needs no pages at all.

This one is therefore the engine's ``signed-full-coverage.pdf`` with a second
page — same shape, same technique, same absence of cryptography — rather than
a new idea. Where the two agree they agree deliberately.

===========================================================================
THE ``/ByteRange`` IS COMPUTED, NOT WRITTEN
===========================================================================

The offsets are positions in the finished file, so the file is laid out ONCE
with a fixed-width placeholder, the real ``/Contents`` hole is measured from
that layout, and the placeholder is overwritten in place — which is the order a
real signer works in (§12.8.3.3), and the reason the placeholder is padded to a
constant width: overwriting it must not move a single byte.

Round numbers would let an off-by-one in the straddle arithmetic pass
unnoticed. That lesson is the engine's; it is repeated here because this script
is standalone and a reader of it will not have the other one open.

===========================================================================
NO CRYPTOGRAPHY, AND NONE CLAIMED
===========================================================================

``/Contents`` is filler zeros. Nothing in pdfcer inspects a signature value —
``pdfcer-core``'s signature module opens *"This module verifies nothing"* — so a
fixture with a real certificate would test nothing this one does not, and would
drag a private key into a repository.

**What this fixture can and cannot support:**

  * it CAN support "pdfcer found a signature and said the right thing about what
    a save does to it", which is the whole of the feature;
  * it CANNOT support any claim about validity, and nothing should be written
    that reads as though it does.

===========================================================================
PROVENANCE
===========================================================================

Wholly synthetic, byte-authored by this script. No downloaded PDF is involved
and no PDF library produced these bytes, so the fixture cannot inherit a bug or
a normalisation from the code it exercises.

Run from the repository root::

    python tools/gen-signed-fixture.py
"""

import pathlib

OUT = pathlib.Path("fixtures/signed-two-pages.pdf")

#: Width of the ``/Contents`` placeholder, in hex digits.
#:
#: A real signer reserves a fixed span before knowing the signature's length.
#: The exact number is arbitrary, but it must be even (hex pairs) and it must be
#: a CONSTANT, because the byte-range arithmetic below is measured from the
#: laid-out file rather than derived from this value.
CONTENTS_HEX_DIGITS = 512

#: A4 in points, matching the shell's own blank-document template so the check's
#: canvas coordinates read against a page size the rest of the suite already
#: uses.
MEDIA_BOX = "[0 0 595 842]"


def build() -> None:
    """Assemble the fixture and write it to :data:`OUT`."""
    contents = b"0" * CONTENTS_HEX_DIGITS
    # Ten digits per number is enough for any file this script produces, and a
    # constant width means overwriting the placeholder cannot change an offset.
    br_placeholder = b"[0000000000 0000000000 0000000000 0000000000]"

    objs: dict[int, bytes] = {}
    objs[1] = (
        b"<< /Type /Catalog /Pages 2 0 R /AcroForm << /Fields [10 0 R] "
        b"/SigFlags 3 >> >>"
    )
    # TWO pages. See the header's point (3): the driven check makes the save
    # structural by deleting one, and a one-page document has no page it can
    # spare.
    objs[2] = b"<< /Type /Pages /Kids [3 0 R 4 0 R] /Count 2 >>"
    objs[3] = (
        b"<< /Type /Page /Parent 2 0 R /MediaBox " + MEDIA_BOX.encode()
        + b" /Resources << >> /Contents 5 0 R /Annots [11 0 R] >>"
    )
    objs[4] = (
        b"<< /Type /Page /Parent 2 0 R /MediaBox " + MEDIA_BOX.encode()
        + b" /Resources << >> /Contents 6 0 R >>"
    )
    # A stroked rectangle per page, so the canvas has something to draw and a
    # human looking at a failed run can see which page is which. Page 1 gets a
    # tall box, page 2 a wide one.
    for num, rect in ((5, b"80 500 200 250"), (6, b"80 300 400 120")):
        stream = b"1 w 0 0 0 RG " + rect + b" re S\n"
        objs[num] = (
            b"<< /Length %d >>\nstream\n" % len(stream) + stream + b"endstream"
        )
    # The signature FIELD, merged with its widget (§12.5.6.19), on page 1.
    # `/V` points at the signature dictionary, which is the shape
    # `signature::census` reads.
    objs[10] = b"<< /FT /Sig /T (Approval) /V 12 0 R /Kids [11 0 R] >>"
    objs[11] = (
        b"<< /Parent 10 0 R /Subtype /Widget /Rect [60 60 300 120] "
        b"/P 3 0 R /F 132 >>"
    )
    # ★ NO `/Reference`. That is what makes this an APPROVAL signature rather
    # than a certification one, and it is the whole point of the fixture: the
    # census counts `certifications` from `/Reference`, never from `/Perms`, so
    # a signature without one lands on `ImpactBasis::ConservativeReport`.
    objs[12] = (
        b"<< /Type /Sig /Filter /Adobe.PPKLite "
        b"/SubFilter /adbe.pkcs7.detached "
        b"/Name (A. Signer) /M (D:20260828120000Z) "
        b"/ByteRange " + br_placeholder + b" /Contents <" + contents + b"> >>"
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

    OUT.parent.mkdir(parents=True, exist_ok=True)
    OUT.write_bytes(bytes(buf))
    print(
        f"wrote {OUT} {len(buf)} bytes  "
        f"ByteRange={real.decode().strip()}  pages=2  certifications=0"
    )


if __name__ == "__main__":
    build()
