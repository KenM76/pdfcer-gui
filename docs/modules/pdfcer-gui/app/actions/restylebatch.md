# `pdfcer-gui/app/actions/restylebatch`

One restyle gesture (a size typed in Properties, a font, bold, italic, colour,
a decoration) over any number of runs, as **one** `EditSession::format_texts`
call through the funnel.

## Contract

- **Input.** The `FormatRequest`s `textstyle::restyle` built, one per show
  operator, in `pin::operators_of_runs` order (per stream, span start
  descending). A `Gesture` carries the page, the style policy sampled once,
  the control's label, the run count and whether pinning stopped early.
- **One undo entry, one re-render.** `vector_edit` takes the session once,
  the engine plans the page once on its fast path (`FormatBatch::planned_once`),
  and the funnel bumps the epoch once.
- **Nothing half-applied by one call.** On `FormatBatchError::Request` the
  engine has applied nothing; the refusal of that request is worded on the
  bar through `textstyle::refusal_of`, as a single-run restyle would word it.
- **Disclosure.** Each report's notes (`textstyle::report_notes`: the ladder
  sentence, the render-mode note, the engine's own disclosures) and the
  batch's `FormatBatch::disclosures`, each sentence once, go to the status
  bar through `textstyle::emit_carried`. Nothing is drawn on the canvas.

## The one split

The engine applies a batch in one walk only when every request is pinned, on
the same page's own `/Contents`, with disjoint spans and no created resource
(font, opacity, decoration). Anything else it applies one request at a time
and folds into one undo entry, and that path refuses more requests than the
undo depth (`TooManyForOneUndo { limit }`, 256), because it holds one entry
per request until the fold. **Only then** is the gesture split into calls of
`limit` requests; each call after the first is folded into the previous with
`coalesce_last(2, CommandKind::FormatText)`, so the gesture is still one undo
entry. A call that refuses stops the gesture; what earlier calls applied
stays and the bar says only part was restyled.

## Traces

| line | when |
|---|---|
| `text-style-batch page= ops= applied= calls= planned_once= wall_ms=` | every gesture; `calls` above 1 means it was split |
| `text-style-chunked ops= limit=` | the engine refused the batch as one undo entry |
| `text-style-fold-failed` | a split call could not be folded into the previous one |
| `text-style-applied page= change= applied= runs=` / `text-style-declined …` | the verdict |

`applied` counts show operators, `runs` counts runs.

Driven by `ui-verify` `a_large_restyle_keeps_painting`.
