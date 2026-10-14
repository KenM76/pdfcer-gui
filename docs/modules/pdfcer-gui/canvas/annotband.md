# `canvas::annotband`

A select marquee over markups, links and ce dimensions. It runs on the same
release as the content marquee (`canvas::marquee::on_release`) and the form
field band, after the content one.

## Content wins a mixed band

`SelectionState` holds content entries or annotations, never both: every
single-target surface (grips, Properties, Delete) reads one or the other. A
band that catches both therefore keeps the content, which is what a band did
before annotations could be banded, and the status line names how many
annotations were left out and how to band them alone (untick the content
kinds under Select). Form fields are separate again: they live in
`OpenDoc::field_set` and combine with content.

## Combining

No modifier replaces, Shift adds, Ctrl subtracts — `marquee::merge`, the
mapping the field band shares. A replace band that caught nothing clears the
annotation selection, as a click on empty paper does; an add or subtract band
that caught nothing changes nothing.

## What is caught

`selection::annot::in_band`: window bands take outlines they contain,
crossing bands outlines they touch, on the `/Rect` outline in canvas space.
The ink-shape refinement a click uses for ce dimensions is not applied: a
band is about extent, not about which pixel was pressed. The candidates are
filtered by `selection::annot::pickable`, the same rule a click uses.

## Trace

`annot-band page caught merge crossing kept=annots|content selected`.
