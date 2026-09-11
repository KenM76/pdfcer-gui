# -*- coding: utf-8 -*-
"""Build `fixtures/layered-drawing.pdf` — a CAD-shaped drawing with six
optional-content groups, one of them off by default.

## Why this fixture exists

`tools/ui-verify/src/checks/layers_search.rs` —
`layers_search_narrows_the_list`, which covers `OPERATOR_REQUESTS.md` **O126**
(*"there is a search to implement on the layers"*) — **SKIPPED on every run of
the driven suite up to and including the full sweep of 2026-09-11**, with this
message:

    the Layers panel drew no rows, so this check could not run. Either the
    panel is not on screen after `view.panel_layers`, or the --pdf has no
    optional content. Give it a layered drawing.

That message is exactly right and it had nowhere to point: **not one file in
`fixtures/` carried an `/OCProperties` entry before this one.** So the check
was registered, compiled, ran, and asserted nothing — the shape of failure
this project has named more than once as the worst one available, because a
SKIP is not red and a check can therefore stop running without saying so in a
colour anybody notices.

⇒ This file exists so that check can be red or green rather than absent.

## The two thresholds it has to clear, and why six layers rather than two

`crates/pdfcer-gui/src/panels/layers/search.rs:120` sets
`MIN_LAYERS_FOR_SEARCH = 2`: the panel **deliberately does not draw the search
field** for a document with one layer, because a search over one row can only
remove the row. The check knows that and treats a one-layer fixture as an
ERROR about the fixture rather than a failure of the program.

So two layers would clear the threshold. Six are here instead, for a reason
that outlives this check:

★ **A fixture that only just clears a threshold cannot tell "narrowed" from
"emptied".** The predicate's whole subject is a query that leaves *some* rows
and removes *others*. With two layers every non-trivial query leaves 0 or 1,
and 1 is also what a broken predicate returning "the first row" produces. With
six, the names below make `dim` leave exactly two and remove four — an outcome
no degenerate implementation reaches by accident.

## The names are chosen, not decorative

| # | `/Name` | on by default | why this name |
|---|---|---|---|
| 1 | `Title Block` | yes | the CAD sheet's furniture; the one layer an operator never hides |
| 2 | `Grid Lines` | yes | shares no substring with any other name — the control for a query that should match exactly one |
| 3 | `Dimensions` | yes | ★ `dim`, case-insensitively, must leave this and #4 and nothing else |
| 4 | `Dimensions Reference` | yes | ★ the second `dim` match, and a **prefix extension** of #3 — so a predicate that anchored on equality instead of substring leaves one row and is caught |
| 5 | `Electrical` | **no** | ★ see below |
| 6 | `Notes` | yes | short name; guards against a predicate that assumed a minimum length |

★★ **`Electrical` is OFF in the default configuration**, i.e. it is named in
`/D << /OFF [...] >>`. Two reasons, and the second is the one that matters:

1. It is what a real layered CAD export looks like. A drawing with every layer
   visible is the uncommon case.
2. **The Layers panel's visibility column has to have something to show.** A
   fixture where every row reads the same state cannot tell a panel that reads
   the document's configuration from one that draws `ON` unconditionally —
   both look identical, and the second is a real defect that ships silently.

⚠ **`Electrical` still paints content.** An OCG named in `/OFF` with no marked
content behind it is invisible in exactly the way a bug is: the row would be
present, the toggle would appear to work, and nothing would ever change on the
page. Its content is a distinct red rectangle low on the sheet, so a toggle is
verifiable by eye and by a pixel check.

## What this file contains

    1  Catalog   -> /OCProperties  ★ THE POINT
    2  Pages
    3  Page      -> /Resources /Properties << /OC1 .. /OC6 >>
    4  Contents  -> six /OC /OCn BDC ... EMC spans
    5  Font      Helvetica
    6..11  the six /OCG dictionaries, in the order of the table above

The page is **2384 x 1684 pt** — ISO A1 landscape, the size of the operator's
own sheets — rather than a convenient small box. A layers panel is a CAD
surface and a fixture that is a postcard exercises none of the geometry that
makes one hard.

## What is deliberately NOT here

**No `/AS` (auto states) and no `/Usage` dictionaries.** Those make an OCG's
visibility depend on the *context* — printing vs viewing vs zoom range — and a
panel that reads them correctly and a panel that ignores them differ only on a
document that has them. That is a real and separate capability, and it wants
its own fixture whose whole subject is that difference; folding it in here
would mean a SKIP on this check could be caused by either thing.

**No nested `/OCMD`.** Same argument: a membership dictionary with `/P /AllOn`
over two groups is a distinct capability
(`tools/ui-verify/src/checks/layers_membership.rs` already exists for it) and
it should fail on its own fixture, not silently change what this one measures.

## Rebuilding

    python fixtures/layered-drawing.PROVENANCE.py

Offsets are computed, so edits are safe.
"""

import io

# ---------------------------------------------------------------------------
# The six layers. `on` is the DEFAULT configuration's state, written into
# `/D << /ON [...] /OFF [...] >>` below.
#
# `paint` is the content-stream fragment drawn inside this layer's
# `/OC /OCn BDC ... EMC` span. Each one paints something visually distinct, so
# a toggle can be confirmed by a human or by a pixel check rather than only by
# a trace line.
# ---------------------------------------------------------------------------
LAYERS = [
    # (name, on-by-default, content fragment)
    (
        b"Title Block",
        True,
        # A border and a title-block box bottom-right, in black.
        b"0 0 0 RG 3 w 40 40 2304 1604 re S\n"
        b"1.5 w 1684 40 660 300 re S\n"
        b"BT /Helv 28 Tf 1710 250 Td (LAYERED TEST SHEET) Tj ET\n"
        b"BT /Helv 18 Tf 1710 200 Td (fixtures/layered-drawing.pdf) Tj ET\n",
    ),
    (
        b"Grid Lines",
        True,
        # A light grid across the sheet. Deliberately the only name in the set
        # sharing no substring with another.
        b"0.8 0.8 0.8 RG 0.5 w\n"
        + b"".join(
            b"%d 40 m %d 1644 l S\n" % (x, x) for x in range(140, 2340, 200)
        )
        + b"".join(
            b"40 %d m 2344 %d l S\n" % (y, y) for y in range(140, 1640, 200)
        ),
    ),
    (
        b"Dimensions",
        True,
        # Blue witness lines and a dimension run.
        b"0 0 0.8 RG 1 w 300 1200 m 300 1350 l S 900 1200 m 900 1350 l S\n"
        b"300 1300 m 900 1300 l S\n"
        b"BT /Helv 22 Tf 0 0 0.8 rg 540 1315 Td (600) Tj ET\n",
    ),
    (
        b"Dimensions Reference",
        True,
        # A second, fainter dimension run. A PREFIX EXTENSION of the name
        # above, on purpose -- see the header.
        b"0.4 0.4 1 RG 1 w 300 900 m 300 1050 l S 1500 900 m 1500 1050 l S\n"
        b"300 1000 m 1500 1000 l S\n"
        b"BT /Helv 22 Tf 0.4 0.4 1 rg 850 1015 Td (1200 REF) Tj ET\n",
    ),
    (
        b"Electrical",
        False,  # ★ OFF in the default configuration. See the header.
        # A distinct red block, low on the sheet, so toggling it is visible.
        b"0.85 0.1 0.1 rg 300 200 m 800 200 l 800 500 l 300 500 l f\n"
        b"BT /Helv 24 Tf 1 1 1 rg 340 330 Td (ELECTRICAL) Tj ET\n",
    ),
    (
        b"Notes",
        True,
        # Green annotation text upper-left. Short name on purpose.
        b"BT /Helv 20 Tf 0 0.5 0 rg 140 1500 Td "
        b"(NOTE 1 - every layer here paints something.) Tj ET\n"
        b"BT /Helv 20 Tf 0 0.5 0 rg 140 1460 Td "
        b"(NOTE 2 - Electrical is off by default.) Tj ET\n",
    ),
]

# Object numbering: 1 catalog, 2 pages, 3 page, 4 contents, 5 font,
# then the OCGs at 6 .. 6+len(LAYERS)-1.
FIRST_OCG = 6
OCG_IDS = [FIRST_OCG + i for i in range(len(LAYERS))]

# ---------------------------------------------------------------------------
# The content stream: one BDC/EMC span per layer, each wrapped in q/Q so a
# layer's graphics state cannot leak into the next one.
#
# ★ q/Q INSIDE the span rather than around it. A `BDC` that is not closed by
# its `EMC` before the state is restored is a malformed nesting that some
# readers tolerate and pdfcer is entitled not to; keeping both pairs properly
# nested means this fixture tests optional content, not error recovery.
# ---------------------------------------------------------------------------
parts = []
for i, (name, _on, paint) in enumerate(LAYERS):
    tag = b"/OC%d" % (i + 1)
    parts.append(b"/OC " + tag + b" BDC\nq\n" + paint + b"Q\nEMC\n")
CONTENT = b"".join(parts)

properties = b" ".join(
    b"/OC%d %d 0 R" % (i + 1, OCG_IDS[i]) for i in range(len(LAYERS))
)

all_ocgs = b" ".join(b"%d 0 R" % oid for oid in OCG_IDS)
on_ocgs = b" ".join(
    b"%d 0 R" % OCG_IDS[i] for i, (_n, on, _p) in enumerate(LAYERS) if on
)
off_ocgs = b" ".join(
    b"%d 0 R" % OCG_IDS[i] for i, (_n, on, _p) in enumerate(LAYERS) if not on
)

objects = [
    # 1 - catalog. `/OCProperties` is the whole reason this file exists.
    #
    # ★ `/Order` is present and lists every group. It is what a viewer's
    # layers panel renders as the tree, and a file that omits it leaves the
    # order to the reader -- which would make this fixture's row order
    # implementation-defined, and therefore useless for asserting on.
    b"<< /Type /Catalog /Pages 2 0 R /PageMode /UseOC /OCProperties << "
    b"/OCGs [" + all_ocgs + b"] /D << /Name (Default) /BaseState /ON "
    b"/Order [" + all_ocgs + b"] /ON [" + on_ocgs + b"] "
    b"/OFF [" + off_ocgs + b"] >> >> >>",
    # 2 - page tree
    b"<< /Type /Pages /Kids [3 0 R] /Count 1 >>",
    # 3 - page. A1 landscape; see the header on why not a postcard.
    b"<< /Type /Page /Parent 2 0 R /MediaBox [0 0 2384 1684] "
    b"/Resources << /Font << /Helv 5 0 R >> /Properties << " + properties
    + b" >> >> /Contents 4 0 R >>",
    # 4 - contents
    b"<< /Length %d >>\nstream\n" % len(CONTENT) + CONTENT + b"endstream",
    # 5 - font
    b"<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica "
    b"/Encoding /WinAnsiEncoding >>",
]

# 6.. - the OCG dictionaries. `/Intent /View` is the default and is written
# explicitly so that a future fixture with `/Intent /Design` differs from this
# one in the dictionary rather than in an absence.
for name, _on, _paint in LAYERS:
    objects.append(
        b"<< /Type /OCG /Name (" + name + b") /Intent /View >>"
    )

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
out += b"trailer\n<< /Size %d /Root 1 0 R >>\nstartxref\n%d\n%%%%EOF\n" % (
    n,
    xref_at,
)

path = "fixtures/layered-drawing.pdf"
io.open(path, "wb").write(bytes(out))
print(
    "wrote %s  %d bytes  %d layers (%d on, %d off)"
    % (
        path,
        len(out),
        len(LAYERS),
        sum(1 for _n, on, _p in LAYERS if on),
        sum(1 for _n, on, _p in LAYERS if not on),
    )
)
