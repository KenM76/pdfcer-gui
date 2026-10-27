# `pdfcer-gui-base/clipboard/richtext`

**Copied text keeps its formatting and its paragraphs.** The module takes a
selection, a page or the whole document. It builds the engine's `RichText`
from it and places that as `HTML Format`, `Rich Text Format` and
`CF_UNICODETEXT` in one clipboard transaction, which is what Word, Outlook and
a plain editor each read.

## Contract

- `page(doc, page, range)` returns the page's `RichText`, or `None` when the
  page has no provenance extraction. `range` is an ordered
  `(start, end)` pair of `TextPosition`s, or `None` for the whole page.
  The caller falls back to a plain copy on `None`.
- `document(doc)` extracts every page with provenance under the operator's
  extraction settings. It joins the pages' paragraphs in page order, and its
  `Err` is a sentence for the status line.
- `place(rich)` is all or nothing; `Refusal` leaves the clipboard as it was.
  `Ok` lists the format names placed. Both registered strings are
  NUL-terminated, and the text is UTF-16LE with a NUL, as the formats
  require.

## Why the provenance extraction

The model the reflow editor builds reads glyph provenance. It uses that to
join wrapped lines into paragraphs and to name each run's font, so the copy
reads the extraction the reflow editor reads (`OpenDoc::provenance_page_text`)
rather than the plain one. The sources are the page's fonts (`PageFonts::read`)
and its underline and strike-through decorations (`page_decorations`). A page
whose decorations cannot be read is copied without them, not refused.

## Disclosure

`RichText::outcome` counts three inferences:

- a bold or italic weight taken from the font's name;
- a family no installed font carries;
- a colour that is not DeviceGray or DeviceRGB, which is not carried.

`text::clipboard::rich_copied` turns non-zero counts into one status-line
sentence. The pasted text itself is never marked.

## Placement

With `PDFCER_DIAG_CLIPBOARD_DIR` set, `place::put_entries` writes the three
formats to that folder instead. `ui-verify`'s `rich_copy` check reads them
there.
