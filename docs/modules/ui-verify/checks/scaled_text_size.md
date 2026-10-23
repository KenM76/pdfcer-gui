# `ui-verify/checks/scaled_text_size`

`scaled_text_shows_and_takes_its_drawn_size` — O309 and the shell's half of
O310, driven.

CAD exporters write every label `1 Tf` and carry its size in the text matrix.
The check writes such a page itself (one Helvetica line, `1 Tf` under an
18-point `Tm`), clicks the line as an object in Edit mode, and requires:

1. the Properties panel's read (`text-style-synced … size=`) is 18, not 1;
2. typing 24 into the size field and pressing Enter applies a restyle, and
   the panel's re-read of the run is 24.

A read of 432 at step 2 is the typed size written as the `Tf` operand under
the 18× matrix — the text redrawn off the page, which is what the operator saw
as "the text chunk goes blank".

Driven off the desktop through the scripted pointer, so it runs under
`--no-input` while the operator uses the machine.

## Falsification

| control build | expected verdict |
|---|---|
| `RunStyle.size` = `tf_size` instead of `GlyphProvenance::drawn_size` | FAIL at step 1, reads `1.00` |
| `StyleChange::Size` stamps `FormatRequest::size` instead of `drawn_size` | FAIL at step 2, reads `432.00` |

Both controls were watched red when the size arithmetic was the shell's own;
they are to be re-watched on the engine's.
