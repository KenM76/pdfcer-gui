# `dialogs::page_size` — **changing the paper an open drawing sits on**

`pages.resize`, the Pages tab's Transform band. The window that answers *"my
sheets are A1 and I want them A3"* — and, before it does anything, tells the
operator which of two very different things he is about to get.

## Why this window exists, and why it is mostly a sentence


[`crate::app::blank`]'s §3a has known this the whole time; its comments
say in as many words that *"pdfcer-core answered it on 2026-08-18 —
`EditSession::set_media_box`, `set_media_boxes` and a `pdfcer_core::paper`
table"*. **Writing it down was mistaken for acting on it.** That is the
fourth instance of the pattern found this week and it is recorded here
rather than in a session log because this file is where a reader will next
be standing when it matters.

## THE DESIGN DECISION: this window's real product is a MEASUREMENT

Changing a `/MediaBox` changes **the paper**. It does not move, scale or
reflow one byte of what is drawn on the page. So:

> **An A1 drawing put on A4 paper is CROPPED, not SHRUNK.**

Every other page-size control the operator has ever used — Word,
LibreOffice, a print dialog's Fit-to-page — reflows or scales. He will
arrive here expecting *"print it smaller"* and, on his own title-block
sheet, the thing he would lose is the title block: measured 2026-09-06, it
sits at x 1831–2207 pt and an A4 sheet stops at 595.28.

A window that let him find that out from the result would be *"fuzzy, never
sneaky"*'s purest failure — he would get exactly what he asked for and
nothing like what he wanted. So three things are on screen before he can
commit, and the third is the one that does the work:

1. [`crate::text::page_size::intro`] — the rule, in words.
2. [`Self::diagram`] — the old sheet, the new sheet and the drawing's own
   extent, drawn to scale, so the overhang is a **picture** before it is a
   number.
3. [`crate::text::page_size::overhang`] — *"the drawing runs 1,636 pt past
   the right edge"*, recomputed as he changes the size.

⇒ **A rule is a thing to agree with; a measurement is a thing to act on.**
Item 3 is why this is not just a size picker with a warning on it.

### And item 3 is a thing the ENGINE said it could not do

`MediaBoxChange::lost_area` carries a named residual in the engine's own
words: it reports that the *sheet* shrank, **not** that any *content* was in
the region it lost, because *"pdfcer has no page-content bounding-box
facility yet"*. That is true of `EditSession` and false of `pdfcer-core` —
`PageObjects::page_bbox` is exactly that facility, and this shell already
holds a decomposition per page. [`crate::app::actions::pagesize::survey`] is
where the two are put together, with its cost bounded and its boundary
stated.

## Rule 4, and the one affordance that is allowed on the canvas

**R8b rule 4: disclosure is off-canvas; applied content renders exactly as
saved content will.** Everything here is off-canvas — it is a desktop
window. The pre-commit preview of the new paper outline is explicitly
welcome as a *cursor* affordance, and it is drawn **inside this window**
rather than over the page, for two reasons: the canvas is another track's,
and a scale diagram beside the size list is read by an operator whose eye is
already on the size list. Nothing provisional is styled onto the page.

## R9, and what is absent rather than greyed

The custom millimetre fields exist only when Custom is chosen, and the
commit button exists only when the size is in range —
[`crate::dialogs::new_document`]'s argument, unchanged: greying is for
*temporarily* unavailable, the refusal line is already on screen naming both
limits, and a control reappearing as a number crosses a bound is a clearer
signal than one changing shade.

## Why here and not Document ▸ Properties

[`crate::dialogs::new_document`]'s header nominated *"Document ▸
Properties"* for a page-resize surface, and that is departed from
deliberately rather than overlooked. The Pages tab's own organising rule
decides it: *"every command here operates on the current document's page
set, and every one respects the thumbnail rail's current selection"* —
which is precisely the drawing-set operation `set_media_boxes` was written
for, and the rail is how the operator says *these* sheets. Document
Properties has no operand and no selection. The reserved id, `pages.resize`,
was on the Pages tab all along.
