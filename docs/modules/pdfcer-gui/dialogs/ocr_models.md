# `pdfcer-gui/dialogs/ocr_models`

**File ▸ Recognise ▸ Manage OCR models…** (`file.ocr_models`): add, remove
and choose the default OCR model, down to none installed, so an operator with
no use for OCR can reclaim the space (O293).

# What it shows

A group per recogniser in `ocr::manage::SUPPORTED` order. Each installed
model is a row: name, version, size, folder, and — when the engine refuses to
run it — the refusal's own sentence. A recogniser with nothing installed
says *not installed*, then *not in this build* or how to install it from
an add-on. Under every recogniser in the build, each of its
`FETCHABLE_MODELS` sets that is not installed gets an *Available to
download* row (label and languages) with a Download button, when the build
has `model-download`. A set counts as installed when a discovered model
carries its `name`; the bundled `paddle` matches no pinned set by bytes, so
names are the only honest test. Models discovered
under an engine token outside `SUPPORTED` are listed after the groups, so
nothing installed is ever hidden. The list scrolls; below it, pinned
(`Host::scrolled`), are the count and total size, Remove all, Install from
file…, Install from folder…, the pending removal's question, the result and
Close. Every list holds more rows than the window at its opening size, so a
question or result drawn after the list would land below the fold, and a
Remove pressed would look like a button that did nothing.

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
way to put back a model no pinned set covers (`paddle-vl`, `tesseract`).

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
- `fetch-models-started engine= name= dir=`, then
  `fetch-models-done engine= name= files=` or
  `fetch-models-failed engine= name= why=` for a download.

A download writes the set's files into `models/<folder>`, then the set's
`FetchableModels::manifest` as the add-on manifest, so the model lists with
its version and is checked by the engine like an installed add-on.

Regions: `ocr-models.row.<name>`, `ocr-models.missing.<token>`,
`ocr-models.default.<name>`, `ocr-models.is-default.<name>`, `ocr-models.remove.<name>`,
`ocr-models.download.<set name>`, `ocr-models.confirm`, `ocr-models.keep`,
`ocr-models.remove-all`, `ocr-models.install-file`,
`ocr-models.install-folder`, `ocr-models.result`, `ocr-models.unwritable`,
`ocr-models.close`.

# With no model installed

Recognise text says *no OCR engine is installed* (not the folders the search
looked in) and offers **Manage OCR models…**, which raises
`Action::Command("file.ocr_models")`. Find's Recognise text offer reaches the
same dialog through `file.ocr`. Commands raised by a dialog are dispatched by
the frame's second command drain, after the dialogs draw (`app::frame`).
