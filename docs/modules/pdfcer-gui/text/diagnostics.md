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

## Item notes

### `fn every_measurement_carries_its_unit`

Not a spelling test — a **units** test. A duration with no unit and a
scale with no multiplication sign are the two ways this surface could
present a number the operator cannot interpret, and both are exactly the
sort of thing a later edit trims for width.

### `fn the_scale_is_not_rounded_to_a_whole_number`

A rounded-to-integer scale would print `1×` for every zoom between 50 %
and 150 % on a 1.0 density display, which is a readout that changes
nothing while the thing it reports changes constantly.

### `fn the_clean_sentence_is_the_one_the_status_bar_uses`

The property this module's header is about, asserted rather than
promised: two surfaces describing one raster must not be able to say
different things about it.

### `fn the_absorbed_line_is_never_written_with_a_slash_or_a_parenthesised_s`

The shape being refused is *"0 structural oddity/oddities … and 0
section(s)"*. Every `diagnostics_*` entry in [`crate::text::status`]
spells both forms, so this is the catalog's own convention being kept
rather than a new rule — and the assertion is on the *characters*,
because that is what an operator sees and what a later "just make it
shorter" edit would reintroduce.

### `fn nothing_absorbed_is_stated_positively`

[`crate::text::status::diagnostics_clean`]'s argument, applied to the
line beneath it: a true answer that reads as an unfilled template is
worse than no line at all, because the operator cannot tell which it is.

### `fn every_blend_space_origin_says_something_different`

A `match` over a unit-variant enum is the shape that reads as obviously
correct and is the shape a copy-paste edit silently collapses: two arms
returning the same string still compiles, still passes a test that only
checks "the answer is non-empty", and hands the operator a sentence
about the wrong document. Pairwise inequality is the only assertion that
can fail for that.

### `fn only_the_output_intent_origin_names_the_setting`

R9's rule applied to prose. `page_blend_space_source` decides what to do
when the page group declares nothing AND the document carries a
resolvable output intent; on a page whose own `/Group` named a space it
is not consulted at all. A sentence sending the operator to Settings for
a page Settings cannot affect is the prose form of a disabled button,
and it costs more than a disabled button because he goes and looks.

Asserted BOTH ways. The positive half alone would pass a catalog that
named the setting in all three; the negative half alone would pass one
that named it in none.

### `fn the_blend_line_says_what_was_blended_and_not_just_a_colour_model`

The failure this pins is the one-word readout - `CMYK` / `RGB` - which
looks tidy in a report and is unreadable next to a duration, because
nothing on the line says what the acronym is a property OF.
