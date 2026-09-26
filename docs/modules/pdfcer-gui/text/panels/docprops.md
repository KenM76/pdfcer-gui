# `text::panels::docprops` — the Document properties panel's copy

Every string the **Document properties** panel says about the file itself:
the four `/Info` fields' labels, the seven read-only facts, and the two
disclosures a document can owe an operator — a value pdfcer could not decode
exactly, and an index pdfcer had to rebuild in order to open the file at all.

## Why this is its own module, and it is two reasons rather than one

**1. The panel is its own panel** — the operator, 2026-09-05: *"the document
properties are still always visible in the properties tab. it needs to get
out of there and be in its own document properties tab."* These strings moved
out of [`super::properties`] with the section they belong to, in the same
commit, because copy that lives in the catalog of a surface it is no longer
drawn on is copy nobody finds when they come to change it. Every one of them
was reached from exactly one file before the move and from exactly one file
after it.

**2. R2, measured rather than anticipated.** `text/panels/properties.rs`
stood at **1,469 lines against the 1,500-line ceiling** on the day of the
move — its own `textobject` module records having been split off at 1,446 for
the same reason. So the move is also the split that gate was going to force
within a couple of sentences, and it is a split along a subject boundary
rather than an arithmetic one: what remains in `properties` describes **what
is selected**; what is here describes **the file**.

## The names lost their `properties_` prefix, deliberately


The three `recovered_*` functions kept their names: they were never
prefixed, they name the *event* rather than the surface, and renaming them
would have been churn with no reader served.

## What is NOT here

The empty state. A panel with no document open never draws its own body —
[`crate::panels::Panel::show`] answers that case once for all twelve panels
with [`super::panel_no_document`] — so there is no "open a document first"
sentence in this file to drift from the other eleven.

## Item notes

### `fn the_two_drop_reasons_are_never_collapsed`

Driven from all four corners rather than from the one case a fixture
happens to produce. The interesting corners are the two SINGLE-kind
ones: a recovery whose drops are all false positives must not produce
the sentence about untrustworthy numbering, and a recovery whose drops
are all mismatches must not produce the reassuring one. Either would be
the collapse the engine's enum exists to prevent, arriving at the last
possible moment.

### `fn an_elided_list_of_dropped_objects_counts_what_it_left_out`

⚠ Both sides of the boundary, because an elision tested only above its
threshold cannot tell a correct rule from one that always elides, and
one tested only below it cannot tell a correct rule from one that never
does.

The exactly-at-the-limit case is here on purpose: an off-by-one there
produces "and 0 more", which is the silent-truncation failure wearing
the opposite coat — a remainder announced that does not exist.

### `fn the_heading_is_not_the_tabs_name_again`

The panel's tab is called *Document properties* — it takes its name from
`file.document_properties`' label, which is how every dock tab in this
build is named. A heading reading the same words immediately under it is
a line of an inspector that tells the operator nothing they did not
learn by clicking, and it is the obvious thing for a later edit to
"tidy" the heading into.

### `fn the_disclosures_describe_the_file`

Both are drawn at ordinary weight beside facts, and the wording is what
carries the distinction — a sentence that opened "pdfcer could not…"
would read as a defect report about the program in a panel whose whole
subject is the operator's file.
