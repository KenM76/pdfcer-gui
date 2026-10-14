# `pdfcer-gui/dialogs/ocr_models`

**File ▸ Recognise ▸ Manage OCR models…** (`file.ocr_models`): add, remove
and choose the default OCR model, down to none installed, so an operator with
no use for OCR can reclaim the space (O293).

# What it shows

A group per recogniser in `ocr::manage::SUPPORTED` order. Each installed
model is a row: name, version, size, folder, and — when the engine refuses to
run it — the refusal's own sentence. A recogniser with nothing installed
says *not installed*, then, in order of preference: a Download button
(`FETCHABLE_MODELS` has pinned files and the build has `model-download`),
*not in this build*, or how to install it from an add-on. Models discovered
under an engine token outside `SUPPORTED` are listed after the groups, so
nothing installed is ever hidden. Footer: the count and total size, Remove
all, Install from file…, Install from folder…, Close.

The models folder's path is printed at the top; when the program may not
write there (`manage::writable`) a sentence says so in words, and every
button that would write is disabled with that reason on hover. Greying
without the sentence would leave the operator guessing at a permission he
can fix.

# Default

The default is the existing `ocr_model` preference that Recognise text
starts from (`ocr::catalog::start`). There is no second preference: two
would disagree. The default row is marked; every other runnable row has
**Set as default**, which writes the preference at once and traces
`ocr-models-default name= saved=`.

# Remove

Remove (one) and Remove all both ask first, in the window: what goes, from
where, and how many bytes. Nothing is deleted until Remove is confirmed;
Keep cancels. A model whose folder is a search root (an extra folder from
Settings) has Remove disabled with the reason — that folder is the
operator's, not one pdfcer installed.

After a removal the window says what went, the space freed, and — when the
default went — what Recognise text starts with now, or that no model is
left. Removing the default clears the preference rather than choosing a
successor: `catalog::start` then picks the engine's first runnable model, and
the sentence names it. A remembered model that is missing would otherwise
make Recognise text choose nothing (`catalog::start` rule 2).

# Install

From a pdfcer OCR add-on zip or a folder, through `ocr::manage::install`:
staged beside `models/`, discovered, verified against its manifest's
`sha256` lines by the engine, then renamed in. A bare folder such as the
bundled `paddle` has no manifest and installs with `checked=0`; it is the
only way to put a removed `paddle` back until the engine pins a download for
it (G194).

# Work off the UI thread

Removals, installs and downloads run on a worker (`jobs`), one at a time;
every writing button is disabled while one runs. Each ends with one
`Finished`, the window rediscovers, and `jobs::sentence` writes the result
line. `ocr::manage::generation` changes with every install or removal, so an
open Recognise text dialog rediscovers on its next frame.

# Trace

- `ocr-models-listed installed= names= default= bytes= writable= missing=`
  on every discovery;
- `ocr-models-default name= saved=`;
- `ocr-models-remove-asked name= bytes=`, `ocr-models-remove-kept`,
  `ocr-models-removed names= freed= default=<new|none|unchanged> failed=`;
- `ocr-models-installed name= checked= bytes= folder=`,
  `ocr-models-install-failed why=<zip|io|no-model|several|verify|exists>`;
- `fetch-models-done` / `fetch-models-failed` for a download.

Regions: `ocr-models.row.<name>`, `ocr-models.missing.<token>`,
`ocr-models.default.<name>`, `ocr-models.is-default.<name>`, `ocr-models.remove.<name>`,
`ocr-models.download.<token>`, `ocr-models.confirm`, `ocr-models.keep`,
`ocr-models.remove-all`, `ocr-models.install-file`,
`ocr-models.install-folder`, `ocr-models.result`, `ocr-models.unwritable`,
`ocr-models.close`.

# With no model installed

Recognise text says *no OCR engine is installed* (not the folders the search
looked in) and offers **Manage OCR models…**, which raises
`Action::Command("file.ocr_models")`. Find's Recognise text offer reaches the
same dialog through `file.ocr`. Commands raised by a dialog are dispatched by
the frame's second command drain, after the dialogs draw (`app::frame`).
