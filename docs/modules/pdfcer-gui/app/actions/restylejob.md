# `pdfcer-gui/app/actions/restylejob`

A restyle of many show operators, spread over frames so the window keeps
painting, and committed as one gesture.

## Why it exists

`format_text` plans against the page's whole content stream, so each call
costs about the same however small the operator: about 100 ms on
`SW41177.pdf`. An object click there selects one text object of a few hundred
runs, and a size typed for it is one call per show operator. Run in one frame,
that froze the window for tens of seconds (O310).

`EditSession` is not `Clone` and is owned by the UI thread's document, so the
calls cannot move to a worker without the engine's help. Slicing is the fix
the shell can make alone; the real fix is one engine call for the whole
gesture, requested as `ENGINE_BACKLOG.md` G227. With it, this module becomes
one call through the funnel.

## Contract

- **Slices.** Each frame runs calls until [`SLICE`] (50 ms) of engine time has
  passed, at least one call per slice. A slice takes the session through
  `funnel::take_quietly`, which neither bumps `edit_epoch` nor invalidates
  any cache: the picture on screen stays exactly as it was.
- **One commit.** The last call goes through `apply::vector_edit`, which bumps
  the epoch once, re-renders once, and records the gesture's disclosures.
- **One undo entry.** Every call after the first folds the stack with
  `EditSession::coalesce_last(2, CommandKind::FormatText)`. A fold the engine
  declines is traced `text-style-fold-failed`.
- **Pins read once.** The requests arrive ordered by
  `pin::operators_of_runs` (per stream, span start descending); see
  `textstyle.md` on why descending order keeps every later pin valid.
- **No edit sees half a restyle.** `funnel::vector_edit_scoped` and
  `apply::apply` call [`finish`] first, which runs the job to its end in that
  frame (`hurried=<why>` on the batch line). So a save, an undo, or any other
  edit always sees the gesture whole.
- **One job, the active document.** The job lives in a thread-local keyed by
  `OpenDoc::serial`; only the UI thread reads it. [`progress`] feeds the
  status bar's *Restyling text: N of M pieces done.*
- **The frame prelude stands aside.** While a job runs, `app::frame` skips the
  page content generation and the split-operand refresh: the page changes
  between slices with no epoch bump, the closing bump re-measures both, and on
  a dense drawing they cost ~85 ms of every frame between slices.

## Traces

| line | when |
|---|---|
| `text-style-slice page= done= of= engine_ms=` | a slice yields with calls left |
| `text-style-batch page= ops= applied= slices= engine_ms= wall_ms= hurried=` | the job ends; `hurried=-` when it ran at its own pace |
| `text-style-fold-failed` | the engine declined to fold an undo entry |

## Known limit

Every slice cancels the render worker before it takes the session, so a
render asked for mid-job (a page scrolled into view, a zoom) is cancelled by
the next slice unless it finishes inside one frame; on a dense drawing it
draws when the job ends. One that does finish draws the session as it stands,
partly restyled. Either way the commit's epoch bump re-renders it, and every
cache read mid-job is keyed on the old epoch, so none outlives the job.
