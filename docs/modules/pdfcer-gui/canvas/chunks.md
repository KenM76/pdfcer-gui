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
