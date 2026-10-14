# `ui-verify/checks/ocr_models_manage`, `ocr_models_unwritable`, `ocr_models_common`

Two checks of File ▸ Recognise ▸ Manage OCR models…, and what the manager
checks share.

# `ocr_models_common`

Each check runs its own copy of the binary in `<out>/<tag>/`, beside its own
`models/` and `userdata/`. The manager deletes and installs in
`<exe dir>/models`, so driving the build's own copy would delete the models
every other check needs, and a shared preferences file would let one run's
default leak into the next. Planted models are copied from the build's own
`models/` (`Program::copy_reference`); that folder missing is an error, not a
pass.

`Drive` launches on `fixtures/synthetic-image-only.pdf` (so Recognise text is
enabled and Find on it finds nothing), off the desktop, with a
`ScriptedPointer`; it clicks named regions and waits on trace events after a
mark. `open_manager` goes File tab → the collapsed Recognise group if needed →
`ribbon.item.file.ocr_models`, and returns the `ocr-models-listed` line.
`finish` parks the pointer, but a found failure wins over a parking error.

# `ocr_models_set_default_and_remove_down_to_none`

Plants `paddle`, `ocrs` and `ocrcer`, then:

1. **The list.** Every planted model is in `names=` and draws
   `ocr-models.row.<name>`; `paddle-vl` and `tesseract` are in `missing=` and
   draw `ocr-models.missing.<token>`.
2. **Set as default** on a planted model that is not the default: the
   `ocr-models-default` line names it, the next listing's `default=` is it,
   and the copy's `preferences.txt` holds `ocr_model = <name>`.
3. **Remove that default.** `ocr-models-remove-asked` comes first with
   `bytes=` equal to the folder's size counted by the check, and the folder
   is still there. After Remove: the folder is gone, `freed=` equals that
   count, and `default=` names one of the remaining models — not `none`, not
   `unchanged`. `ocr-models.result` is drawn.
4. **Remove all**, asked then confirmed: no planted folder is left,
   `default=none`, and the next listing has `installed=0`.
5. **Recognise text with nothing installed** (Close, then
   `ribbon.item.file.ocr`): `ocr-no-engine none-installed=true` and
   `ocr-open-manager` drawn; pressing it lists the manager again and the
   program is still running.
6. **Find's offer** (close both windows with `ocr-models.close` and
   `ocr-close`, Ctrl+F, type `e`, Enter): `find-ocr-offer.accept` is drawn on
   the textless page, and pressing it gives `ocr-no-engine
   none-installed=true` again.

Falsified, each restored by copying the file back:

- Remove deleting on the first press (`start_removal` called beside `ask`):
  red at step 3, folder deleted before the operator confirmed.
- The no-engine sentence switched off (`none` forced false): red at step 5,
  `none-installed=false`.
- Find's offer not raising `file.ocr`: red at step 6.
- A dialog-raised command left to the apply phase (the frame's second drain
  absent): red at step 5, the button opened nothing; the trace shows
  `command-action-undrained id=file.ocr_models`.

# `ocr_models_unwritable_folder_is_stated`

Plants `ocrs`, then denies Everyone *create files* and *create folders* on
`models/` itself (`icacls /deny *S-1-1-0:(WD,AD)`), which is exactly what the
program's probe exercises. The deny is lifted with `icacls /remove:d` before
the verdict; one left by an interrupted run does not stop the next run
deleting the folder, because deletion needs neither right.

Asserts the listing says `writable=false` and draws
`ocr-models.unwritable`; that pressing Remove on `ocrs` asks nothing; and
that `models/ocrs` is still there.

Falsified: `manage::writable` answering true on a refused probe — red,
`writable=true` and no sentence.
