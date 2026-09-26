# `ui-verify/checks/chunk_ghost_pixels`

`the_travelling_copy_is_on_the_glass` — **O215 ask 5, measured in pixels
instead of in the painter's own account of itself.**

# What this adds to the row beside it

[`crate::checks::chunk_ghost`] asserts that `overlay::draw_raster_ghost`
decided to blit, and how many pieces it blitted. That is the painter's claim
about the painter. A blit at alpha 0, a blit behind an opaque panel and a
blit sampling the wrong corner of the page texture all write the same
`drawn=1 clipped=0 reason=none`, so the trace cannot separate a working
preview from three broken ones.

Layout and clipping defects have exactly one oracle, a rendered screenshot.
This is that oracle for the one class of behaviour the harness was blind
to: an affordance that exists only between the press and the release.
[`crate::input`]'s `drag_observed` holds the frame open so it can be
photographed.

# The five questions, in the order they must be asked

| # | Question | Instrument |
|---|---|---|
| 1 | is the destination blank BEFORE the press? | [`pixels::region_not_uniform`] over the drop rect |
| 2 | is anything drawn there DURING the hold? | the same, inverted |
| 3 | did the document stay unmarked? | [`pixels::mean_luminance`], source before against source during |
| 4 | is it a COPY rather than the content? | the same, destination against source |
| 5 | did it come from the right part of the texture? | [`pixels::ink_run_into`], coverage compared |

★★ **The order of 3 and 4 is load-bearing, and it was got wrong first.**
Question 4 measures the copy against the source, so it is only meaningful
while the source is what it was. A build that washed the un-moved line
while a move was in flight was caught — but by question 4, which named it
*"the copy is not distinguishable from the content"* when the actual defect
was the opposite: the content had been altered to look like a copy. A
reference has to be established before it can be compared against.

⚠ **The inset is what makes question 2 an assertion rather than
theatre.** The outline ghost strokes the boundary of the very rectangle
being measured, so ink anywhere in the un-inset region is satisfied by the
affordance that already shipped — an assertion both outcomes satisfy. Only
the strict interior separates lettering from a box. Every region here is
inset by [`INSET_PX`] on all four edges for that reason, and the source
region is inset by the same amount so the two are comparable.

★ **Question 3 is the R8b assertion and it is the reason this check
measures the source twice.** *Fuzzy, never sneaky* forbids marking applied
content: a document being dragged must render exactly as the saved document
will render, with no tint, dashed outline or provisional layer over the
content that has not moved yet. A build that faded the source line while a
move was in flight would pass every other question here and would be
marking the canvas. Nothing else in the suite asks it.

⚠ **Question 5 is weak and is labelled weak.** It is a similarity test, not
an identity test: a build that sampled a *different line of the same
paragraph* would produce a comparable ink fraction and pass. It discriminates
blank paper from lettering and nothing finer. Tightening it means comparing
the per-column ink profile — the word shapes — which is worth building the
day a defect of that shape appears and not before.

# Calibration, and why it comes first

Question 1 is not a precondition dressed up as an assertion; it is the
second direction of the oracle. An after-capture alone says *"there is ink
at the destination"* without establishing that the ink was not already
there, and a destination chosen over a line of the paragraph would satisfy
every question below on a build that drew no ghost at all. The precedent is
`tools/ui-verify/tests/pixel_oracle_against_real_evidence.rs`, which pins
the contrast oracle by asserting both the unreadable heading and the
readable prose eighty pixels from it.

A destination that fails the before-check is a **SKIP**, not a FAIL: it says
the fixture was not laid out where this check believes it is, which is a
statement about the harness rather than about the build.

# Fixture — pinned, and `--pdf` is ignored

`fixtures/paragraph.pdf`, six lines of one text object. The geometry table
is in [`crate::checks::chunk_band`]'s header; [`LINE_BAND`] is its line 0
verbatim, chosen because it is the widest of the six and therefore the one
carrying the most pixels to measure.

[`DROP_DY_PT`] puts the copy 300 pt below that line, which on this document
is blank paper: the text occupies 620.0 to 708.4 and the drop band is 400.0
to 408.4, clear of line 5's floor by 212 pt.

# ⚠ HOW TO FALSIFY THIS CHECK — do this before believing a PASS

**Copy each file aside first** and restore from the byte copy; never revert
with git, because this project runs parallel tracks and a chained revert
discards another track's uncommitted work. Rebuild **both** `-p pdfcer-gui`
and `-p ui-verify`, and confirm the exe is newer than the source: a stale
binary is the commonest cause of a falsification that did not reproduce.

1. **Delete the call site.** Remove the `overlay::draw_raster_ghost(…)`
   statement in `canvas::painting::draw`. Question 2 goes red: the
   destination interior stays as blank as the before-capture found it.
   `chunk_ghost` beside it goes red too — the call site is where its trace
   line is written, so this plant does not separate the two rows.
2. **Remove the tint.** `overlay::raster::RASTER_GHOST_ALPHA` to `255`.
   **This is the plant that justifies the row existing.** `chunk_ghost`
   stays GREEN — the painter still decides to blit and still says so, and
   every trace field is unchanged — while question 4 here goes red, because
   a copy that renders at full strength is indistinguishable from the
   content itself. Measured, both ways, on the same build.
3. **Take the UV against the page rect** rather than against
   `strip::PageView`'s `paint_rect`. ⚠ **Measured: a NO-OP at the zoom this
   check runs at, and it stayed green.** `paint_rect` equals the page rect
   at every zoom below the pixmap ceiling, and [`WANT_ZOOM`] is far below
   it, so the plant changes nothing to catch. Question 5 is therefore
   **unfalsified**, not merely weak: nothing here is evidence about where
   the UV came from. Exercising it needs this gesture driven at the region
   tier, which is a different check and is not this one.
4. **Mark the canvas.** Paint the un-moved line over itself at
   `Color32::from_white_alpha(110)` after the blit in `draw_raster_ghost`.
   Question 3 goes red and nothing else does.
