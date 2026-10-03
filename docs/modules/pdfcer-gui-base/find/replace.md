# `find::replace` — the Replace row's state

The replacement text and whether the row is open live on `FindState`, next to
the query, so they survive closing and reopening the bar exactly as the query
does.

`replace_ask(epoch, all)` is the only way a press reads the hits. It answers
`None` unless the readout is `At`, so a stale search (the document changed
since) or an empty one replaces nothing; the operator searches again first.
Each hit is handed over as its page and its `Quad` in unrotated PDF user space,
the frame the engine matched in, not the canvas box drawn on screen.

`land_on(doc, index)` puts the bar back on a position after the re-search a
replace runs, clamped to the last hit.

The row itself is `find::bar::replace`: drawn only when the caller offers it
(the mode can edit content), its buttons live only on a current hit, and Enter
in its field replaces the current hit.
