# Inkscape 1.4 — Align and Distribute, the reference `O263` is built against

Read from the `INKSCAPE_1_4` tag of `gitlab.com/inkscape/inkscape` (not by
running Inkscape). Files: `share/ui/align-and-distribute.ui`,
`src/ui/dialog/align-and-distribute.cpp`, `src/actions/actions-object-align.cpp`,
`src/actions/actions-node-align.cpp`, `src/ui/dialog/tile.cpp`,
`src/ui/dialog/grid-arrange-tab.cpp`, `src/ui/dialog/polar-arrange-tab.cpp`,
`src/object/algorithms/unclump.cpp`, `src/seltrans.cpp`, `share/keys/inkscape.xml`.
"Source-observed" = what the code does, not seen running.

pdfcer's mapping of each item is in `ALIGN_AND_DISTRIBUTE.md`.

## Dialog

Object ▸ Align and Distribute, **Ctrl+Shift+A**. A notebook of three tabs:
**Align**, **Grid**, **Circular**. Under the notebook an **Arrange** button
("Arrange selected objects"), hidden on Align, runs the Grid/Circular tab.

## Align tab — object mode (any tool but Node)

### Frame "Align"

- Toggle "Enable on-canvas alignment handles" — label *Alignment handles with
  third click*.
- Toggle "Treat selection as group" — label *Move/align selection as group*.
- **Relative to:** Last selected · First selected · Biggest object · Smallest
  object · Page · Drawing · Selection Area. Default *Selection Area*. With one
  item selected the list shrinks to Page · Drawing, default Page, remembered
  apart from the multi-item choice.

| Row | Buttons, left to right (tooltip) |
|---|---|
| Horizontal | Align right edges of objects to the left edge of anchor · Align left edges · Center on vertical axis · Align right edges · Align left edges of objects to the right edge of anchor · Align text anchors horizontally |
| Vertical | Align bottom edges of objects to the top edge of anchor · Align top edges · Center on horizontal axis · Align bottom edges · Align top edges of objects to bottom edge of anchor · Align text anchors vertically |

**Align math.** Target point `mp = mx0·b.min + mx1·b.max` on the target box
`b`; item point `sp = sx0·min + sx1·max`; each item moves by `mp − sp`.

| Token | target (mx0,mx1) | item (sx0,sx1) |
|---|---|---|
| left | 1,0 | 1,0 |
| centre | ½,½ | ½,½ |
| right | 0,1 | 0,1 |
| left-to-anchor's-right | 0,1 | 1,0 |
| right-to-anchor's-left | 1,0 | 0,1 |

Target box: last/first selected item's box; biggest/smallest item's box — the
comparison is on the **perpendicular** dimension (a horizontal align compares
heights; source-observed); the page box; the drawing's box; the selection's
box. The anchor item never moves. **Group mode:** `sp` comes from the union of
all moving items, so every item gets one delta. Moves ≤ 1e-9 are skipped.
Undo label "Align".

**Text anchors.** Only text moves, by its baseline anchor. Reference: the
anchor item's baseline anchor if it is text, else its box's min corner; for
Page/Drawing/Selection the box's min corner. *Horizontally* sets each anchor's
X; *vertically* sets Y.

### Frame "Distribute"

| Row | Buttons |
|---|---|
| Horizontal | even left edges · even centres · even right edges · even gaps · text anchors horizontally |
| Vertical | even top edges · even centres · even bottom edges · even gaps · text anchors vertically |

Needs ≥ 2 items. Items sorted (stable) by the anchor. **Edge/centre:** first and
last stay; item i goes to `first + i·(last−first)/(n−1)`. **Gaps:** sorted by
centre; `step = (last.max − first.min − Σextent)/(n−1)` (may be negative); the
first item's min stays and `pos += extent + step`. **Text:** min and max anchors
stay, the rest spaced evenly. Undo label "Distribute".

### Frame "Rearrange"

Nicely arrange selected connector network · Exchange positions — selection
order · — stacking order · — rotate around center point · Randomize centers
in both dimensions · Unclump objects: try to equalize edge-to-edge distances.

- **Exchange** (all three): each item's centre goes to the previous item's
  centre in the list; the first takes the last's. Lists: selection order;
  z-order; angle `atan2` around the selection centre, then distance.
- **Randomize:** per axis, keep the centres' min/max: two distinct random items
  land exactly on min and max, the rest uniform in between.
- **Unclump:** edge-to-edge distance through the ellipse inscribed in each box
  (nearest-edge points when aspect > 1.5 or < 0.66); neighbours chosen greedily,
  dropping one lying behind a closer one; with ≥ 2 neighbours an item is pushed
  from the closest by `0.3·(avg−dmin)` and pulled to the farthest by
  `0.35·(dmax−avg)`. Repeated clicks keep moving things.
- **Graph:** libcola constrained majorisation over items joined by connectors.
Undo label "Rearrange".

### Frame "Remove overlaps"

`H:` and `V:` gaps (−1000…1000, step 1, default 0), and "Move objects as
little as possible so that their bounding boxes do not overlap". Each box
grows by gap/2 each side (never inverted); libvpsc `removeoverlaps`; each item
moves its centre to its solved rectangle's. Needs ≥ 2. Undo label "Remove
overlaps".

## Align tab — node mode (Node tool active)

**Align Nodes**, relative to: Last selected · First selected · Middle of
selection · Min value · Max value (default First). "Align selected nodes to a
common horizontal line" sets every Y; "…vertical line" sets every X.
**Distribute Nodes** horizontally / vertically: min and max stay, the rest
evenly spaced. Remove overlaps is hidden.

## On-canvas alignment handles

With the toggle on, a third click on a selected object cycles its handles to
**align**: sides, corners and a centre. A side aligns the selection to that
side of the selection box; Shift aligns to the outside (anchor) instead. The
centre centres on the vertical axis; Shift on the horizontal. Corners do both
axes. The tip advertises Ctrl for "group" and the code ignores it.

## Grid tab

Rows × Columns (1–10000, kept in step: `cols = ceil(n/rows)` and back; on a new
selection rows = cols = `ceil(√n)` unless both > 1); **Equal height** / **Equal
width** (on); **Alignment** — 3×3 anchor within a cell (centre); **Fit into
selection box** or **Set spacing** X / Y (default 15 px, the default mode).
Order is spatial: take the tallest item of the top band, everything whose Y band
contains its midpoint is the row, sort that row by left X, repeat. Column width
is its widest item, row height its tallest (or the overall maxima when Equal).
Origin: the selection box's min corner. Undo label "Arrange in a grid".

## Circular tab

**Anchor point:** Objects' bounding boxes (3×3 anchor, centre) · Objects'
rotational centers. **Arrange on:** First selected circle/ellipse/arc · Last
selected · Parameterized (Center X/Y 0, Radius X/Y 100, Angle start 0° / end
180°). **Rotate objects** (on). Count is reduced by one on a partial arc so
the ends are both used; `angle = begin + i/count·arc`; point `(cx + rx·cos,
cy + ry·sin)`. Rotation turns each object's top away from the centre. The
reference ellipse does not move. Undo label "Arrange on ellipse".
Source-observed: Parameterized divides by 0 or −1 in 1.3–1.4.2.

## Shortcuts

Ctrl+Shift+A opens it. Ctrl+Alt+keypad, relative to the dialog's current
choice: 4 left · 6 right · 8 top · 2 bottom · 7 horizontal centre ·
1 vertical centre · 5 both centres. Nothing else is bound.
