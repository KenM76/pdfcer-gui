# `app::actions::pagesize` — changing the **paper** an open drawing sits on

The body of [`PageAction::SetPageSize`], and the pre-commit
[`survey`] the sheet-size window reads to tell the operator, *before* he
commits, which of two very different things he is about to get.

Its own file under **R2** rather than part of [`super::pages`], whose
subject is *"a page index is a position, not an identity"* — the resync a
**structural** edit owes. A media
box change is not structural in that sense. It adds, removes and renumbers
nothing, so every one of that file's five table rows is "unchanged" for this
verb, and putting it there would have made the reader look for a row that is
not there.

---

## 1. THE ONE FACT THIS MODULE EXISTS TO CARRY

**`/MediaBox` is the paper. Changing it does not move, scale or reflow one
byte of what is drawn on the page.**

So an A1 drawing put onto A4 paper is **cropped, not shrunk**, and the
operator who reaches for this control expecting *"print it smaller"* loses
his title block instead. Every other page-size control he has ever used —
Word, LibreOffice, a print dialog's Fit-to-page — reflows or scales. This
one does not.

### Measured, not assumed

Through the engine's own `set-page-size` (the `/MediaBox` write
`EditSession::resize_pages` makes), on
`fixtures/a1-titleblock.pdf`, A1 → A4:

| what was measured | result |
|---|---|
| every glyph run's coordinate (`extract-text --json`, full diff) | **byte-identical**. The title block sits at x 1831–2207 pt before and after; an A4 sheet stops at 595.28, so it is entirely off the paper and still in the content stream |
| `/CropBox`, `/BleedBox`, `/TrimBox`, `/ArtBox` | **all four untouched**. Only `/MediaBox` is rewritten |
| annotation `/Rect` (`annots-with-everything.pdf` → A6) | **identical**. A `Square` at 120,560–320,700 is still there on a 298 × 420 sheet, i.e. off it |
| form fields (`list-fields`, full diff) | **identical** |
| ce dimensions (`dimension-list`, `/PieceInfo` sidecar, printed value) | **identical** — the value stayed `400.00 pt`, which is *correct*: the drawing did not move, so the measurement is still true |
| a certified document | refused by name, `CertificationForbidsChange` |
| an encrypted document with no password | refused by name at open |

**R8b rule 15, and the reason this verb is safe.** A **pdf dimension** —
the printed measurement a CAD exporter drew — is page content: it does not
move, and it can end up off the sheet. A **ce dimension** — the one pdfcer
authored — is an annotation plus a `/PieceInfo` sidecar: it does not move
either, and its number stays true *because* nothing moved. A verb that
scaled the drawing to fit would have to rescale every ce dimension group's
calibration to keep its numbers honest, and would silently falsify every
group it missed. This verb cannot get that wrong, because it does not touch
them.

## 2. Scaling the drawing is the engine's verb, not a composition

[`scale`] is the body of `PageAction::ScalePages`: one call to
`EditSession::scale_pages`, which wraps each page's content in
`q s 0 0 s tx ty cm <old visible region> re W n … Q`, rewrites every present
page box to the new sheet, and moves annotations, widgets, `/Measure`
factors and link destinations with it, as one undo step. Composing it here
from `transform_objects` and the five annotation and dimension verbs was
rejected: its likely failure, a ce dimension left at the old calibration,
prints a wrong measurement that looks right. The engine refuses a page that
carries ce dimensions instead (`ScaleRefusedCeDimensions`), and [`refusal`]
words that by page number.

`ScaleRequest` states the sheet as the page is **displayed** (after
`/Rotate`), while the window states it as the media box is written, so
[`SheetSurvey::scale_request`] transposes when every operand is
quarter-turned and asks for `OrientationPolicy::Match` when the set is mixed.
[`SheetSurvey::scale_span`] quotes the factor before the commit from the
engine's own `pageops::scale::plan_placement`, so the number in the window is
the one the commit applies.

What `scale` discloses, off-canvas: on a Fill, the sheets whose scaled
drawing now runs past the paper ([`fill_overflow`], 0.01 pt slack); and any
geospatial `/Measure` the engine left unscaled.

## 3. The shell measures what the engine says it cannot

`MediaBoxChange::lost_area` carries a named residual, in the engine's own
words: it reports that the **sheet** shrank, *not* that any **content** was
in the region it lost, because *"pdfcer has no page-content bounding-box
facility yet"*.

That sentence is true of `EditSession` and **false of `pdfcer-core`**:
`pdfcer_core::vector::PageObjects::page_bbox` is *"the union of every
object's page bbox — the page's drawn extent in page space"*, and this shell
already holds one per page in [`crate::app::cache`]. So the disclosure the
engine could only make in geometry — *"the sheet lost area"* — is made here
in the operator's terms: *"the drawing runs 1,636 pt past the right edge"*.

**And the boundary is stated rather than implied.** `page_bbox` is the
union of the **drawn** objects. Annotations are separate objects with their
own `/Rect`, which it does not walk; they keep their coordinates too, so
they can fall off exactly as content can.
[`crate::text::page_size::annots_not_counted`] says so on screen, and its
own test holds that it keeps saying so.

## 4. Why the PLURAL verb, even for one sheet

`EditSession::resize_pages` with `CropFollow::WhenItMatched`: a crop box
equal to the old sheet becomes the new sheet, so growing the paper grows
what is seen, and one cropped to a region is kept and reported by
`disclosure_crop_inside`. Like `set_media_boxes` it is *"a sheet set is
resized as a set"*, one
undo entry however many pages, refusals raised **before anything is
committed** so an out-of-range index leaves the document untouched rather
than half-resized. Calling the singular verb in a loop would be functionally
identical and would leave the operator pressing Undo once per sheet. Its
own doc comment names this shell as the caller it was written for.

## Item notes

### `fn count_entry`

A helper rather than three inline filters because `MediaBoxEntry` is
`#[non_exhaustive]`: one place to look when a variant is added, instead of
three that each silently stop counting it.

### `fn disclosures`

**Counted across the set rather than named per page**, which is the opposite
of what the CLI does and right for the opposite reason. The CLI's invocation
*is* the commit and it has one file in front of it, so it prints a note per
page. This surface is a status line an operator reads in passing while the
document is on screen in front of him: *"7 sheets lost area"* is a fact he
can act on, and seven sentences differing only in a number is a wall he will
stop reading — which is how a disclosure stops being one.

### `fn his_title_block_is_measured_as_running_off_the_right_edge`

His A1 title block sits at x 1831–2207 pt (measured from
`fixtures/a1-titleblock.pdf` with `extract-text --json`). A4 stops at
595.28. This is the arithmetic the window shows him before he presses
anything, and getting it wrong in either direction is the difference
between a warning he heeds and one he ignores.

### `fn an_unmeasured_extent_is_none_and_not_zero`

The single most dangerous confusion available in this module, and the
one the engine's own residual refuses to make. `None` must survive as
`None` all the way to the caller, because the caller words the two
differently and a `unwrap_or_default()` anywhere on this path would
silently turn a stated boundary into a promise.

### `fn a_drawing_that_fits_reports_a_measured_zero`

The other half of the pair above. Without this, a build that returned
`None` whenever the overhang was zero would pass the test above and
never once show the operator the sentence that says he is safe.

### `fn producer_rounding_does_not_make_a_uniform_set_mixed`

A4 in points is 595.2755905511811, and producers write 595.276, 595.28
and 595.32. An exact-equality check would call a set of ten identical A4
sheets "mixed" and put a false sentence on screen for every real
document.

### `fn a_drawing_set_with_a_detail_sheet_reads_as_two_sizes`

The falsifying direction of the test above: a tolerance wide enough to
absorb producer rounding must not be wide enough to call an A3 detail
sheet an A1. The nearest two distinct sizes in the engine's table differ
by far more than a point, which is what makes 1 pt safe.

### `fn the_new_sheet_keeps_the_corner_the_old_sheets_shared`

Both directions, because each is a single missing clause and each
produces a silent shift of the paper relative to the drawing —
invisible on screen and visible on a plot.

### `fn the_verb_reaches_the_document_and_only_shrinking_is_disclosed`

The only test in this module that calls [`set`] — everything above it
exercises the arithmetic the window reads, and none of it would notice
an engine call that was never made or a disclosure wired to the wrong
field.

Both directions, in one document, because that is what makes it a
measurement rather than a coincidence:

* **A4 → A1** grows the sheet. Nothing can fall off, so `lost_area` must
  be false and the operator must be told **nothing** — a window that
  warned about losing content every time it was used would be a window
  nobody reads.
* **A1 → A5** shrinks it, and the lost-area sentence must appear.

A build with the two arms swapped, or with the disclosure raised
unconditionally, passes neither half. A build that never calls the
engine passes neither, because the assertion is on `session.pages()`.

R1 still applies: this is not a report of working software. It cannot
see the ribbon, the window, the operand rule or the save. That is
`ui-verify`'s `resizing_a_sheet_changes_the_paper_in_the_saved_file`,
whose verdict is taken in a different process from a written file.

### `fn a_refused_change_leaves_the_document_exactly_as_it_was`

Measured against the engine (`fixtures/certified-comments.pdf`
→ `CertificationForbidsChange`), and asserted here as the *shape* the
shell relies on: [`set`] propagates the refusal rather than swallowing
it, so `vector_edit` can trace it and
[`crate::text::page_size::refused_certified`] can word it.

ⓘ The blank template carries no certification, so what this can assert
without a certified fixture in this crate is the **other** refusal on
the same path: a degenerate rectangle, raised by `normalize_media_box`
**before anything is touched**. The property is the same one and it is
the one that matters — a refusal leaves the document exactly as it was,
rather than half-resized.

### `fn an_origin_anchored_sheet_is_exactly_the_engines_rectangle`

What makes [`SheetSurvey::target_rect`]'s corner rule safe to apply
unconditionally: on a page at `(0, 0)` — every CAD export in his corpus
— it produces exactly what `PaperSize::rect_with` produces, so the
offset case costs the common case nothing.

### `struct SheetSurvey`

Built once when the window opens, from state the application already holds:
the flattened page vector (walked every frame anyway) and the object-model
cache (built for the current page anyway). **Nothing here decomposes a page
that was not already decomposed** — see [`Self::unread`] for the price that
bounded cost is paid in, and why it is paid in honesty rather than in
silence.

### `fn uniform`

`None` on a mixed pick — the ordinary state of a drawing set with a
detail sheet in it, which is why the window has a sentence for it.
Compared to [`pdfcer_core::paper::PaperSize::CLASSIFY_TOLERANCE`], for
the reason that constant exists: producers round A4 to 595.276, 595.28
and 595.32, and exact equality would call an obviously uniform set
mixed.

### `fn distinct_sizes`

Reported rather than a bare "they differ" because *"9 sheets in 2
different sizes"* tells the operator he has one odd sheet and *"9 sheets
in 7 different sizes"* tells him he has picked the wrong thing.

### `fn overhang`

`None` when nothing could be measured — which is *not* the same as zero
and must not be collapsed into it. The caller words the two differently:
`Some((0,0,0,0))` is [`crate::text::page_size::fits`], a promise;
`None` is [`crate::text::page_size::overhang_unmeasurable`], a stated
boundary.

### `fn target_rect`

# The lower-left corner is the operands' own, not the origin

`PaperSize::rect_with` puts a named sheet at `(0, 0)`, and its own doc
comment calls that *"a choice, not a law"*: §7.7.3.3 does not require a
media box to start at the origin, and imposition output and cropped
scans really do carry offset ones. On such a page, writing an
origin-anchored sheet moves **the paper relative to the drawing** —
which is not what "change the sheet size" means and is invisible until a
print comes out shifted.

So the new sheet keeps the corner the old sheets had, when they share
one. On the overwhelmingly common `(0, 0)` case this is byte-identical
to `rect_with`, which is what makes it safe unconditionally.

When the operands do **not** share a corner, one rectangle cannot
preserve all of them — `set_media_boxes` takes one rectangle for the
whole selection, and that is the property that buys the single undo
entry. The fallback is the origin, and
[`crate::text::page_size::origin_differs`] is drawn in the window rather
than the choice being made quietly.

### `fn survey` — the drawn extent skips paths that paint nothing

`PageObjects::page_bbox` counts `n`-painted paths, so after a scale the
`re W n` clip read as drawing the size of the old visible region. The survey
folds the object bounds itself, skipping `PaintStyle::is_invisible` paths;
reported to the engine as G109.

### `fn survey`

`pages` is the operand list `crate::app::dispatch::pages` already resolved,
so this function never re-decides which sheets are meant — one statement of
the operand rule, which is the same argument
`SelectionState::deletable_objects_on` makes.

# What it costs

One index into `doc.pages` per operand, plus **at most one** borrow of the
object-model cache — `OpenDoc::page_objects` returns the decomposition for
the page currently on screen, already built if the canvas has drawn it. No
page is decomposed on this call's account. See [`SheetSurvey::unread`].
