# `panels::compare`

Acrobat's Compare Files. The open document is the newer version; Tools >
Compare (or *Choose…* in the panel) picks the older one. The panel holds the
options, the list of changes and the marking button.

## Contract

- **The worker never holds the session.** `CompareUi::start` hands the worker
  the newer document's path, or, when it has edits not on disk (or no file),
  its incremental-save bytes taken on the UI thread. The worker
  (`job`, thread `pdfcer-compare`) loads both and runs
  `pdfcer_core::compare::compare_documents` (text, optionally formatting)
  and `pdfcer_render::visual_compare` (appearance). The frame polls a
  channel; a dropped sender reads as a lost worker and is said.
- **A result belongs to one document at one edit epoch.** `Results` records
  `OpenDoc::serial` and `edit_epoch`. Another document shows "compared with
  another document"; an edit since disables marking (the engine's places
  would be stale), explained on hover.
- **Rows** (`rows`): one per text change (inserted, deleted, replaced,
  formatting), one per changed area of a page pair, one per page added or
  removed. Each carries its place in the newer document when it has one; a
  deletion's place is where it would have been. A row without a place (a
  page only the older file has) cannot jump and says so on hover.
- **A row click** traces `compare-jump`, pushes `Action::GoToPage` when the
  page differs, and `Action::RevealRect { why: "compare-row" }` for its
  rectangle. The canvas is never marked: the selection of a row is the list.
- **Marking** pushes `Action::NewComment(NewComment::ComparisonMarks)`;
  `app::actions::comparemark` writes it. The labels name both files by name.
- **Options persist** across documents; the result and the older path reset
  with `PanelsState::forget_document`.

## Disclosure

Everything inferred is listed here, off-canvas (R8b): the engine's notes on
the comparison (work limit hit, a page size that differs, a page that could
not be rendered) under the counts line; marking's unmarked count and
any fold the undo stack could not make on the status line.

## Placement

Tools tab, its own `compare` group (collapse rung 4), command
`tools.compare`, enabled when a document has pages. The panel is in
`Panel::ALL` and in every mode's arrangement; Edit lists it after Output
Preview.

## Trace

- `compare-picked source=env|native answer=…` — `app::files::pick_compare_source`;
  `PDFCER_DIAG_COMPARE_PATH` answers it for driven checks.
- `compare-started bytes={0|1} text={0|1} appearance={0|1}`.
- `compare-ran text_changes= regions= notes= ms=` — worker side.
- `compare-spawn-failed` — the thread could not be started.
- `compare-listed rows= kinds=kind@page,…` — page 1-based, `-` for no place;
  kinds `inserted deleted replaced formatting area page-added page-removed`.
- `compare-jump row= kind= page=` — page 1-based.

Regions: `panel:compare`, `compare.run`, `compare.mark`, `compare.row:<n>`
(0-based, declared only while visible).
