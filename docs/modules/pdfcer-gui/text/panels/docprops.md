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
