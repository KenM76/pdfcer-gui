# `app::overprintopen` — the overprint default, applied as a document opens

`seed` runs from `PdfcerApp::adopt`, after the page display and off-page
seeds, and sets `view.overprint_preview` from Settings ▸ Display ▸ *Use
overprint preview*:

| choice | read | preview |
|---|---|---|
| Only for PDF/X files | `overprintseed::claims_pdfx`, at once | on when the file claims PDF/X |
| Always / Never | nothing | on / off |
| Automatic | the overprint walk, on a thread | on until the answer arrives; then on only if something asks, or if the walk could not finish |

`PdfcerApp::overprint_seed_poll` runs every frame for the document on screen.
While its walk runs it asks for a repaint every 100 ms; when the answer
arrives it applies it, unless the operator has switched the preview since it
was seeded. A parked document's answer waits on its channel and is applied
when its tab is shown.

**Unknown is on, and is said.** A walk stopped by an unreadable stream or the
engine's budget proves nothing, and the page as it prints is the safe
reading. While the preview is on for that reason the status line says
*Overprint preview is on — Automatic could not check all of this file for
overprint* (region `status-group:overprint`, shared with the line for the
preview being off, which can never be up at the same time).

Trace, one line as the document opens and one more when *Automatic*'s walk
answers:
`overprint-default choice=<id> pdfx=<0|1> asks=<0|1|?|-> on=<0|1>`, the second
with ` unwalked=<n> switched=<0|1>`. `asks` is `?` while unknown and `-` when
the choice does not read the document.

Driven by `ui-verify --check overprint_default`.
