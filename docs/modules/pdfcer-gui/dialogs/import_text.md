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
