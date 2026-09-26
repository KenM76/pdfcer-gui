# `ui-verify/checks/shape_preview`

`dragging_a_node_bends_the_line` — the driven proof of
`OPERATOR_REQUESTS.md` **O63**'s first and third pieces.

# What this is about

**Ken, 2026-08-30:** *"if I moved the end of a line, it didn't show me the
shape change of the line, it just had a perimeter box around it. this goes
for anything I change right now. there isn't a real preview like there is in
inkscape."*

He was right, and it was a written convention rather than an oversight —
`canvas/handledrag.rs` said *"a preview shows the cursor, the render shows
the document."* That convention is now overruled by operator ruling.

# Why this needs TWO trace lines and not one

The shell publishes:

| line | question it answers |
|---|---|
| `canvas-shape-preview` | was a preview **built**? |
| `canvas-shape-drawn` | did it reach the **painter**? |

A preview that is built and never painted — a `None` lost on the way through
`interact`, a painter arm never reached, a page index that does not match —
**looks exactly like a preview that was never built** to anything reading one
line. This project has shipped that shape of defect before: a panel that
rendered off-screen with every gate green, and a control published at its
content position that no pointer could ever reach.

⇒ So both are asserted, in order, and the failure message names which of the
two stages the build got to. *"Nothing happened"* and *"something happened
and nobody saw it"* have different causes and different fixes.

# And the third piece: it has to OUTLIVE the release

`canvas-held-preview` is written when the gesture hands its geometry to the
document to keep on screen while the page raster catches up. Without it the
operator watches the object snap back to where it started and then jump
forward a second later, which reads as the program refusing the edit and
changing its mind.

The hold is asserted **after** the drag, from the same trace, because it is
the half that has no visible difference from "no preview at all" on a page
that rasterises quickly — and every fixture in this repository rasterises
quickly. A check that only drove the fast case would pass on a build with the
hold deleted.
