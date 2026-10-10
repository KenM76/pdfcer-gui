# `app::actions::vector::textmerge` — merge text runs on the page or inside a placed drawing

Commits `VectorAction::MergeTextRuns { page, target, runs }` through
`vector_edit_on_page`: one undo entry, a page-scoped re-render.

## Routing

`target` is a `TargetId`. `Object(i)` calls
`EditSession::merge_text_runs(page, i, runs, &MergeOptions::default())`;
`Leaf(i)` calls `merge_text_runs_in_form(page, i, runs, ..)` on the form the
selection has entered. Both return a `MergeReport`; the in-form verb wraps it
in a `FormTextOutcome` whose `invocations` and `pages` say how often the form
is drawn.

## Disclosure

The status line carries the report's disclosures. When the engine disclosed
nothing but the merged run's width changed, `text::runmerge::width_changed`
supplies the sentence. When an in-form merge reaches a form drawn more than
once (`invocations > 1` or `pages > 1`), `text::unshare::remedy_if_shared`
appends the shared-content remedy: the reach is returned as counts, not as a
disclosure, so the shell words it.

## Refusals

A `FormatError` becomes a `RunMergeRefusal` (`text::runmerge`), recorded as a
`canvas-decline-recorded` line and worded on the status line.
`FormatError::PageIndex` and `FormLeafOutOfRange` read as a stale selection.

## Trace

`merge-text-runs-applied page= object= in_form= merged= scale=` after the
engine returned `Ok`; `object` is the page-object or leaf index, `in_form`
says which.
