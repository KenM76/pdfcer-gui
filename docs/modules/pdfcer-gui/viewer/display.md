# `viewer::display` — how many pages are on screen, and in what arrangement

One enum, [`PageDisplay`], and the three rules that hang off it: which
modes scroll, which modes pair pages into spreads, and what a fresh
profile gets in each ribbon mode.

## Continuous is an option, not a replacement — and that is the whole point


> *"continuous scroll should be an option under the view tab as the way I
> move around a page is great when working with drafting drawings."*

[`PageDisplay::Single`] is therefore **not** a legacy mode and is not on a
path to removal. Paging one sheet at a time is the right model for drafting
review — a drawing sheet is a unit of work, and a page boundary is a
deliberate step rather than an interruption — and it stays the default
everywhere except Read. `GUI_ROADMAP.md`'s decision table records the same
ruling in one line: *"An option, not a replacement. Single page stays the
default; the four modes sit together on View."*

The consequence for anybody editing this module: a change that makes
`Single` a degenerate `Continuous` — one page in a scrolling strip, with
the strip's gaps, the strip's scroll range and the strip's current-page
tracking — has failed even if every test stays green. [`crate::viewer::strip`]
is built so that `Single` produces a one-row strip whose size **is** the
page's drawn size and whose scroll range **is** the page's own, so the
single-page experience is bit-for-bit what it was before Phase 4. The tests
in that module assert exactly that, and they are the ones to keep honest.

## Read defaults to continuous; every other mode keeps single page


> *"Read defaults to continuous scroll; Review and Edit default to single
> page. … Reading a document is a continuous act — you scroll through it,
> and a page boundary is an interruption. Marking up and editing a drawing
> is a per-sheet act — you work on one sheet, and paging is how you move
> between them deliberately. The right default was never global; it was per
> mode."*

[`PageDisplay::default_for_mode`] is that sentence in code, and it is the
**only** place the rule is written down. It takes a mode id as a `&str`
rather than a `crate::app::modes` type on purpose: the three ids are the
manifest's own (`"read"`, `"review"`, `"edit"`), a customized manifest may
declare others, and an unknown mode has to fall back to something rather
than fail. It falls back to `Single`, because `Single` is the default and an
unrecognised mode is not evidence that the operator wants a different one.

## Why the on-disk spelling lives here

[`PageDisplay::id`] and [`PageDisplay::from_id`] are the persistence
format, and they sit beside the enum rather than in the store
([`crate::viewer::remembered`]) for the reason every "one spelling" rule in
this project has: a variant added here with no line in `id` would round-trip
as something else, and the pair is asserted exhaustively by
[`tests::every_mode_round_trips_through_its_on_disk_spelling`] over
[`PageDisplay::ALL`] — so adding a variant and forgetting its spelling is a
test failure rather than a silent data loss the operator discovers on the
next launch.

## Item notes

### `fn all_lists_every_variant`

Everything below iterates `ALL`, so a variant missing from it would
make those tests vacuously pass about the variant that matters. The
exhaustive `match` is what makes this fail to *compile* when a variant
is added, which is stronger than failing to run.

### `fn every_mode_round_trips_through_its_on_disk_spelling`

The persistence format's whole correctness. A variant with no `id` arm
would not compile; a variant whose `id` collides with another's would
fail here, and the symptom in the field would be an operator's
remembered choice quietly becoming a different mode on the next launch.

### `fn facing_pairs_pages_after_a_solitary_cover`

The spread rule, stated as the mapping a reader can check by eye
against a physical document. Getting the parity backwards puts page 3
on the right of a spread it should open, which on a drawing set with a
title sheet is visibly wrong.

### `enum PageDisplay`

The four positions of View ▸ Page display. Exactly one is active at a time
— it is a radio, not four toggles — which is why this is an enum on
[`crate::viewer::ViewState`] rather than a pair of booleans. Two booleans
would admit a fifth state ("facing, but also single") that means nothing,
and the ribbon would have to reconstruct which of them is "on".

### `const ALL`

The order is the ribbon's and it is not arbitrary: it runs from fewest
pages on screen to most, so the group reads as a scale rather than as a
list. Exhaustive by construction — [`tests::all_lists_every_variant`]
fails if a variant is added and not listed, which is what makes the
round-trip and command-id tests below complete rather than merely
passing.

### `fn is_continuous`

The single predicate the rest of the build asks. It is what decides
whether the strip holds every page or only the current row, whether the
current page is derived from the scroll offset or set by navigation,
and whether more than one page can need a raster at once.

### `fn id`

Lowercase, hyphenated, and **never** the enum's `Debug` spelling: a
`Debug` impl is a developer convenience that a `derive` may change,
and a persistence format that changed with it would silently reset
every operator's remembered choice. See
[`crate::viewer::remembered`] for the file these ids appear in.

### `fn from_id`

`None` rather than a default, deliberately: the caller is reading a
file that may have been written by a newer build or edited by hand, and
*"this line names a mode I do not have"* is a different fact from
*"this document has no remembered mode"*. The store treats the first as
a line to drop and the second as a document to give the mode default
to; collapsing them would make an unrecognised entry look like a
deliberate choice of `Single`.

### `fn default_for_mode`

**The one place `MODES_AND_PANELS.md`'s per-mode rule is written
down.** Read is continuous; everything else — including an id this
build does not know — is single page. See the module header for the
operator decision behind it and for why an unknown id falls back to
`Single` rather than refusing.

### `fn row_count`

`Single` and `Facing` still report the document's full row count even
though they show one row at a time: the number is what a *strip* would
hold, and the two non-scrolling modes are the same strip with one row
selected. Keeping one definition means a page step and a scroll step
cannot disagree about how many rows there are.
