# `pdfcer-gui/canvas/textedit/paint`

## Item notes

### `fn sizes`

A draft on an existing run is set in the shell's font at **the run's own size
times the zoom**, unclamped, so the stand-in occupies the space the saved text
will and a long draft is visibly long. Small text at low zoom is small here
too; zooming in is the remedy, as it is for reading the page.

New text (an origin or a box) has no run to take a size from. Its box is the
nominal slot's screen height clamped to `MIN_PREVIEW_PT`–`MAX_PREVIEW_PT`, and
its glyphs `PREVIEW_FILL` of that, because a 4 pt slot at 25 % zoom is a box
two pixels high.

### `fn selection`

For an existing run this is the union of its glyph boxes; for a new-text
origin it is a nominal one-line box at the click. Both are converted through
[`crate::viewer::pdf_space_to_canvas`], the inverse of the bridge
[`resolve_run`] uses, so the caret lands on the glyphs it was resolved from.
**Highlight what is selected**, one rectangle per run of characters that
share a row.

# Why it is measured character by character rather than from two
# endpoints

Because a selection can wrap. Two endpoint rectangles describe a selection
on one row and say nothing useful about one spanning three — the middle rows
are not between the two x-coordinates in any sense a painter can use, and
reconstructing them means asking the galley for its row geometry, which is
a second derivation of what `pos_from_cursor` already answers.

So each character's own slot is asked for, and adjacent slots on the same
row are merged into one rectangle. The cost is O(n) in the SELECTION's
length, per frame — and a draft is one show operator, so n is tens of
characters. That is the same trade `Draft::caret` makes for character
indices, made again for the same reason.

A character's right edge is taken from **the next slot's left edge**,
not from its own `max.x`. The two differ where a row ends: the last
character of a wrapped row has a next slot on the row BELOW, which is how
the row break is detected at all.

### `const REGION_BOX`

## D4a's ghost text, the decision that followed it, and why that
## decision was half-right

The old shell drew the draft *as text*, in an `egui` proportional font, over
a **translucent mask** — which `DEFECTS.md` D4a names as the second
contributor to "weird": *"you type in the wrong typeface at the wrong
widths, then it snaps to reality on Accept."*


> *"The characters themselves are shown off-canvas, in the status bar, where
> `text::textedit` owns the sentence."*

**That promise was never kept.** `text::tool::text_edit_live` says *"Enter
commits what you have typed. Esc abandons it."* and nothing anywhere renders
the draft. So the operator typed into a bracket and their characters
appeared nowhere at all:

> *"I can edit text now, but there is no live preview of that either."*
> — 2026-08-20

## Why an in-place editor BOX is not the ghost D4a condemns

The distinction is not cosmetic and it is the whole justification for
reversing the decision:

| | old ghost | this |
|---|---|---|
| drawn | translucent, **over** the original glyphs | **opaque**, covering them |
| reads as | the document, in the wrong typeface | an editor, obviously |
| on commit | "it snapped to reality" | the editor closed |

D4a's defect is that the ghost **imitated applied content**. Rule 4's
one-line test — *would a screenshot of the editing canvas differ from a
screenshot of the same document saved and reopened?* — caught it because the
old shell's canvas differed **in the one respect the operator was looking
at**, while claiming to be the document.

An opaque editor box differs too, and does not claim otherwise. Rule 4
permits exactly this by name: *"a snap indicator, a hover highlight, a
rubber-band … these are the cursor; they describe what is about to
happen."* An in-place editor **is** the cursor. What the rule forbids is
styling content **already applied** as though it were pending — and this
covers the applied content rather than restyling it.

Every program does it this way, which is the second half of the argument: a
spreadsheet cell, a Word table cell, a CAD attribute editor, a file-name
rename in Explorer. All of them cover the original with a filled box while
you type and reveal the result on commit. Nobody is surprised when the box
closes.

## The two objections that stood, and what happened to them

**"A ghost in the wrong face is a lie about the document."** True of a
translucent one. An opaque box makes no claim about the document's typeface
because it is visibly not the document — it has an edge, a fill and UI text.

**"A ghost in the right face would need re-rasterizing the run's embedded
font per keystroke, and `BENCHMARK.md` says ~99 % of render cost on dense CAD
is resolution-independent."** Still true, and this is why the box does **not**
attempt the document's typeface. It costs one filled rectangle and one text
layout per frame.

## The caret is measured against the text AS DRAWN
