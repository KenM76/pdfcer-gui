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
