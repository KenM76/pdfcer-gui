# `app::actions::comparemark`

Writes a comparison's changes as comments on the open (newer) document:
`pdfcer_core::compare::mark_regions` for the appearance areas, then
`mark_changes` for the text, as the engine's own `compare --output` runs
them. Through `funnel::vector_edit`, so it is refused on an encrypted
document with the unlock offer (`Refusal for ResultError`), re-renders the
touched pages, and reports on the status line.

## One gesture, one Undo

Each mark is its own engine command. The gesture is one, so the commands it
pushed are folded with `EditSession::coalesce_last`. The count is the undo
depth's difference, not the reports' sums, because the summary page and a
mark spanning lines push commands the reports do not count one for one. A
stack at `MAX_UNDO_DEPTH` may have dropped its oldest entries, which makes
the difference a lie, so a full stack is never folded and the status line
says the marks take several Undos. The engine pushing one entry itself is
requested in the channel (`ENGINE_BACKLOG.md`).

## Partial failure

When the area marks were written and the text marks then fail, the area
marks are kept (they are correct) and the status line says why the text
ones are missing. A failure before anything was written is the funnel's
ordinary refusal.

## Trace

`compare-marked annotations= unmarked= folded={0|1}`.
