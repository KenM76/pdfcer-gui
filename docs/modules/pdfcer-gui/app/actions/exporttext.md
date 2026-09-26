# `app::actions::exporttext` — the plan a text export is made of, and the
pure parts of making one

`file.export_text`, on the operator's ask:

> *"also the engine can export PDFs as text. we should have export/import
> for that."*

This module is the **shell-side value** a text export is described by, plus
everything about producing one that can be computed without a `Document`:
which pages, how the pages are joined, how the bytes are encoded, what the
file is called, and what came out empty. [`super::export::text`] does the
parts that need the open document — the extraction, the picker and the
write.

The split is [`super::imageexport`]'s, and for its reason: the decisions are
testable without a PDF and the extraction is not, so the decisions get tests
and the extraction gets one call site.

---

# THE IMPORT HALF IS ONE FEATURE OF THREE, AND THIS SAYS WHICH

The operator asked for **"export/import"**, one word with a slash in it.
*"Import text"* is three different features wearing one name, and the
engine offers one of them — which is the one that ships. The next reader's
first question is going to be *"where is the import"*, and the answer
depends entirely on which of the three they meant:

| what an operator could mean | the nearest verb | why it is not that feature |
|---|---|---|
| **Make a PDF out of a text file** | `EditSession::place_text`, plus `blank_document` for the page itself | **This one ships**, as `file.import_text` — see [`crate::app::actions::importtext`]. `place_text` paginates, which is the property that makes it a feature rather than a trap: `EditSession::add_text` is one page, one call, at coordinates, and a two-page text file put through it would paint the second page's words off the sheet and report success. |
| **Replace a page's text with a text file's** | `EditSession::edit_text` | Addresses **one located run**, via a `find` string or a pinned operator span. There is no *"replace page N's text with this string"*. And the mapping cannot be reconstructed from an export: `plain_text()`'s line breaks and word spaces are pdfcer's own derivation (negative result S5), one glyph is not one character (§9.10.3), and 13 % of runs carry glyphs from more than one show operator (`operator_span_invariant.rs`, measured over 4,289 fixtures). A round trip built on that would edit the wrong text and say it had succeeded. |
| **Put a text layer over a scan** | `EditSession::add_ocr_layer` | Takes `&[OcrPageLayer]`, whose one payload field is `recognised: &crate::ocr::OcrPage` — **positioned words**, produced by the recogniser from the raster. A `.txt` file has no positions, so there is nothing to hand it. This is `file.ocr`, and it already ships. |

⇒ **Nothing is faked for the two that do not ship.** No half-import is
drawn, no control declines when pressed, and the window says nothing about
a round trip.

---

# The default plan writes the CLIPBOARD's own bytes

`file.copy_document_text` puts `extract_document_view(…).plain_text()` on
the clipboard. At [`TextExportPlan`]'s defaults this export writes that
same string,
byte for byte — same extraction options (the settings funnel), same
`plain_text()`, same U+000C between pages, no BOM, no line-ending rewrite.

Every departure is opt-in and every one is named in the receipt afterwards.
Two answers to *"what is the text of this document"* inside one program is
worse than either answer alone, because both of them look like text.

## Item notes

### `fn a_revision_in_the_stem_survives_the_suggested_name`

The DXF path shipped this bug for weeks behind a comment asserting the
opposite. It is asserted here rather than described, because the whole
lesson of that incident is that a claim in a comment is not a test.

The second half of the assertion is the one that makes it a **data-loss**
test rather than a cosmetic one: two revisions of the same drawing must
not suggest the same output name, because the save dialog's only
protection is the operating system's generic overwrite warning.

### `fn the_typed_range_is_the_print_dialogs_parser`

Asserted here as well as in `dialogs::print::tabs` because the claim
being made is not *"the parser works"* — that is tested there — it is
*"this feature reaches THAT parser"*. A second implementation would pass
the first assertion and fail the intent, and the way it would be
noticed is by these expectations diverging from the print window's.

The expectations are the print dialog's own: `5,1-2` keeps the order
typed, `1,1` is two entries and not one, and a range past the end
refuses the whole spec rather than clamping — *"clamping would turn a
typo into a job."*

### `fn the_form_feed_separates_and_does_not_bracket`

This is `plain_text()`'s own `if i > 0`, and asserting it here is what
keeps this export producing the clipboard's own string rather than one
that merely resembles it.
