# `pdfcer-gui/text/maxzoom`

## Item notes

### `fn the_top_preset_reads_as_what_is_actually_stored`

`1e12` rounds to `999,999,995,904` — four parts in a billion
low, unobservable at a zoom where one screen pixel is a millionth of a
point. But the label is read by a person, and a row claiming a round
trillion while the preferences file says otherwise is the kind of small
inconsistency that makes somebody doubt the whole control.

So the preset list uses `MAX_MAX_ZOOM_PERCENT` and this asserts the
honest rendering. If a future edit makes the two agree by rounding the
LABEL instead, this fails — which is the right way round, because the
file is the thing the operator can check.

### `fn crossover_note`

It names the **consequence**, not the mechanism. *"pdfcer draws the whole
page below this and only the visible part above it"* is an implementation
detail; *"panning stays instant below, and redraws above"* is what he will
actually notice, and it is the same fact.

### `fn preset`

Spelled in the units he used — *"1,000,000,000,000%"* — rather than in
exponent notation. A person reading a menu should not have to decode
`1e12`, and the grouping separators are what make the difference between
a million and a billion legible at a glance.

### `fn current_suffix`

A suffix rather than a tick, because the rows are a `selectable_label` set
and the selection is already drawn — this says *why* one is selected for
somebody who arrives at the popup without having set it.
