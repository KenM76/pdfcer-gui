# `ui-verify/checks/resize`

`resize_scales_a_shape` — **the eight grips commit**, driven end to end
against the operator's own drawing.

# What this is for

The resize grips were drawn at S4 and have been **cursored, hit-tested and
drag-consuming ever since, committing nothing** — the oldest unbuilt thing
in this project and `FEATURES.md`'s last Phase 1 ⛔. An operator aiming at
one got a resize cursor, a drag that felt like it was working, and no
change: `DEFECTS.md` D4a's shape exactly.


# ★ Why this cannot be a unit test

Six links, and four of them are only observable in a running window:

| # | link | its own test |
|---|---|---|
| 1 | a click selects an object and the outline draws grips | `canvas::handles` — the layout, not the hit |
| 2 | a press on a grip is routed to `DragKind::Resize` rather than to a marquee | `gesture::meaning` — the decision, not the routing |
| 3 | the drag's screen delta becomes scale factors about the right anchor | `canvas::resizing` — yes, and it is pure |
| 4 | the anchor converts screen → canvas → PDF against **this** frame's mapping | **nothing** |
| 5 | every node's new position reaches `move_nodes` as one command | **nothing** |
| 6 | the engine rewrites the content stream and the page redraws | `pdfcer-core` |

Link 4 is the one that would fail silently and plausibly: a resize computed
against the wrong anchor still resizes, just about a different corner, so
the object moves *and* changes size. That looks like a slightly clumsy
gesture rather than a defect.

# ★★ The oracle is `resize-commit`, and it carries the numbers

`resize-commit grip=… sx=… sy=… ax=… ay=…`. A line saying only *"a resize
committed"* would be identical for a build that scaled about the centre,
mirrored an axis, or applied one factor to both — which is `DEFECTS.md`
D14's lesson (the freehand trail that authored two points) applied before it
bites: **a trace line must carry the number a wrong build would get wrong.**

And the edit itself is asserted separately, through `move-nodes`, because a
commit that computed the right geometry and never reached the engine is the
defect this whole feature is a fix for.
