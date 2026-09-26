# `canvas::chunks` — **the boxes that show what a text block is made of**

`OPERATOR_REQUESTS.md` **O215**, ask 3, in his words: *"click on a text
block and it would show us boxes around all the blocks contained within it,
then let us use our usual mouse selection methods to move the chunks."*

## What a chunk is, and why the boxes are the prerequisite

A *chunk* is what [`crate::panels::objects::provider::ObjectModelProvider`]
calls a text **line**: the run range `text_line_runs_of` answers for, which
is the Part rung's unit and the operand of `MoveTextLine`,
`format.select_text_line` and the run menu. One text object on a CAD sheet
holds hundreds of them.

Selection already descends to that unit, and the operator cannot see it.
`canvas::presspick::covers` decides on **press** whether a drag moves the
chunk under the pointer or the whole block, and it decides by asking whether
the press landed inside the current selection's outline — a rectangle
nothing draws. A press a few points off it re-selects at the Object rung and
the same gesture moves the whole block instead. That is the mechanism behind
the non-determinism O215 ask 1 reports, so drawing the boxes is not
decoration: it is what makes the gesture repeatable, because it is what makes
the thing being aimed at visible.

## R8b — these are a cursor, not content marking

*Fuzzy, never sneaky* forbids styling applied content as provisional. A chunk
outline styles nothing: it is a **pre-commit affordance**, in the same class
as a snap indicator, a rubber-band or a selection handle, and the rule admits
those by name. Saving and reopening the document produces the same page; only
the cursor differs.

## Where the state lives

Two homes, exactly as [`crate::canvas::smart`] does and for its reason: the
live answer in `egui::Memory`, because the painter and the click path reach
it with a context and nothing else, and the persisted answer on
[`crate::app::prefs::Prefs`], because an operator who turns a mode off
expects it to still be off tomorrow. `crate::app::frame` mirrors the second
into the first once a frame, and the only writer of the persisted answer is
the dispatch arm an operator's press runs.

## Item notes

### `fn reaches`

Its own function so the rule can be tested without a decomposed page: the
failure it guards against is a build where both arms touch, which behaves
identically for every crossing band and takes far too much for every
enclosing one — and which no test of `within`'s plumbing would notice.

### `fn only_a_crossing_band_takes_a_chunk_it_merely_clips`

The one rectangle pair that tells the two directions apart: fully
outside is refused by both and fully inside is taken by both, so a build
in which the enclosing arm also merely touched would pass every other
case. Left-to-right encloses, right-to-left touches — the page-rung
band's rule, unchanged at this rung.

### `fn every_decline_reason_is_distinct`

The vocabulary earns its keep only if a harness can tell one from
another; two reasons that happened to be spelled the same would collapse
*click something first* into *the program is broken* with nothing to say
which had happened.
