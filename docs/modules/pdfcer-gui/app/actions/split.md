# `app::actions::split` — one document written as several

The body of `PageAction::SplitDocument`. The arm is in `app::actions::pages`.

## Contract

- `plan(view, request, stem, source)` is the preview and the write's first
  step. Its refusals, in order: a pattern holding `\ / : * ? " < > |` or not
  ending `.pdf`; no folder; a folder that does not exist; the engine's
  `plan_split` errors (`NoSplitPoints`, `AmbiguousNames`, else its Display);
  a part whose target is the open document's own file. Otherwise the parts
  and how many targets already exist.
- `split(doc, request, separations)` re-plans against the document as it is
  now, assembles with `pageops::split_with_labels` and writes each part with
  `std::fs::write`, replacing a file of the same name. A failure stops the
  run; files written before it stay, and the receipt says how many.
- Reads `doc.session.view()`, so unsaved edits are in the files. No epoch,
  no undo step: the open document is untouched.
- `{stem}` is the stored file's stem, else the path's, else `document`.

## Why the source check is here

The engine names files and never sees a folder, so only the shell knows
that `{stem}.pdf` in the document's own folder is the document. The check is
by canonical path, case-insensitive where either path does not exist yet.

## Receipt and trace

Status line: `wrote`, plus the labels sentence when the document has labels;
or `failed(written, detail)`.

- `split-part n= of= first= last= pages= name= bytes= labels_dropped= label_ranges=`
  per file, 0-based pages, from the engine's `AssembleReport`.
- `split-written files= rule=<every-N|after-K|bookmarks> labels=<keep|drop> folder=`.
- `split-failed written= detail=`.
