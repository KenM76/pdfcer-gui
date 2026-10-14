# `pdfcer-gui-base/ocr/manage` and `ocr/addonzip`

**Installing and removing OCR models in `<exe>/models`.** The operator's
Manage OCR models… window (`dialogs::ocr_models`) calls these off the UI
thread. Discovery stays the engine's
(`pdfcer_core::ocr::addons::discover_ocr_models`); this module never decides
what a model is by looking at files itself.

# Supported recognisers

`SUPPORTED` is the manager's list and order: `paddle`, `ocrs`, `ocrcer`,
`paddle-vl`, `tesseract` — the engine tokens. `in_build(token)` says whether
this build can run that kind at all; a `tesseract` add-on carries its own
program and runs through the host, so it is always in the build. Removing
needs no recogniser, which is why `file.ocr_models` is registered in every
build.

# Remove (`remove`)

Deletes the folder of one discovered model, and only when:

1. it is not a search root itself (`IsRoot`) — a folder the operator added in
   Settings is his, not a model folder pdfcer may delete;
2. discovery run again on the same root still finds a model in that folder
   (`Gone`), equal in every field to the one listed (`Changed`). The list the
   operator confirmed is a snapshot; the re-check is what stops a stale row
   deleting whatever is now in that folder.

The folder is renamed aside (`.pdfcer-removing-*`) before deletion. Windows
refuses a rename of a folder with a file in use or without delete rights, so
a refusal (`Refused`) deletes nothing. A failure after the rename
(`Partial`) is reported with the path left behind. Returns the bytes freed,
counted before deletion by `folder_bytes` (regular files under the folder,
recursively; links are not followed). The manifest is deleted first, so a
folder left part-deleted is never discovered as a model.

# Install (`install`)

From a `Source::Zip` (a pdfcer OCR add-on zip) or a `Source::Folder`:

1. unpack or copy into a staging folder `.pdfcer-installing-<pid>` beside
   `models/` — same volume, so the final step is a rename;
2. discovery on the staging folder must find exactly one model (`NoModel`,
   with the engine's notes; `Several`);
3. `OcrModel::verify` checks every `sha256` line of its manifest (`Verify`).
   A bare folder with no manifest (bundled `paddle`) verifies zero files and
   is accepted: there is nothing to check it against, and the trace's
   `checked=0` says so;
4. a model of that name already in `models/` is refused (`Exists`) — replacing
   one is Remove then Install, two decisions the operator sees;
5. rename into `models/<folder name>`.

The staging folder is deleted on every path. Every change that reached the
models folder (an install, or a removal past its rename) bumps
`generation()`, which an open Recognise text dialog compares to rediscover
(`dialogs::ocr_model::ModelPicker::is_stale`).

# Writability (`writable`)

Creates and deletes a probe file in the folder, or its nearest existing
ancestor. Asking the file system is the only answer that covers ACLs,
read-only media and a Program Files install alike; attribute bits do not.
The window states an unwritable folder in words and disables Remove,
Install and Download with the reason on hover.

# The zip reader (`addonzip`)

The workspace has no zip crate, and adding one is an operator decision. An
add-on zip needs only what `zipfile` and the engine's packager write: stored
or deflated entries (inflated with `miniz_oxide`, already a dependency), no
encryption, no ZIP64. Each entry is checked against its CRC-32 and size.
Refused before writing: absolute names, `..`, drive or stream colons, NUL,
duplicates, encrypted entries, other methods, ZIP64 or multi-disk archives,
and more than `MAX_ENTRIES` entries. A refusal mid-archive leaves partial
files in the staging folder, which `install` deletes.
