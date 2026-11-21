# `ui-verify/checks/form_preview_choices`

`form_preview_reading_choices_reach_the_form` — every Reading choice of the
Form preview window is offered, and an answer reaches the views, the merge,
the layout or the drawing it names.

# What it drives

A copy of `fixtures/xfa-choices.pdf` (built by
`fixtures/xfa-choices.PROVENANCE.py`), opened off-screen under `--no-input`
with a `ScriptedPointer`. The form opens Form preview by itself. The window
is launched through `form_preview::launch`, shared with the
`form_preview` checks.

1. The first `form-preview` line must carry `choices=default excluded=0`:
   the file names no views, so nothing is left out.
2. Every choice must declare its `form-preview.choice.<slug>.1` region, and
   `picture-rounding.2` too: a choice the window drops is caught by name.
3. Click *A border with no fill: Is filled white*
   (`form-preview.choice.missing-fill.1`). The redraw must carry
   `missing-fill:1` in its choices and a new `digest`: the `unfilled` field
   over the grey `shade` is painted white.
4. Click *The form is read for: Printing as well*
   (`form-preview.choice.view.1`). The redraw must carry `view:1`,
   `excluded=1` and a new digest, and a `form-preview-left-out` line must
   name `form1.body.screen_only`, the fixture's `relevant="-print"` draw.
5. Click *A section laid out in rows: Moves whole to the next page*
   (`form-preview.choice.tiled-split.1`). The redraw must carry
   `tiled-split:1`: the layout answer reached the worker.

# What would make it fail

A choice missing from `choices::CHOICES`, `Choices::paint` ignoring
`MissingFill`, `Choices::views` not adding `print`, `merge` discarding the
excluded paths, or `lay_out_naming` laying out with `LayoutOptions::new()`.
Planting `MissingFill::None` for both answers turns the check red at step 3.

# What it does not prove

Steps 3 to 5 sample one answer from the drawing, views and layout groups;
the rest are proved to reach their settings by
`dialogs::xfa_preview::choices` unit tests, not by pixels.
