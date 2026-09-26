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

### `enum PageSeparator`

Two values rather than a `bool`, because the two are not *"a marker, on or
off"* — they are two different characters-in-the-file, one of which is the
engine's and one of which is pdfcer's own prose. A `bool` would have made
the second look like a formatting preference rather than like added content.

### `struct TextExportPlan`

Rides `super::write::WriteAction::Text`, so `Clone` and `PartialEq` for that
enum's derives — [`super::imageexport::ImagePlan`]'s reason, stated there.

The pages are **resolved**, not a scope and a string. The window has
already parsed the typed range — it needs the answer to decide whether
Export is pressable — so re-parsing in the apply phase would be a second
reading of the same box against a document that may have changed pages in
between.

### `fn suggested_path`

Beside the document, named after it, with `.txt`.

# `set_file_name`, NEVER `set_extension` — and this is a defect that
has already shipped once in this crate

`Path::set_extension` replaces everything after the **last** dot. A document
called `plan.rev2.pdf` has a stem of `plan.rev2`, which contains a dot, so
`set_extension("txt")` produces **`plan.txt`** — the revision is silently
deleted.

That is not a cosmetic loss. `plan.rev2.pdf` and `plan.rev3.pdf` would both
suggest `plan.txt`, so exporting the second **overwrites the first**, in a
save dialog whose only warning is the operating system's generic *"a file
with that name already exists"*. `.rev2` / `.rev3` is an ordinary CAD naming
shape, and the two files that collide are the two an operator is most likely
to want side by side.

A `set_extension` here would truncate at the first dot, and the mistake is
invisible to review because the comment asserting *"appending would produce
`plan.rev2.txt` either way"* reads as true until the helper meets a stem
with a dot in it. This function is written the safe way and
[`tests::a_revision_in_the_stem_survives_the_suggested_name`] is the test
that keeps it that way — **a claim in a comment is not a test.**

A document with no extension at all still gains one: the stem of `plan` is
`plan`, and the format appends unconditionally.

### `struct Assembled`

A struct rather than a tuple because the caller has to pick a *different
sentence* per fact and three of the four are optional — a `(String, Vec,
usize, usize)` would put four unlabelled positions where a reader needs
four names.

### `fn assemble`

Takes `(one-based page number, that page's text)` pairs rather than the
engine's `PageText`, which is what makes this function pure and testable
without a PDF on disk. The caller does the `plain_text()` call; this decides
what goes *between* the results and counts what came out empty.

# The algorithm, and the two things it must not get wrong

1. **A separator goes between pages, never before the first or after the
   last.** `plain_text()`'s own rule. A
   leading form feed makes a one-page export start with a page break that
   means nothing; a trailing one leaves every file ending in a phantom page.
2. **An empty page still counts as a page.** It contributes a separator and
   a marker like any other, so page 5 of a six-page export is where page 5
   is even when page 4 was a scan. Skipping it would silently renumber the
   file.

The marker replaces the form feed rather than joining it. Writing both
would give a reader two page boundaries per page and a `split('\u{000C}')`
that no longer lines up with the visible marks.

### `fn encode`

Two transformations, in this order, and the order matters:

1. **Line endings.** `\n` becomes `\r\n` when asked. Done on the text so a
   `\r` the *document itself* contained is not doubled — the replacement
   matches a bare `\n`, and any `\r\n` already present is normalised through
   a `\n` first rather than becoming `\r\r\n`.
2. **The byte-order mark**, prepended to the finished bytes. It must be the
   first three bytes of the file or it is not a byte-order mark.

UTF-8 throughout, because that is what a Rust `String` is and what the
window promised. There is no code-page option and there will not be one: a
CAD drawing carries degree signs and diameter marks, and offering an
encoding that cannot represent them is offering a way to lose them.
