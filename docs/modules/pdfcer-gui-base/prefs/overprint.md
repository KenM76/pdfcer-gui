# `prefs::overprint` — whether a document opens with overprint preview on

`Prefs::overprint_default: OverprintDefault`, file key
`overprint_preview_default`, tokens `pdfx` | `always` | `never` | `automatic`,
default `always`. Shown in Settings ▸ Display as **Use overprint preview**, the
four choices in Acrobat's order and wording.

| Choice | Opens with the preview on when |
|---|---|
| Only for PDF/X files | the file claims PDF/X (`PrintClaims::pdfx`) |
| Always | every document |
| Never | no document |
| Automatic | some page asks for overprint (`PrintClaims::overprint`) |

**Default `always`, not Acrobat's `pdfx`.** pdfcer drew overprint simulated
before this setting existed, and a page that asks for no overprint draws the
same either way, so `always` changes nothing for a file without overprint and
shows a file with it as it prints. Acrobat's choice is one click away.

**Decided once, at open.** `app::lifecycle::seed_overprint` resolves the
choice into `view.overprint_preview` as the document opens, after the page
display and off-page seeds. View ▸ Display ▸ Overprint preview flips it from
there; nothing re-applies the default to an open document, and changing the
setting affects the next document opened.

**The document is read only when the answer depends on it**
(`reads_document`: `pdfx` and `automatic`), through
[`printclaims`](../printclaims.md). `always` and `never` cost nothing.

Trace, one line per open:
`overprint-default choice=<id> pdfx=<0|1> asks=<0|1> on=<0|1>` — `pdfx` and
`asks` are `0` when the document was not read.

Settings regions: `settings.display.overprint.<id>`, one per radio, declared
while visible.

Driven by `ui-verify --check overprint_default`.
