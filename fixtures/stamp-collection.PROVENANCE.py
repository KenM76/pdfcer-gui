# -*- coding: utf-8 -*-
"""Build `fixtures/stamp-collection.pdf` — an Acrobat custom-stamp collection,
in the shape Adobe's own files have.

## Why this fixture exists

`OPERATOR_REQUESTS.md` **O169**: *"if acrobat has a way of adding custom stamps
or text, we need the same feature too with the same import/export"*. The finding
that shaped the feature is that **there is no interchange format to match** — a
stamp collection simply IS an ordinary PDF, one file per category, one page per
stamp, the category in `/Info` `/Title` and the names in the catalog's `/Names`
-> `/Pages` name tree. Handing somebody the file is the export; `file.open` is
the import.

Which means an operator can open one, and without the Document properties
section this fixture exists to drive, pdfcer shows him a document of unrelated
pictures with nothing at all to say that it is anything else.

*** NOTHING ELSE IN `fixtures/` CAN REACH THAT CODE PATH. *** Every other file
in this directory has no `/Names` -> `/Pages` tree, so `stamp_file::read`
returns an empty collection and `is_collection` is false. A check driven against
one of them would be asserting the absence of a disclosure on a document that
has nothing to disclose — a check that cannot fail, which is a shape this
project has filed by accident more than once. Both halves need this file: the
presence assertion runs against it, and the absence assertion runs against
`four-pages.pdf` as its control.

## What this file contains

    1   Catalog   -> /Pages 2 0 R  /Names << /Pages 10 0 R >>   *** THE POINT
    2   Page tree -> three kids
    3   Page      "Approved"       (page 1)
    4   Page      "For Review"     (page 2)
    5   Page      "Issued"         (page 3)
    6-8 Contents  one per page
    9   Font      Helvetica
    10  Name tree flat /Names array, three entries
    11  Info      /Title (Site Review)                          *** THE CATEGORY

## The three deliberate properties, each of which a check reads

*** 1. THE TREE ORDER IS NOT THE PAGE ORDER, and that is the whole reason the
Document properties rows are indexed by tree position rather than by page. A
name tree is sorted lexicographically by byte (S7.9.6), so `#SRIssued` — `#` is
0x23, `S` is 0x53 — sorts FIRST while its page is LAST. A build that quietly
enumerated pages instead of tree entries would produce a list that looks
plausible, is in the wrong order, and matches on this fixture only if the
fixture were sorted the lazy way. It is not.

*** 2. EXACTLY ONE STAMP IS DYNAMIC. `#SRIssued` carries the `#` prefix Acrobat
uses for a stamp whose text it recomputes at placement time. One-of-three is
chosen so the count sentence exercises the middle branch — "3 stamps, 1 of them
dynamic" — rather than the two easy ends. The operator's own signature
collection is entirely dynamic and would only ever produce the "all of them"
branch.

*** 3. EVERY NAME RESOLVES TO A PAGE OF THIS DOCUMENT. Deliberately: pdfcer
currently cannot distinguish "this stamp names a page that is not here" from
"this document's page tree could not be read at all", because `stamp_file::read`
swallows the page-tree failure and then reports every stamp as pointing at
nothing. That is filed as an engine request. Until it is answered, a fixture
with a dangling name would be pinning an ambiguity rather than a behaviour, and
the sentence it produces is pinned by a unit test on the string instead.

## What is NOT here, and why

No `/Limits`, and no `/Kids`: the tree is a single flat node, which is what
Adobe ships (its largest collection is twelve stamps in one node) and what the
engine's reader walks first. A nested tree would test the engine's recursion
rather than this shell's disclosure, and the engine tests that itself.

## Rebuilding

    python fixtures/stamp-collection.PROVENANCE.py

Offsets are computed, so edits are safe.
"""
import io


def content(label):
    """One stamp page: a box and its word, so the page is not blank.

    A blank page would still be a valid stamp, but a driven check that captures
    the window has nothing to look at, and an operator opening the fixture to
    see what the harness saw would learn nothing from three empty sheets.
    """
    return (
        b"0.2 0.35 0.6 RG 2 w 6 6 188 48 re S\n"
        b"BT /Helv 22 Tf 20 22 Td (" + label + b") Tj ET\n"
    )


PAGES = [b"Approved", b"For Review", b"Issued"]

objects = [
    # 1 - catalog. /Names -> /Pages is the ONLY test for "is this a stamp
    #     collection"; a /Title without this is just a PDF with a title.
    b"<< /Type /Catalog /Pages 2 0 R /Names << /Pages 10 0 R >> >>",
    # 2 - page tree
    b"<< /Type /Pages /Kids [3 0 R 4 0 R 5 0 R] /Count 3 >>",
]

for i, label in enumerate(PAGES):
    objects.append(
        b"<< /Type /Page /Parent 2 0 R /MediaBox [0 0 200 60] "
        b"/Resources << /Font << /Helv 9 0 R >> >> /Contents %d 0 R >>"
        % (6 + i)
    )

for label in PAGES:
    body = content(label)
    objects.append(b"<< /Length %d >>\nstream\n" % len(body) + body + b"endstream")

objects.append(
    b"<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica "
    b"/Encoding /WinAnsiEncoding >>"
)

# 10 - the name tree. Sorted by byte value, which puts the dynamic stamp first
#      and its page last. See property 1 in the header.
objects.append(
    b"<< /Names [ (#SRIssued=Issued) 5 0 R "
    b"(SRApproved=Approved) 3 0 R "
    b"(SRForReview=For Review) 4 0 R ] >>"
)

# 11 - the category, which is nothing more exotic than the document title.
objects.append(b"<< /Title (Site Review) >>")

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
out += (
    b"trailer\n<< /Size %d /Root 1 0 R /Info 11 0 R >>\nstartxref\n%d\n%%%%EOF\n"
    % (n, xref_at)
)

path = "fixtures/stamp-collection.pdf"
io.open(path, "wb").write(bytes(out))
print("wrote", path, len(out), "bytes")
