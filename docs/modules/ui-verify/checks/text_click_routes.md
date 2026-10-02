# `a_text_tool_click_reaches_what_is_under_it`

Three launches of the release binary, off-screen, in Edit mode with the Edit
Text tool armed (`PDFCER_DIAG_INVOKE=mode.edit,edit.text`), each clicked once
through the scripted pointer:

| Fixture | Click (PDF points, page 1) | Passes when |
|---|---|---|
| `synthetic-image-only.pdf` | (153, 198), the full-page image | `text-edit-declined reason=PictureOfText`; region `status-group:decline.remedy` is drawn; clicking it declares `ocr-dialog` |
| `three-text-fields.pdf` | (165, 237), inside `FieldOne` | `text-click-routed to=field` and `form-focus` |
| `annots-with-everything.pdf` | (200, 420), inside the FreeText | `text-click-routed to=note` and `note-popup-toggle` |

In all three, no `text-edit-became-add` line may follow the click: that line
is the click silently starting new text, which is the defect.

The remedy button is drawn only when `file.ocr` is registered, so a build
without OCR fails the scan case by name rather than passing over an absent
button.
