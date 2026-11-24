# `canvas::measure::lines` — the tools whose picks are lines

Two-line and Centre line both take two clicks, each resolved to a line on the
page by `pick_line_in_page` against the page's decomposition. They share
`MeasureState::two_lines`, which a tool switch clears, so one tool never
inherits the other's half-picked pair.

The operator's parallel threshold (`doc.settings.parallel_epsilon_degrees`)
is passed to both; a default policy here would make the Settings value do
nothing.

| Tool | After the second line |
|---|---|
| Two-line | `TwoLineAuthoring` becomes the `Placing` preview; the next click places the label. |
| Centre line | `author_centre_line` is committed at once through `DimensionAction::Commit`: a centre line has no label, so there is nothing to place. |

A refusal goes to the status line by name (`record_note`, because nothing is
committed for a disclosure to ride on) and clears the pair, so the next click
is a first pick.

Trace lines: `centre-line relation=parallel|angled forced_parallel=0|1
overlap=<pt>`, `centre-line-refused reason=…`, `two-line-refused reason=…`.
Driven by `a_centre_line_is_drawn_between_two_lines`.
