# `dialogs::stamp_collection` — turning this document into stamps Acrobat
will show in its own menu

`OPERATOR_REQUESTS.md` **O169**, the operator's words:

> *"if acrobat has a way of adding custom stamps or text, we need the same
> feature too with the same import/export to make the stamps as Adobe has
> and is compatible with adobe's"*

## The finding that decided the whole shape of this window

**There is no interchange format, because Acrobat has none.** A stamp
collection *is an ordinary PDF* — one file per category, one page per stamp,
the category in `/Info` `/Title` and the names in the catalog's `/Names` →
`/Pages` name tree. So "the same import/export" is not a format to
implement: **handing someone the PDF is the export**, and dropping one into
Acrobat's stamps folder is the import. What the operator was missing was not
a converter — it was a way to *author* one of those files, and that is
exactly what this window does.

⇒ The window therefore has no format picker, no options page and no
"compatibility mode". It asks two questions — **what is this set called**
and **what is each page called** — and writes a file measured against the
four Adobe collections on this machine.

## The trap this window is arranged around

`pdfcer_core::stamp_file::name_stamp_pages` names `stamps[i]` to **page `i`
of the document it is given — by counting, not by lookup.** Untick page 1,
hand the engine the full name list, and every stamp names the page below the
one it describes: a file that opens, holds the right number of stamps,
appears in Acrobat's menu, and **stamps the wrong picture every time**.
There is no symptom short of looking at the artwork.

The guard is structural rather than careful. [`crate::stamps::Plan`]'s
`for_engine()` and `pages_to_extract()` are the *same filter over the same
rows*, so the name list and the extracted page list cannot drift apart, and
`crate::stamps::tests::plan_and_extraction_agree` is the unit test that
fails if somebody makes them two filters again.

## Why the operator's document is never touched

`Save as stamp collection…` is a **Read-mode-legal act** by the standing
rule — *Read may produce a new document; it may not modify this one*. The
write path never takes a mutable handle on his session at all: it extracts
the ticked pages into a new document and names *those*. See
[`crate::stamps::write`]'s header for the two reasons that matters.

## Rule 4, in this window

Every adjustment pdfcer makes to a *hidden* identifier is listed **before**
the write, off-canvas, in [`Self::adjustments_group`] — and only when there
is something to list. Nothing on the page is marked, tinted or flagged,
because the operator's own words are that *"the nagging and red flagging in
the original GUI made for a lot of extra bugs in the visibility when
editing."*

The half that is easy to skip is the one that matters most here: an operator
**cannot see** the identifier. His name is written exactly as typed; the
identifier beside it may have lost a space, a `#` or eight characters, and
the only place he could ever learn that is a sentence like these.

## Item notes

### `const FOOTER_PTS`

A named constant used **both** by the opening-height calculation and by
the scroll area's `max_height`, which is `dialogs::formfield`'s recorded
finding: those were a literal `40.0` in one place and nothing at all in the
other, which is how the two halves of one reservation drift apart.

### `const ROWS_SHOWN`

A **constant inventory**, never a measurement from inside the scroll area
— R128's feedback loop. A document of eighty sheets must not open a window
eighty rows tall; it opens at eight and scrolls, which is a decision made
here rather than a number egui arrives at by growing.

### `fn rows_group`

# Why every keystroke re-derives the WHOLE list

Uniqueness is a property of the **set**, not of a row. Renaming row 1
can free the name row 4 was renumbered away from, and a per-row update
would leave row 4 wearing a *"we renamed it"* sentence explaining a
collision that no longer exists — a disclosure whose subject has gone.
`crate::stamps::tests::rederiving_after_a_rename_drops_the_stale_disclosure`
is the test that holds this.

The cost is a quadratic pass over a list whose length is a page count,
run on a keystroke. On the operator's largest sheet set that is a few
hundred string comparisons in a frame that already rasterized a page.

### `fn adjustments_group`

Drawn only when the list is non-empty. A permanently-present box
reading *"no adjustments"* trains an operator to stop reading the place
adjustments appear, which costs exactly the one time it matters.

### `fn commit_row`

Greyed with the reason on hover, which is the one situation **R9**
reserves greying for: *temporarily* unavailable, and one keystroke or
one tick makes it live. The blocker's sentence is also drawn beside the
button rather than only on hover, because a hover tooltip is a thing you
find after you have already wondered why nothing happened.

### `fn open`

The plan is built **once, here**. Re-deriving it per frame would be a
name-tree read and a full uniqueness pass sixty times a second for an
answer that only changes when the operator types — and, worse, would
throw away every name he had typed on the frame after he typed it.

# Where the seeded category comes from, in order

1. The collection's own category, when this document already is one.
   Re-opening a collection to fix one typo must not retype the heading.
2. The document's `/Info` `/Title`.
3. The filename's stem.

All three are *suggestions in an editable field the operator is looking
at*, which is why none of them owes a disclosure sentence: what will be
written is on screen, in the box, before anything is written.

### `fn open_for`

The no-document guard is real rather than ceremonial: the window's rows are
one per page and its category is seeded from the file, so there is nothing
to build without one. An empty document declines for the same reason —
a collection with no stamps in it is not a thing Acrobat will show.
