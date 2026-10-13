# `dialogs::bates_batch` — Bates-number several files in one run

`tools.bates_files`, Tools ▸ Batch ▸ Bates number files…. Opened by
`app::dispatch::batch` after the multi-file picker (`app::files::pick_bates_sources`,
seam `PDFCER_DIAG_BATES_SOURCES`) returns at least one file. Application-scoped:
it is drawn above the no-document guard in `DialogsState::show`, works with
nothing open, and closing a document leaves it open.

Three files: `bates_batch.rs` (the window), `naming.rs` (where each copy goes, pure),
`run.rs` (the two workers).

## Contract

| Item | Contract |
|---|---|
| Order | The list order, top to bottom, is the numbering order. Rows move with Up and Down and leave with Remove; Add files… appends, skipping a path already listed. There is no automatic sort, as in Acrobat. |
| Numbering | The `BatesForm` Pages ▸ Stamp ▸ Bates numbering… uses, drawn by `BatesForm::controls`. Each file numbers every page; file *k* starts at the number after file *k−1*'s last page (`BatesForm::batch`, which returns one `Span` per file). A run whose last label outgrows the digit count is refused as a whole, before anything is written. |
| Continuation | The form is seeded from `DialogsState::bates_form` and, on close, written back by `remembered`, advanced to the number after the last label of a completed run. The single-document window and the batch therefore continue one run between them. |
| Preflight | Each listed file is read on a worker (`run::probe`): load, signature census, page count, and a trial `EditSession::stamp_bates` over a fresh session that is then dropped. A password-protected file, a signed file, an unreadable file and any engine refusal (encryption, a certification forbidding page changes, hidden-object exposure) are named in the file's row. Number files is absent while any row is unread or refused (R9). |
| Signed files | Refused, not numbered. Numbering appends changes after the signature; in a batch that would happen to a file the operator may not have looked at. One signed document is numbered on its own through Pages ▸ Stamp, where the save-time signature question asks. |
| Copies | Never an original. Folder: beside each original, or one chosen folder (seam `PDFCER_DIAG_BATES_FOLDER`). Names: the original name; the original stem with text before and after (default `_Bates` after); the file's label range `first-last`; or the range then the original stem — the last is the combination Acrobat lacks. `naming::Output::plan` refuses a copy that would land on any listed original, two copies on one name, a range or affix holding a character Windows forbids in a name, and one-folder with no folder chosen. |
| Existing files | A run that would write over an existing file stops before writing anything unless *Replace numbered copies already there* is ticked. An original is never replaced either way, because `plan` refuses that case before the run. |
| Writing | `run::run`, on a worker: per file, `Document::load`, `SettingsExt::open_session`, `stamp_bates(stamp, first)`, `to_incremental_bytes(SettingsExt::save_options())`, written to `<target>.pdfcer-part`. Only when every file is numbered are the parts renamed into place; a failure on any file deletes every part, so no copy is written whose numbers assumed a file that failed. A rename failing part way is reported with the count placed. Each copy is its original's bytes plus an appended update, as Save a copy writes. |
| Disclosure | Inside the window: the summary line (files, first and last label, next number) before the run, progress during it, and the outcome sentence after. The window stays open while a run is in progress. |

## Trace and regions

- `bates-batch-picked source n`, `bates-batch-folder-picked source path`
- `bates-batch-opened n start`
- `bates-batch-probed path pages refused`
- `bates-batch-run n first replace`
- `bates-batch-wrote index source target pages first_label last_label next` — the engine's `BatesOutcome`, not the window's arithmetic
- `bates-batch-done files pages first_label last_label next`, or
  `bates-batch-refused reason=exists|failed|partly`

Regions: `bates_batch.body`, `bates_batch.run`,
`bates_batch.row.<i>.{up,down,remove}`, and the numbering controls under
`bates_batch` (`bates_batch.prefix`, `bates_batch.start`,
`bates_batch.position.<i>`).
