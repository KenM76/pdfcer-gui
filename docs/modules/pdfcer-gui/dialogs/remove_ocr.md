# `dialogs::remove_ocr` — File ▸ Remove OCR text

`OPERATOR_REQUESTS.md` O289 item 14: Remove OCR text takes the same page
choices as Recognise text.

## What it asks

- **Which pages** — `dialogs::page_scope`, the group Recognise text draws.
- **Which recognisers' text**, only when the document holds layers from more
  than one. Each layer pdfcer writes records its `/Engine`; a layer that
  recorded none is its own row, "Not recorded". With one recogniser the
  question has one answer and is not drawn (R9).

The window states what the answer names — *"This removes N OCR text layer(s)
on M page(s)"* — and greys Remove while that is none.

## Opened only when there is something to remove

`open_for` lists the document's pdfcer OCR layers once
(`EditSession::find_ocr_layers`). With none, or a page tree that cannot be
walked, no window opens: dispatch sends the unfiltered
`Action::RemoveOcrLayers { pages: None, engines: None }`, whose refusal
sentence says the document has no OCR text pdfcer added. A window whose only
content is "there is nothing here" would be a modal in the way of a status
line.

## The removal

Remove pushes `Action::RemoveOcrLayers` carrying the pages and engines;
`app::actions::ocrlayers::remove` filters the layers it finds by both and
removes the rest exactly as before: one undo entry, and any layer (group) left
empty is deleted in the same step. Layers other software wrote are never
found, so never offered.

## Trace and regions

| Name | What |
|---|---|
| `remove-ocr.body` | the window body |
| `remove-ocr.scope`, `.all`, `.current`, `.picked` | the page group and its radios |
| `remove-ocr.engine.<token>` | one recogniser's checkbox |
| `remove-ocr.commit` | Remove |
| `remove-ocr-named layers= pages=` | what the answer names, on change |
| `remove-ocr-requested pages= engines=` | Remove pressed |
| `remove-ocr-layers-applied removed= pages= groups-deleted=` | the removal (`ocrlayers`) |

Driven by `remove_ocr_text_takes_the_pages_chosen`.
