# `ui-verify/checks/text_in_own_font`

O247. With the caret in real page text, the draft must look like the page's
own text: the same font, size, colour and position.

## The oracle

Three captures, all measured over the `text-edit.box` region inset by 3 px.
The inset excludes the accent outline and a caret at either end of the line.

1. **before**: Edit text is armed, with no draft yet. This is the page's own
   render of the run.
2. **typed**: the caret is moved to the start and `X` typed, so every glyph
   after it shifts. The trace must
   carry `text-edit-shaped shaped=1`; otherwise the shell-font box was drawn,
   and that is the failure.
3. **original**: Backspace puts the original text back. This capture must
   match *before*: no more than 3 % of pixels may lack a counterpart within one
   pixel in the other capture. A counterpart differs by less than 64 in every
   channel. The page and the draft come from two rasterisers, and their glyph
   edges disagree by up to a pixel even when the glyphs match. Measured on
   `layered-drawing.pdf`: 1.7 % with the original text, 19.4 % with an `X` at
   the start. Without the one-pixel allowance, the matching draft scores 4.3 %.

Only an in-font draft can match *before*. The shell-font box draws a
different typeface on a theme fill, and it fails step 3 even when the trace
says `shaped=1`.

**Calibration.** *typed* must differ from *before* by more than the 3 %
threshold. If it does not, the comparison cannot see a one-character change
at this size. The check then SKIPs rather than passing on a blind measurement.

## Inputs

It needs a `--doc-point` on visible page text. The sweep uses the same point
on `layered-drawing.pdf` as `text_edit_on_a_real_drawing`. A point on empty
page SKIPs, because the page under the box has no ink to match.
