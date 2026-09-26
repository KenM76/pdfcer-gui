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

## Item notes

### `const REGION_SIZE_ITEM_PREFIX`

Published for [`crate::dialogs::new_document`]'s reason, which is the same
one here: an egui combo popup is an `Area` laid out at paint time, so
nothing outside the process can compute where an entry is, and a check that
can open a list but not choose from it can assert only that a control
exists. The property worth asserting is that **picking a size makes the
sheets that size in the saved file**, which needs a click on a specific
entry.

### `const REGION_OUTCOME`

Published so a driven check can assert that the sentence changed when the
size changed. A window that showed "everything fits" for every size would
satisfy a check that only looked for the region's presence, which is the
shape `text::security::cannot_author` failed in for three corrections and
zero call sites.

### `const MIN_CUSTOM_MM`

[`crate::dialogs::new_document::MIN_CUSTOM_MM`]'s value and reasoning, and
deliberately the same number: two windows in one program that disagree about
the smallest sheet a PDF may have would be a defect whichever of them was
right. ISO 32000-1 Annex C.2 advises 3 units (≈ 1.06 mm); 2 mm is
comfortably over it and is a bound an operator can remember.

### `const MAX_CUSTOM_MM`

5,080 mm = 200 inches = 14,400 default user space units, Annex C.2's advised
maximum. See [`MIN_CUSTOM_MM`] for why the number is shared with the
sized-New window rather than re-derived.

### `const DOCUMENT_SURVEY_CAP`

128 is a whole drawing set with room to spare and a bounded cost on a
document that is not one. The census is a `PDFCER_DIAG` line — it costs a
disabled build nothing at all, because `crate::diag::trace` takes a closure
— so the cap is about the size of a trace artefact a harness has to parse,
not about frame time.

### `enum Choice`

A separate type from `pdfcer_core::paper::PaperSize` rather than
`Option<PaperSize>`, for [`crate::dialogs::new_document`]'s reason: "custom"
is a *state of the window* — it opens two fields and changes what the
summary reads — and not a missing size.

### `fn sheet_pt`

# One function, read by four callers, and that is the point

The summary line, the diagram, the outcome sentence and the commit all
ask this. Four separate computations of "what did they pick" is how a
window comes to promise 841 × 1189, draw a portrait outline and produce
1189 × 841 — and a transposition is exactly the arithmetic that is easy
to write four times and hard to notice once.

### `fn size_id`

# Why this exists, and why the `choice={:?}` beside it is not enough

A driven check has to be able to say *which size the entry it clicked
actually was*, and it cannot ask `pdfcer-core`: `ui-verify` has exactly
one dependency by deliberate design, and pulling the engine into a
verification harness would make it fail to build for reasons unrelated
to the thing under test, on the day it is most needed.

So the size list is clicked **by index** — `page-size.size.item.6` — and
`PaperSize::ALL` is `#[non_exhaustive]` with its own doc comment saying
the table will grow. A size inserted before A6 would make a check click
A5 and then assert A6's dimensions: a red run whose message blames the
application for a table that moved. This line is what lets the check
notice instead.

⚠ **Not the `Debug` spelling.** `choice={:?}` is emitted beside this for
a human reading a trace, and nothing parses it: Debug-formatting a
domain type and then parsing it produced two false failure reports in
this project in one week. `PaperSize::id` is the opposite of a Debug
spelling — the engine documents it as *"ASCII, lowercase, hyphenated,
and must not change once shipped"*, which is precisely the contract a
harness needs.

### `fn is_valid`

Only a custom size can fail: every entry in `PaperSize::ALL` is a real
sheet in range by construction. Checked on the **millimetre fields**
rather than the computed points, so the refusal can name the numbers the
operator typed.

### `fn would_change`

Not a validity question and not folded into one. The engine reaches
§11.1's net-zero rule by itself — a page asked for the size it already
has records no command — so pressing the button on an unchanged size is
safe, and it is also a control the operator pressed that did nothing and
said nothing, which is the defect class this project is named after. So
the window says so first.

### `fn outcome`

Three states, and keeping them three is the whole point:

| state | wording |
|---|---|
| measured, nothing outside | [`t::fits`] — the only promise this window makes |
| measured, something outside | [`t::overhang`] — the amount, per edge, in points |
| **not** measured | [`t::overhang_unmeasurable`] — a stated boundary, and **not** the first |

⚠ Collapsing the third into the first is the single most dangerous edit
available in this file, which is why
[`crate::app::actions::pagesize::SheetSurvey::overhang`] returns
`Option` and its test asserts an unmeasured extent is not a zero one.

### `fn diagram`

A cursor affordance under R8b rule 4, drawn off-canvas. It exists
because an overhang expressed as *"1,636 pt past the right edge"* is a
number an operator has to convert into a mental picture, and the
conversion is exactly where a wrong decision gets made — 1,636 pt is
meaningless until you see that it is most of the sheet.

# What is drawn, and what each outline means

* the **old** sheet — the size the picked pages are now, weak;
* the **new** sheet — filled with the accent, because it is the thing
  being chosen and the thing that will exist;
* the **drawing** — the union of the drawn extents that could be read,
  as a dashed outline, so the part of it hanging outside the new sheet
  is visible as a shape rather than as a number.

Both sheets share one scale — the union of everything, fitted to the
strip — which is the only arrangement in which the comparison means
anything. Two independently-fitted outlines would draw A4 and A1 the
same size.

No raw `Color32`: the three colours are theme roles
(`Theme::accent_pair`, `weak_text_color`, `warn_fg_color`), so a preset
change moves them with everything else.

### `fn landscape_transposes_a_standard_size_and_a_custom_one`

The single most likely defect in a window with a size list and an
orientation pair: a standard size that turns and a custom size that does
not, or the reverse. Both go through [`PageSizeDialog::sheet_pt`]
precisely so they cannot diverge, and this is what holds that. The
consequence of getting it wrong here is worse than in the sized-New
window — there it makes a blank page the wrong way up, here it crops a
drawing along the wrong axis.

### `fn it_opens_on_the_size_the_sheets_already_are`

The decision this file's `open` doc comment argues, checked rather than
merely written down. A default that drifted back to A4 — the sized-New
window's default, and the obvious thing to copy — would mean an operator
who opened this window to *look* was one careless press from cropping an
A1 drawing to A4.

### `fn opening_on_the_current_size_would_change_nothing`

Both halves. `would_change` false must not hide the button — an operator
who opened the window to look would be left with nothing to press but
Cancel — and it must be *said*, because a control that acts and reports
nothing is the defect class this project is named after.

### `fn choosing_a4_for_an_a1_drawing_reports_the_overhang`

The property the whole window exists for. `overhang` is
`SheetSurvey`'s and tested there; what this asserts is that **the
dialog's own chain** — choice → `sheet_pt` → `target_rect` → `overhang`
— reaches it. A window that computed the right rectangle for the summary
line and a different one for the measurement would pass every test in
`pagesize` and put a false promise on screen.

### `fn growing_the_sheet_reports_that_everything_fits`

The falsifying direction of the test above, and it is not redundant: a
sign error in the overhang arithmetic passes that test and fails this
one. An operator putting an A3 detail sheet onto A1 paper must see the
promise, not a warning.
