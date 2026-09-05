#!/usr/bin/env python3
"""Generate ``fixtures/nested-page-tree.pdf`` — twelve pages under a THREE-LEVEL
page tree.

===========================================================================
★★★ THE NESTING IS THE POINT. A FLAT REPLACEMENT MAKES EVERY ASSERTION THAT
    USES THIS FIXTURE UNFALSIFIABLE.
===========================================================================

Read this paragraph before changing anything below, and before "simplifying"
this generator into the four-line loop that every other fixture in this
directory uses.

ISO 32000-1 §7.7.3.2 gives a page tree two independent descriptions of how many
pages it has: the ``/Kids`` structure, walked to the ``/Page`` leaves, and the
``/Count`` on every ``/Pages`` node — *"the number of leaf nodes that are
descendants of this node"*. A well-formed file has them agree at every node.

On a **flat** tree — one root whose ``/Kids`` are all leaves — the immediate
parent of every page *is* the root. A writer that removes a page and updates
only the immediate parent therefore updates the root by accident, and the file
comes out correct. **The defect cannot occur.** That is not a hypothesis: it
was measured on 2026-09-05 against ``fixtures/four-pages.pdf`` and the output
was clean, ``/Kids [3 0 R 6 0 R] /Count 2``, perfect.

The operator's own file is not flat:

    "I tested deleting pages from a pdf. when I open the document in Acrobat
     there are blank pages at the end of the document equalling the number of
     pages I deleted."

``SW41177.pdf`` — a SolidWorks drawing set — has a two-level tree: a root with
seven ``/Pages`` children of five or six pages each. ``delete-pages --pages
2,3`` rewrote **exactly one object**, the immediate parent, and left the root
declaring 36 pages over 34 reachable ones. Acrobat builds its page list from
the root ``/Count`` and showed 36 pages with the last two blank. Every
synthetic fixture in either corpus had a one-level tree, which is why the
defect was invisible to the whole test suite and visible on every document he
actually works with.

⇒ A test that used a flat document to assert *"every ancestor agrees"* would
**pass against a build whose walk never goes upward at all.** The property
under test is only expressible on a tree with an ancestor above the parent.
So: if you flatten this fixture, or regenerate it with a helper that emits a
single ``/Pages`` node, you have not simplified a test — you have deleted it
while leaving it green.

===========================================================================
WHY THREE LEVELS AND NOT TWO
===========================================================================

Two levels (the operator's own shape) proves that *something* above the
immediate parent is missed. It cannot distinguish two different bugs:

  (a) the writer updates only the immediate parent — no upward walk at all;
  (b) the writer walks up but stops one short of the root.

Both produce an identical file on a two-level tree, because there the parent's
parent *is* the root. Three levels separates them: delete one page from ``A1``
and a build with bug (a) leaves both ``A`` and the root stale, while a build
with bug (b) leaves only the root stale. The fixture is therefore diagnostic
and not merely detective, at the cost of two extra objects.

Measured against ``pdfcer.exe`` v0.38.0 (``b01964f``) it reports **bug (a)**:
after ``delete-pages --pages 2``, node ``A1`` is correct (``/Count 3`` ->
``/Count 2``) and **both** ``A`` and the root are unchanged, still declaring 6
and 12 over 5 and 11 reachable leaves. The appended revision defines exactly
one object. There is no upward walk at all — the ancestor chain is not walked
short, it is not walked.

``page-copy --pages 2 --cut`` produces byte-for-byte the same corruption
(``changed=3 objects=1 appended=234``), which is what establishes that the
defect belongs to a shared removal path and not to one CLI verb.

===========================================================================
THE SHAPE, EXACTLY
===========================================================================

    root      /Count 12   /Kids [A, B]                    <- catalog /Pages
      A       /Count  6   /Kids [A1, A2]
        A1    /Count  3   /Kids [p1, p2, p3]
        A2    /Count  3   /Kids [p4, p5, p6]
      B       /Count  6   /Kids [B1, B2]
        B1    /Count  3   /Kids [p7, p8, p9]
        B2    /Count  3   /Kids [p10, p11, p12]

Seven ``/Pages`` nodes, twelve ``/Page`` leaves, three levels of node above a
leaf. Every node carries ``/Count`` and ``/Parent`` as §7.7.3.2 Table 29
requires, because a guard that reads ``/Count`` must be given a file where
``/Count`` is present and correct to begin with — otherwise a passing "clean"
verdict is indistinguishable from a walk that found nothing to read.

**Twelve pages, not four.** Deleting two pages has to be able to leave an
ancestor stale *and* leave the tree obviously non-degenerate: with three pages
per bottom node there is a page to delete on either side of the one removed, so
a subtree never empties by accident and the leaf tallies stay distinguishable
from the node count.

===========================================================================
WHAT IS DRAWN ON EACH PAGE, AND WHY IT MATTERS
===========================================================================

Each page carries its own number in 96 pt Helvetica, centred, plus a small
line naming the node path it hangs from (``A / A1``). Two reasons, both about
falsification rather than decoration:

  * **A blank page and a page that failed to render are indistinguishable.**
    The symptom being reproduced is literally *blank pages at the end*, so a
    fixture of blank pages could not tell the defect from itself. With a number
    on every page, a reader that shows a blank page is showing a page that is
    not in the file.

  * **Page identity survives a reorder.** After a delete the sixth page is the
    one that used to be seventh; a check that asserts *which* pages survived
    needs the page to say so itself, not to be identified by position.

Standard-14 Helvetica, not an embedded font: the fixture must stay small and
must not make any assertion about this document depend on font embedding, which
is a separate feature with its own licence decision attached.

===========================================================================
USAGE
===========================================================================

    python tools/gen-nested-page-tree-fixture.py

Writes ``fixtures/nested-page-tree.pdf`` relative to the repository root
(derived from this file's location, so it does not matter where you run it
from). Deterministic: same bytes every run, so regenerating it produces no git
diff unless this generator changed.

To see the defect it exists for:

    pdfcer.exe delete-pages --pages 2 -o out.pdf fixtures/nested-page-tree.pdf
    pdfcer.exe dump-object --id 1 out.pdf     # root  -> /Count 12 over 11, STALE
    pdfcer.exe dump-object --id 3 out.pdf     # A     -> /Count  6 over  5, STALE
    pdfcer.exe dump-object --id 4 out.pdf     # A1    -> /Count  2 over  2, correct

The generator prints the id of every node when it runs, so the three numbers
above do not have to be trusted from this docstring.
"""

import pathlib
import sys

# --------------------------------------------------------------------------
# The tree, as data. The object numbering is fixed and is referenced by the
# generator's own docstring above and by the fixture's consumers, so it is
# written out explicitly rather than derived from an enumerate().
#
#   1  root      6  B1        10..21  the twelve /Page leaves
#   2  A         7  B2        22      the Helvetica font
#   3  catalog   4  A1
#   5  A2        8  (unused)  9  (unused)
#
# ...except that leaving holes in the numbering would make `list-objects`
# report free entries and invite the question "what was deleted?". So the ids
# are assigned in one pass below and the map is printed at the end.
# --------------------------------------------------------------------------

PAGE_W, PAGE_H = 612, 792  # US Letter, in points

# (node name, parent name or None, [child node names] or [page numbers])
TREE = [
    ("root", None, ["A", "B"]),
    ("A", "root", ["A1", "A2"]),
    ("A1", "A", [1, 2, 3]),
    ("A2", "A", [4, 5, 6]),
    ("B", "root", ["B1", "B2"]),
    ("B1", "B", [7, 8, 9]),
    ("B2", "B", [10, 11, 12]),
]

PATH_OF = {1: "A / A1", 2: "A / A1", 3: "A / A1",
           4: "A / A2", 5: "A / A2", 6: "A / A2",
           7: "B / B1", 8: "B / B1", 9: "B / B1",
           10: "B / B2", 11: "B / B2", 12: "B / B2"}


def content_stream(page_no):
    """The page's own content: its number, big, plus the node path beneath it.

    Hand-written operators rather than a library, for the reason every
    generator in this directory gives: the whole file has to be readable in a
    text editor when a check disagrees with it, and a produced-by-a-library
    fixture is a second thing to debug.

    Widths are eyeballed rather than measured from the AFM — the text is
    centred well enough to be read, and nothing asserts its position. If
    something ever does, measure it properly rather than tightening these.
    """
    label = str(page_no)
    # 96 pt Helvetica digits are ~53 pt wide each; centre on that estimate.
    x = (PAGE_W - 53 * len(label)) / 2
    return (
        "BT /F1 96 Tf 1 0 0 1 %.1f 400 Tm (%s) Tj ET\n"
        "BT /F1 14 Tf 1 0 0 1 %.1f 340 Tm (page %s of 12  -  under %s) Tj ET\n"
        % (x, label, 190, page_no, PATH_OF[page_no])
    )


def build():
    """Assemble the whole file and return its bytes.

    A classic cross-reference **table** (§7.5.4), not a cross-reference stream,
    and no object streams: the fixture's subject is the page tree, and the
    single most useful property it can have when an assertion about it fails is
    that ``/Count`` and ``/Kids`` are greppable in a hex editor without
    inflating anything.
    """
    objects = {}  # id -> serialized body (without "N 0 obj"/"endobj")
    ids = {}
    next_id = 1

    def claim(name):
        nonlocal next_id
        ids[name] = next_id
        next_id += 1
        return ids[name]

    # Order chosen so the root is object 1 and the catalog object 2 — the two
    # a reader dumps first when a page tree is under suspicion.
    claim("root")
    claim("catalog")
    for node, _parent, _kids in TREE:
        if node != "root":
            claim(node)
    for page_no in range(1, 13):
        claim("page%d" % page_no)
        claim("content%d" % page_no)
    claim("font")

    # --- the /Pages nodes -------------------------------------------------
    for node, parent, kids in TREE:
        kid_refs = " ".join(
            "%d 0 R" % ids["page%d" % k] if isinstance(k, int) else "%d 0 R" % ids[k]
            for k in kids
        )
        # /Count is the number of LEAVES beneath the node, not the number of
        # kids. They differ at every node above the bottom level, and that
        # difference is the entire reason this fixture is nested — a generator
        # that wrote len(kids) here would produce a file that is already
        # inconsistent and would make the guard fire on a clean save.
        leaves = 12 if node == "root" else (6 if node in ("A", "B") else 3)
        entries = ["/Type /Pages", "/Kids [%s]" % kid_refs, "/Count %d" % leaves]
        if parent is not None:
            entries.append("/Parent %d 0 R" % ids[parent])
        objects[ids[node]] = "<< %s >>" % " ".join(entries)

    # --- the catalog ------------------------------------------------------
    objects[ids["catalog"]] = "<< /Type /Catalog /Pages %d 0 R >>" % ids["root"]

    # --- the leaves and their content -------------------------------------
    for page_no in range(1, 13):
        parent = next(n for n, _p, kids in TREE if page_no in kids)
        stream = content_stream(page_no).encode("ascii")
        objects[ids["content%d" % page_no]] = (
            "<< /Length %d >>\nstream\n" % len(stream)
        ) + stream.decode("ascii") + "endstream"
        objects[ids["page%d" % page_no]] = (
            "<< /Type /Page /Parent %d 0 R /MediaBox [0 0 %d %d] "
            "/Resources << /Font << /F1 %d 0 R >> >> /Contents %d 0 R >>"
            % (ids[parent], PAGE_W, PAGE_H, ids["font"], ids["content%d" % page_no])
        )

    objects[ids["font"]] = (
        "<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica "
        "/Encoding /WinAnsiEncoding >>"
    )

    # --- serialize --------------------------------------------------------
    out = bytearray(b"%PDF-1.7\n%\xe2\xe3\xcf\xd3\n")
    offsets = {}
    for oid in sorted(objects):
        offsets[oid] = len(out)
        out += ("%d 0 obj\n" % oid).encode("ascii")
        out += objects[oid].encode("ascii")
        out += b"\nendobj\n"

    startxref = len(out)
    count = max(objects) + 1
    out += ("xref\n0 %d\n" % count).encode("ascii")
    out += b"0000000000 65535 f \n"
    for oid in range(1, count):
        out += ("%010d 00000 n \n" % offsets[oid]).encode("ascii")
    out += (
        "trailer\n<< /Size %d /Root %d 0 R >>\nstartxref\n%d\n%%%%EOF\n"
        % (count, ids["catalog"], startxref)
    ).encode("ascii")
    return bytes(out), ids


def main():
    data, ids = build()
    root = pathlib.Path(__file__).resolve().parent.parent
    target = root / "fixtures" / "nested-page-tree.pdf"
    target.parent.mkdir(parents=True, exist_ok=True)
    target.write_bytes(data)
    print("wrote %s (%d bytes)" % (target, len(data)))
    print("object ids:")
    for name in ("root", "A", "A1", "A2", "B", "B1", "B2"):
        print("  %-5s = %d" % (name, ids[name]))
    return 0


if __name__ == "__main__":
    sys.exit(main())
