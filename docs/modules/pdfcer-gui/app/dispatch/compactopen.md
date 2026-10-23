# `app::dispatch::compactopen` — the commands that open *Save a compacted copy*

Three registered commands open the one compact window, each at its own
starting point (`dialogs::compact_target::Start`):

| command | label | opens at |
|---|---|---|
| `file.save_compacted` | Save a compacted copy… | Lossless, no size target: nothing on any page changes |
| `file.reduce_size` | Reduce file size… | the Standard preset: pictures above 225 ppi scaled to 150, thumbnails removed |
| `file.fit_size` | Fit to a size… | Lossless with *Fit under* on, at half the plain copy |

All three are on File ▸ Save and on a document tab's right-click menu. They
are three commands rather than one with an argument because R8 makes a
registered command the only way a surface learns a capability exists, and a
menu row names a command, not a command plus an operand.

## Contract

- **The operand is the right-clicked tab** when `PdfcerApp::tab_menu_target`
  is set: that tab is activated first, so the window measures the document the
  operator pointed at. From the ribbon it is the document on screen.
- **A refusal is recorded, not swallowed.** `pdfcer-core` refuses a full
  rewrite for a hybrid file whose `/XRefStm` does not parse and for one whose
  object numbering is too sparse for a single-section table; the sentence goes
  to the status record and to the trace as `compact-refused`.
- **A window already open is left as it is.** A second command does not reset
  choices the operator has made in it.

Driven by `ui-verify` check `size_reduction_is_on_the_ribbon_and_the_tab_menu`.
