# -*- coding: utf-8 -*-
"""Build `fixtures/contradicts-itself.pdf` — a document whose catalog names the
same key twice, with two different values.

## Why this fixture exists

Engine decision 145 and `Pass 283.0`. The operator's own 46 KB drawing opens in
Acrobat and was **refused whole** by pdfcer, because its document catalog
carries `/PageMode` twice:

    … /PageMode /UseOC /PageLayout /SinglePage … /PageMode /UseOutlines …

One repeated key in one object cost the whole document, on a file every other
reader opens without comment. The engine now keeps one value, counts the
choice, and hands it out as `Document::load_anomalies()`; this shell discloses
it in the status bar and in Document properties
(`crate::app::status::anomalies`).

★★★ **Nothing else in `fixtures/` can reach that code path.** Every other file
in this directory is well-formed enough that `load_anomalies()` returns empty,
so a check driven against one of them would assert the absence of a disclosure
against a document that has nothing to disclose — a check that cannot fail,
which is the exact shape of report this project has filed by accident more than
once. This file is here so the disclosure can be driven rather than argued.

## What this file contains

    1  Catalog  -> /PageMode /UseOC  … /PageMode /UseOutlines   ★ THE POINT
    2  Pages
    3  Page     -> one line of text so the page is not blank
    4  Contents stream
    5  Font     Helvetica

★ **The two `/PageMode` values are DIFFERENT on purpose.** A doubled key whose
two values were equal would still be a §7.3.7 violation, but the disclosure's
whole subject is *what pdfcer chose between* — the panel row reads
"pdfcer kept /UseOutlines and left /UseOC" — and a fixture with equal values
would let a build that printed the kept value twice pass. The two values are
also the operator's own two values, so the row this fixture produces is the row
he would see.

★ **`/UseOutlines` is second and therefore the one KEPT**, under the engine's
default `DuplicateKeyPolicy::KeepLast`. That default is sourced, not assumed:
`qpdf`, `pdf.js` and `pdfium` all keep the last occurrence and none of the
three refuses the file. If a future engine revision changed the winner, this
fixture would keep working and the assertion on WHICH value was kept would
correctly go red — which is the behaviour wanted.

⚠ **The xref is sound and every offset is correct.** That is deliberate and it
is the second half of what this fixture proves: `Document::recovery()` and
`Document::load_anomalies()` are DISJOINT questions — recovery is about the
cross-reference machinery, anomalies are about the objects — and the operator's
file lights the second and not the first. A fixture with a broken xref would
light the rebuild disclosure instead and the check would pass for the wrong
reason.

⚠ **Exactly ONE anomaly, deliberately.** A stream with an unusable `/Length`
would be a second one, but `StreamLengthPolicy::RecoverFromEndstream` is
reachable only from the rebuild-by-scan recovery path — on a file with a sound
xref a bad `/Length` is refused, not recovered — so a fixture that tried for
two would either not produce the second or would drag in the recovery path this
one is careful to avoid. The census clause for a single anomaly is what a
single anomaly should produce.

## Rebuilding

    python fixtures/contradicts-itself.PROVENANCE.py

Offsets are computed, so edits are safe.
"""
import io

CONTENT = b"BT /Helv 18 Tf 40 120 Td (This file names /PageMode twice.) Tj ET\n"

objects = [
    # 1 - catalog, with /PageMode TWICE and two different values.
    #     Written in the operator's own order: /UseOC first, /UseOutlines last.
    b"<< /Type /Catalog /Pages 2 0 R /PageMode /UseOC /PageLayout /SinglePage "
    b"/PageMode /UseOutlines >>",
    # 2 - page tree
    b"<< /Type /Pages /Kids [3 0 R] /Count 1 >>",
    # 3 - page
    b"<< /Type /Page /Parent 2 0 R /MediaBox [0 0 400 200] "
    b"/Resources << /Font << /Helv 5 0 R >> >> /Contents 4 0 R >>",
    # 4 - contents (length is CORRECT; see the header on why)
    b"<< /Length %d >>\nstream\n" % len(CONTENT) + CONTENT + b"endstream",
    # 5 - font
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

path = "fixtures/contradicts-itself.pdf"
io.open(path, "wb").write(bytes(out))
print("wrote", path, len(out), "bytes")
