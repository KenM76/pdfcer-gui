# `app::actions::comparemark`

Writes a comparison's changes as comments on the open (newer) document. It
calls `pdfcer_core::compare::mark_comparison`: the appearance areas, then the
text changes and the summary page, in the order the engine's own
`compare --output` uses. With no text comparison it calls `mark_regions`
alone. Both go through `funnel::vector_edit`, so the call is refused on an
encrypted document with the unlock offer (`Refusal for ResultError`). The
funnel re-renders the touched pages and reports on the status line.

## One gesture, one Undo

The engine records the whole marking as one undo entry of kind
`CommandKind::MarkComparison`, folding it as each command lands. That includes
a stack at `MAX_UNDO_DEPTH`. The shell folds nothing.

## Partial failure

The engine's error says where it stopped. `ResultError::Summary` comes after
every mark, and the engine keeps them as one entry, so the shell refreshes the
pages and the status line says marking stopped and that one Undo removes the
marks. `ResultError::Pages` comes before anything is authored and is the
funnel's ordinary refusal, as is an encrypted document.

## Trace

`compare-marked annotations= unmarked= top=`. `top` is `mark-comparison`
when the undo stack's top entry is the marking, otherwise `other` or `none`.
A partial failure traces `compare-marked partial=1 top=`.
