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
