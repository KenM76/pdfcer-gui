# `pdfcer-gui/canvas/measure/circpick`

## Item notes

### `fn a_degenerate_removal_radius_never_removes`

The degenerate-mapping case, and the direction of the failure is the
whole point: a tool that stopped accepting points would look broken, and
a tool that accepted them and removed something else would be worse —
the operator would watch their set shrink as they clicked.

### `fn a_picks_origin_survives_into_the_set`

The disclosure `OPERATOR_REQUESTS.md` O106 rests on: five free positions
and five snapped nodes produce the same numbers, and only one of the two
is the drawing's own geometry. The canvas does not distinguish them —
rule 4 forbids marking applied content — so this value is the only thing
the Tool panel has to tell the operator with.
