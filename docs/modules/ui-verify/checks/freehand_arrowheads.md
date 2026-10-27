# `a_freehand_line_can_end_in_an_arrowhead`

**Detects:** a Markup ▸ Style with no arrowhead chooser, a chooser whose
choice does not reach the pen, a freehand stroke with an arrowhead committed
as `/Ink` (which has no `/LE`, ISO 32000-2 Table 182), or a saved polyline
missing the arrowhead.

## Drive

1. Open a copy of `fixtures/square-no-appearance.pdf` in Review, with
   `PDFCER_DIAG_SAVE_PATH` set by `run_saving`.
2. Reveal `markup.style.ends`, press it, and require the open list to publish
   `markup.style.ends.end`; press that entry. The last `markup-pen` line must
   read `ends=None,OpenArrow`.
3. Arm Freehand and draw a 40-move half-sine arc at page height 640.
4. Save a copy.
5. Press Escape, click the arc's apex (page 300, 680), require `annot-select
   subtype=PolyLine`, and reveal Format.

## Oracle

The stroke's `markup-commit … freehand=1` line names `kind=PolyLine`, and the
saved file lists a `/PolyLine` whose whitespace-stripped body contains
`/LE[/None/OpenArrow]`. The trace separates "the chooser did not move the pen"
from "the pen did not route the stroke"; the saved bytes judge what another
viewer will draw.

Format's `ribbon.item.format.arrowheads` must then be at least 10 × 10
pixels. An item the band does not offer is still declared, at zero size, so
the declaration alone passes on a build that reads no endings from a
polyline.
