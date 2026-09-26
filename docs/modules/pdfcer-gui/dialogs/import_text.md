# `dialogs::import_text` — a text file becomes pages

`file.import_text`, the import half of the operator's ask:

> *"also the engine can export PDFs as text. we should have export/import
> for that."*

## It stands on a primitive only the engine can own

**Making a sheet out of nothing is `pdfcer-core`'s verb.** Copying a page,
inserting one from another document and deleting one all rearrange pages
that already exist; `blank_document` creates one, and `place_text` is the
paginating placer built on it. A shell cannot stand in for either without
writing a second page writer, which is the one thing this crate must not do.

⇒ The standing rule that follows, and the reason a window like this arrives
all at once rather than in halves: **a capability the engine does not have is
disclosed in words, never as a control that declines when pressed.** R9 greys
what is *temporarily* unavailable, and an absent engine verb is not that.

## This window is a CHOOSER, and that inverts its sibling's job

`export_text` names **losses** — layout, fonts, position — because exporting
throws things away. Importing **invents**: a sheet size, margins, a
typeface, a size, a leading, an alignment, none of which is in the text file.
So every control here is a decision somebody has to make, the defaults are
answers to *"what would he have picked?"*, and only one sentence warns about
anything. `text::import_text`'s header carries that argument in full.

## The defaults, and each one is a choice rather than an inheritance

| control | default | why |
|---|---|---|
| sheet | **A4** | `PageTemplate::new()` is US Letter; this operator's world is metric and every other sheet chooser in this program opens on a metric size |
| margin | 72 pt | one inch, the engine's `DEFAULT_MARGIN_PT`, and the number a reader expects on a page of prose |
| font | Helvetica | the engine's own default, and the face a plain text file most often wants |
| size | 11 pt | the engine's is 12; 11 fits a 66-character line on A4 at one-inch margins, which is the classic measure for readable prose |
| position | **after the current page** | `insert_pages`' default, and for its stated reason: it is what *"insert here"* means to somebody who navigated to a sheet first |

**The sheet list is `pdfcer_core::paper::PaperSize::ALL`**, rendered by
`text::new_document::size_name` — the same list and the same labels the New
Document and Page Size windows use. Three surfaces, one answer to *"what is
A1?"*, and an operator who learned the list in one meets it unchanged in the
others. A fourth private table here would be a fourth chance to disagree
about a sheet's dimensions.

## What this window does NOT do, and the reason is the engine's

**It does not read the file.** Not to count its lines, not to preview its
pagination, not to check its characters against the chosen face. Every one of
those means running the import to draw a window that offers to run the
import — and `place_text` **plans before it writes**, refusing with nothing
created, so the answers arrive as real numbers a moment later instead of
provisional ones now.

⇒ The file is read once, in the apply arm, and the report is the receipt.

## Item notes

### `const FACES`

# Five of the fourteen, and the narrowing is the design

`Std14` has fourteen members; twelve are text faces and two (`Symbol`,
`ZapfDingbats`) are pictorial. A chooser listing all fourteen would offer
**Dingbats** for a page of prose, which is not a choice anybody is making,
and would offer four Helvetica variants where the bold and oblique ones are
*emphasis* rather than a body face — and this import sets one face for the
whole document, so emphasis has nothing to contrast with.

⇒ What is left is the actual decision: **serif, sans, or monospace**, plus
the two bolds an operator might want for a short notice. `Courier` earns its
place because a text file is very often a listing, a schedule or a register
where columns aligned with spaces only survive in a monospaced face — which
is the one case where the *font* changes whether the import is readable.

### `const SIZE_RANGE`

Six is below the smallest an operator would set for body text and is where
a `PageTooShort` refusal stops being plausible; seventy-two is one inch, past
which a single line no longer fits an A4 measure and the import becomes one
word per page. Both ends are far outside anything reasonable **on purpose**:
this is a guard against a scrub running away, not a judgement about
typography.

### `const MARGIN_RANGE`

Zero is legal and is what somebody importing a listing to be re-cropped
wants; the ceiling is a quarter of A4's short edge, past which the column is
narrower than the margins around it and `PageTooShort` becomes likely. The
engine refuses that case by name and this range makes reaching it a
deliberate act rather than an accident of a scrub.

### `enum Where`

A local enum **only** for the radio state, converted to
`pdfcer_core::pageops::InsertPosition` at the point of use — copied
deliberately from `dialogs::insert_pages`, whose own note gives the reason:
two of the four need the current page index, which the radio does not carry
and the dialog does.

⇒ It is a *copy* rather than a shared type, and that is worth defending:
the four variants are a fact about **this program's vocabulary for
inserting**, and both dialogs converting the same four into the same engine
enum at the same place is the property that matters. A shared enum would add
a module for four unit variants and would not make the two windows any more
alike than reading them side by side already does.

### `fn template`

Built from `PageTemplate::new()` and then overridden, rather than
constructed field by field. `PageTemplate` is the engine's type and it
gains fields — `leading`, `alignment`, `color` and `unmappable` are all
left exactly as the engine set them, which is the whole point: this
window chooses the four things it draws controls for and takes the
engine's answer for everything else, so a field added tomorrow arrives
with the engine's default rather than with a zero this shell invented.

### `fn sheet_of`

Clamped rather than indexed, because `PaperSize::ALL` can shrink between
builds as well as grow — the engine says the table moves — and a stored
index from a longer list must not panic a window open.

### `fn default_sheet`

**A4, found by id rather than by position.** `PaperSize::ALL`'s order is
the engine's business and it has said the table will grow; a hard-coded
index would silently open on a different sheet the day one is inserted
before A4 — and a window that opens on the wrong paper is a defect an
operator only notices after importing.

Falls back to the first entry, which cannot be wrong in a way that matters:
the chooser is right there and the sheet is the first thing in the window.

### `fn face_name`

A one-line adapter over `text::import_text::face_name`, and the clamp is
the whole reason it exists: this window stores an INDEX, and a stored index
from a longer list must not panic a window open. The words themselves live
in the catalogue rather than here, because `check-ui-strings` reads the
catalogue — a displayed label spelled in this file is one the gate reports.
