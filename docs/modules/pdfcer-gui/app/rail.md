# `app::rail` — drawing the left rail

`OPERATOR_REQUESTS.md` **O123** part 7 and **O126**'s addendum. The
permanent vertical strip down the left dock's outer edge: the panel tabs,
the navigate selectors, the selection controls, rotate.

Three files, three jobs, and the split is the R7 line:

| file | owns |
|---|---|
| `crate::shell::manifest::rail` | **what is in it** — command ids, groups, fold policy, as manifest data |
| `egui_shell::dock::rail` | **the geometry and the fold ladder** — a constant width, and which rows exist at which budget |
| this file | **what a row looks like**, and what a press does |

`tools/gates/check-shell-purity.sh` is what makes that split load-bearing
rather than tidy: the shell may not learn that `pages` is a page thumbnail
list, so the shell plans ids and this file paints them.

## Why every row publishes through `ui_rect_visible`

`crate::diag::ui_rect` says *"this region was laid out at these
coordinates"*. That is **not** the claim a rail needs to make. A panel entry
can be unreachable and still publish a perfectly healthy rectangle with
every gate green — which is how three of them shipped unreachable. A rect
channel that reports layout cannot tell a working rail from that state, so
this file reports **visibility** instead, and only ever that.

⇒ So there are **two** regions per press target, deliberately:
`dock.left.toolrail` from the shell says *the strip is on screen*, and
`rail.<group>.<command>` from here says *this control was drawn inside it
and enough of it survived the clip to click*. A build whose handler
returned early would keep the first and lose the second.

## The width is not this file's to choose

[`egui_shell::dock::rail::WIDTH_PTS`] is a constant and the strip's `Ui` is
already clipped to it. Nothing here measures a label. See that module's
header on R128 and the fit-zoom feedback loop; the short form is that a
rail sized from the word `Signatures` moves the canvas, which re-fits the
zoom, which is a loop this project has paid for twice.

## The strip scrolls rather than truncating

`RIBBON_SCALING.md`'s third rung. The ladder stops at
[`Rung::Cramped`](egui_shell::dock::rail::Rung::Cramped) and does not shed
further rows; below that the `ScrollArea` here carries them. A rail that
simply cut its last entry off the bottom edge of a short window would be
the unreachable-control defect arriving by a different route.
