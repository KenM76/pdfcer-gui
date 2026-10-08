# `declined::remedy` — the button beside a decline

`Declined::remedy` returns the id of the command whose dialog removes the
decline's cause, or `None`:

| Decline | Remedy |
|---|---|
| A text click on a picture of text, or on a page with no readable text | `file.ocr` (File ▸ Recognise text…) |
| A text edit or a re-wrap refused because the document is protected | `file.encrypt` (Security ▸ Encrypt…, which can remove the protection) |

`app::status::decline::show` draws the command's registered label beside the
sentence, publishes it as region `status-group:decline.remedy`, and dispatches
the command on a click. A command this build does not register draws nothing.
