# `ui-verify/checks/markup_reshape`

Two checks on `fixtures/markup-reshape.pdf`, driven with
`PDFCER_DIAG_INVOKE=mode.edit,file.properties`:

- `a_highlights_end_drags_onto_the_next_line`
- `a_lines_fill_colours_its_closed_arrowheads`

The fixture is one 400 x 400 page of Courier 12 pt text, so every glyph is
7.2 pt wide. It carries two annotations:

- a highlight over "beta" on line 1;
- a `/Line` with closed arrowheads at both ends and no `/IC`.

Both appearances are hand-drawn.

# Highlight

1. A click on the highlight requires `annot-select`. It also requires the
   `canvas.markup-end.1` grip region to be published.
2. Drag that grip's centre in 10 steps to (125, 283). That point is the right
   half of the final "a" of "zeta" on line 2.
3. Require `markup-respan`, then `respan-text-markup-applied` with
   `quads=1->2`, one quad per covered line.

A screenshot is taken after the release.

# Line fill

1. A click on the line's stroke requires `annot-select`.
2. Scroll the `properties.markup.fill` row into view and require it.
3. Pick a colour in its picker.
4. Require exactly one `set-markup-style-applied` after the pick, and require
   that it shows:
   - `interior` other than `none`;
   - `endings=ClosedArrow/ClosedArrow`, read back from the file.

Requiring exactly one edit is what catches a swatch that commits on every
frame of a picker drag.

# Driven off-screen

The checks use the scripted pointer, with the window off every monitor and
`spec.place = false`. They run under `--no-input`.
