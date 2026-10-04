# `ui-verify/checks/replace_text`

`replacing_text_strikes_and_carets` — Markup ▸ Replace text over a text
selection puts a `/StrikeOut` over the selection and a `/Caret` at its end
carrying the typed words, as one comment.

# What it drives

Its own fixture, `fixtures/word-fragmented-lines.pdf` (ignores `--pdf`; a
US Letter page whose first line, `Date Premises Required____`, sits at y=700
from x=72).

1. Review mode, `ribbon.tab.markup`, `ribbon.item.markup.replace_text` with
   nothing selected: no `replace-text-open` may follow. The command is
   enabled only on `selection.text`.
2. A scripted drag along the first line, (74, 704) to (160, 704), then the
   item again: `replace-text-open page=0 quads=N`.
3. `new words` typed into the window's text field, then `text-annot.accept`:
   `replace-text-read … chars=9 quads=N x=…` with x right of the sweep's
   start, then `replace-text-placed … caret=<non-zero> strike=<non-zero>`.

`chars` is the oracle that the typed words reached the action; `quads`
matching the open line is the oracle that the selection's boxes survived
the window; the two ids are the engine's `ReplaceTextAdded` receipt.

# Driven off-screen

Scripted pointer, window at `-4200,-4200`, `spec.place = false`; it runs under
`--no-input`.
