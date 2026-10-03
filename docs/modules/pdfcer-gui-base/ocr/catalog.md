# `pdfcer-gui-base/ocr/catalog`

**The models Recognise text can offer, and whether this build can run each.**
Discovery is the engine's: `pdfcer_core::ocr::addons::discover_ocr_models`
(decision 182) walks the roots for `pdfcer-ocr-model.txt` manifests and bare
`ocrs`/`ocrcer`/`paddle` folders. This module adds runnability and the
starting choice; it does not walk folders itself.

# Roots

`roots(exe_dir, extra)`: `<exe>/models` first, then the operator's extra
folders (`OcrModelPrefs`) in order, duplicates dropped. Earlier roots win a
name clash; the engine reports the shadowed one as a note. A missing bundled
folder is not reported (a build may ship no models); a missing extra folder is.

# Runnability (`Unrunnable`, trace token in brackets)

| Case | Token |
|---|---|
| Manifest says `kind = program` — no host for an add-on program | `program` |
| Engine `paddle-vl` — the engine's runner cannot run one yet (G102) | `no-vl-runner` |
| Engine token this shell does not know | `unknown-engine` |
| Engine known but not linked into this build | `not-in-build` |
| Folder lacks a file `EngineId::model_files` names | `missing-files` |

`OcrModel::verify` (checksums) is never called here: a VL add-on is over a
gigabyte and the dialog opens on every click of the command.

# Starting choice (`start`)

1. A remembered `ocr_model` that is found and runnable is chosen.
2. A remembered model that is missing or unrunnable chooses **nothing**; the
   dialog names it (`Start::Remembered`) and Run stays disabled until the
   operator picks one. Silently picking another would run a different model
   than the one he set.
3. With no remembered model: the first runnable model of the remembered
   `ocr_engine`, else of the first engine in `EngineId::ALL` order.

# The dialog (`pdfcer-gui/dialogs/ocr_model`)

A ComboBox (`ocr-model`, items `ocr-model.item.N` in catalog order). Runnable
entries are selectable; unrunnable ones are disabled buttons whose hover gives
the folder and the reason. Traces: one `ocr-model name= engine= runnable=
why= folder=` per entry and `ocr-model-start chosen= remembered= roots=` at
open; `ocr-model-chosen name=` on a change; `ocr-started … source=bundled|
extra-folder model=` at Run. After a run the model and its engine are stored
(`ocr-engine-remembered engine= model= saved=`).

Driven by `an_extra_ocr_folder_adds_its_models_to_the_dropdown`.
