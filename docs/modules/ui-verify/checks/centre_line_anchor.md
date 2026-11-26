# `centre_line_anchor` — a ce dimension follows the centre line it was placed on

`a_dimension_follows_the_centre_line_it_was_placed_on` draws a centre line on
`two-lines.pdf` as `a_centre_line_is_drawn_between_two_lines` does, then arms
Measure ▸ Linear and picks the centre line's midpoint (200, 140), the lower
line's left end (100, 100), and a placement point. A pick that only promotes a
derived snap is clicked once more.

1. The commit must trace `dimension-added … anchors=0:0:line-…`: point 0 is
   bound to dimension 0, the centre line. A build that does not carry the
   snap's anchor into the commit traces no `anchors=` token.
2. The centre line is selected and its end grip 1 dragged 40 window points up.
   The engine's `move-dimension-vertex` must carry the disclosure *"moved with
   the geometry it is anchored to"*, which the funnel takes from
   `last_anchor_resolutions` — the dependent was re-solved and the operator
   told so.

The Properties panel's *Detach* button is not driven here.

The scripted pointer drives it, so it runs under `--no-input`. Falsified on
the released `v0.5.0-dev.20261125.3`: its commit carries no anchor.
