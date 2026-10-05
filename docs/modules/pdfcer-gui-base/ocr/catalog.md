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

# Runnability (`Unrunnable`)

The engine decides: `pdfcer_ocr_host::OcrRunner::check_runnable(model,
policy)`. This module keeps the refusal's own sentence as the reason and maps
the `RunnerError` variant to a stable trace token (`token_of`):

| Engine refusal | Token |
|---|---|
| `EngineNotInBuild` (includes an engine token nobody knows) | `not-in-build` |
| `MissingFile` | `missing-files` |
| `NeedsProgramKind` | `needs-program-kind` |
| `Verify` | `verify` |
| `ProgramRefusal::RefusedByPolicy` | `refused-by-policy` |
| `ProgramRefusal::NoProgramHash` / `NoProtocol` / `ProgramMissing` / `NotAProgram` | `no-program-hash` / `no-protocol` / `program-missing` / `not-a-program` |
| any other refusal | `program-refused`, `program`, `engine`, `other` |

`policy(allow)` builds the `ProgramPolicy` from the Settings checkbox
(`ocr_program_addons`). Refused program add-ons are still listed, disabled,
so the operator sees what the setting costs.

`check_runnable` does not hash: `OcrModel::verify` on a VL add-on reads over
a gigabyte and the dialog opens on every click of the command. A program
add-on's hashes are checked by the runner before each page, and a mismatch
stops the run.

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
