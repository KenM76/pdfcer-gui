# `ui-verify/checks/ocr_fetch_models`

Two checks over File ▸ Recognise ▸ Manage OCR models…'s Download rows, one
per pinned set (`FETCHABLE_MODELS`):

- `ocr_models_download_into_the_models_folder` — `ocrs`, whose files the
  build packages, so the download is compared byte for byte.
- `a_paddle_set_downloads_and_is_listed_by_name` — `ppocrv5-latin`, which
  the build does not package; the manifest and the listing are the evidence.

# What it drives

A copy of the binary in `<out>/fetch-models-<set>/`, so the download lands
in that folder's `models/<set>` and never in the build's own. Off the
desktop under `--no-input` with a `ScriptedPointer`, no document open:

1. File tab, the collapsed Recognise group if needed, then
   `ribbon.item.file.ocr_models`.
2. `ocr-models.download.<set>`, then up to four minutes for
   `fetch-models-done` or `fetch-models-failed`. It must be `done` with the
   set's `name=` and its pinned file count.
3. `ocr-models.result` must be drawn at least 60% visible: the window
   scrolls a new result into view, as the list is taller than the window.
4. The newest `ocr-models-listed` after the click must name the set, so the
   window counts it installed and stops offering it.
5. Every pinned file and `pdfcer-ocr-model.txt` with the line
   `name = <set>` must be in `models/<set>`.
6. For `ocrs`, each file must equal the copy in the build's own
   `models/ocrs` (the packaged pins).

# Needs

The network. For `ocrs`, the reference copies in the driven build's
`models/ocrs`; without them the check is an error, not a pass.

# Falsified

The manifest written empty: discovery rejects the folder, and both checks
fail at step 4 with `names=none`.

The result drawn without scrolling to it: both fail at step 3, the line
36% visible below four Download rows.

The download written to the wrong folder (`target_in` joining a different
sub-folder): step 5 fails with the files not where Recognise text looks.

The window drawn below `Dialogs::show`'s no-document guard: with nothing
open it never appears, and step 2 fails with no Download button.
