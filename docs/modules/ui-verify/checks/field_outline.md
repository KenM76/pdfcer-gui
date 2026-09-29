# `ui-verify/checks/field_outline`

`field_outline` — **a text field placed with the dialog's defaults draws a
visible outline on the page.**

## What it guards

With no `/MK /BC`, the engine's text-field appearance draws no box. So an
empty field is invisible on the canvas, on paper, and in a reader that does
not highlight fields (O262). `Draft::fresh` therefore gives the Text and
Choice kinds a black border colour. The engine strokes the frame whenever that
colour is set and the `/BS` width is positive.

## How it drives

1. The check writes a blank 300 × 300 pt page with no form, so it needs no
   `--pdf`.
2. It rings `mode.edit,edit.form_text_field`, and `PDFCER_DIAG_FORM_ACCEPT=1`
   accepts the placement dialog with its defaults.
3. It clicks the page and waits for a clean `add-form-field`.
4. It presses Escape, then clicks blank paper, so no selection chrome is drawn
   over the field.
5. It reads the field's rect from the `form-target` census and captures the
   window.
6. It counts ink pixels (every channel ≤ 110) in a ±3 px band round the
   field's edge, taken as the outer box minus the inner box.

It passes when that count is at least half the one-pixel perimeter. The shell's
field shading is a light fill, so it does not count as ink.

## Failure versus skip

- **SKIP:** no field was authored, no census line appeared, or the field is too
  small on screen. Placement is `form_field`'s subject.
- **FAIL:** a field exists and its edge carries no outline.

## Falsified

With the default removed from `Draft::fresh`, a rebuilt binary measures 0 ink
pixels against a 498-pixel perimeter and fails. With the default in place it
measures 467 and passes.
