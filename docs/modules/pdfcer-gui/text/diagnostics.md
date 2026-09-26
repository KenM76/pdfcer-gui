# `text::diagnostics` — every word the Render-diagnostics dialog shows

The copy for `tools.render_diagnostics`, on **Tools ▸ Diagnostics**, drawn
by [`crate::dialogs::diagnostics`].

## What is deliberately NOT here: the findings themselves

The sentences that name what the renderer substituted or skipped —
*"3 glyphs drawn with a bundled substitute face"*, *"1 content stream
missing from the file"* — live in [`crate::text::status`] and are used from
there unchanged. They were written for the status bar's disclosure and they
are the same facts said to the same operator; a second wording here would be
`DEFECTS.md` D5's shape in a catalog rather than in a list, and the two
copies would drift the first time one of them was improved.

So this file holds only what the **dialog** adds and the bar has nowhere to
put: a title, the three measurements of the render itself, the headings that
separate them from the findings, and the sentence shown when there is no
raster to describe at all.

## Why the measurements are worded as a pair

One measured fact makes a bare duration misleading on this project's own
documents: on dense CAD roughly 99 % of render cost is
resolution-independent. A small thumbnail is not a cheap thumbnail — a 1×1
*point* region of such a sheet still costs about 691 ms.

An operator reading "1,240 ms" alone will reach for the zoom, and on a CAD
sheet that will not help. So the scale and the pixel size are shown beside
the duration rather than under a separate heading, and the tooltip on the
group says what the relationship actually is. This is the same editorial
rule [`crate::text::status::diagnostics_layers_hidden`] follows one surface
over: **name the cause, or the number reads as a fault.**
