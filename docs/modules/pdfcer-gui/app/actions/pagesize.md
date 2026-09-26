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

Through the engine's own `set-page-size` (which calls the same
`EditSession::set_media_boxes` this module calls), on
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

## 2. Why there is no "scale to fit", and what it would have cost

The engine has no scale-to-fit verb, and composing one here would not have
been a small thing. `EditSession::transform_objects` can wrap a selection of
**page objects** in `q <cm> … Q` — kind-agnostic, one undoable command — but
it takes explicit object indices on **one** page and does not touch
annotations, form-field widgets or ce dimensions, which have their own five
verbs (`move_annotation`, `resize_annotation`, `move_widget`,
`move_dimension`, `set_group_scale`). A true scale-to-fit is therefore six
verbs composed across N pages, each with its own refusals, and its most
likely failure — a ce dimension group left at the old calibration — produces
a drawing that *prints a wrong measurement and looks perfectly correct*.

⇒ **pdfcer changes the paper.** The decision is not "scale-to-fit is hard",
it is that a half-built scale-to-fit is the worst artefact in this problem
space. What is built instead is the thing that makes the crop survivable:
the operator is told, **before he commits and in points**, exactly how far
his drawing runs past the paper he has picked. See [`survey`].

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

`EditSession::set_media_boxes` — *"a sheet set is resized as a set"*, one
undo entry however many pages, refusals raised **before anything is
committed** so an out-of-range index leaves the document untouched rather
than half-resized. Calling the singular verb in a loop would be functionally
identical and would leave the operator pressing Undo once per sheet. Its
own doc comment names this shell as the caller it was written for.
