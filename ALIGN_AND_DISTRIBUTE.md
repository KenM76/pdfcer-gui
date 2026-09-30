# Align and Distribute — how Inkscape's panel maps onto pdfcer

`OPERATOR_REQUESTS.md` O263: *"a tab with exactly all the same functions and
features of the alignment tab in inkscape."* The reference, control by
control, is `docs/reference/inkscape-align-and-distribute.md`. This document
records how each item lands here and why.

## Where it lives

A dock panel, **Align and Distribute**, with Inkscape's three tabs (Align,
Grid, Circular). It is opened from Edit ▸ Arrange (one control, `edit.align`)
and by **Ctrl+Shift+A**. The Ctrl+Alt+keypad chords run the align buttons against
the panel's current *Relative to*, as Inkscape's do. A panel rather than a
ribbon tab because Inkscape's is a docked dialog with state (the Relative-to
choice, the group toggle, gaps, grid and circle parameters) that a ribbon
button cannot hold.

## What it acts on

Page content objects selected at the Object rung, on one page. That is the
only multi-selection this shell has: a markup and a form widget are single
selections, so there is nothing for them to align against. With a markup or
field selected the panel says so rather than acting.

Node mode acts on the selected anchors of the one entered object, through
`move_nodes`.

## The frame the arithmetic runs in

**Canvas space** — points, y down, with the page's `/Rotate` applied
(`viewer::canvas_to_pdf_space` at scale 1). Inkscape aligns in desktop space,
which is what the operator sees. Doing it in PDF user space would make
"left" mean the page's x axis, which on a rotated sheet is the operator's top.
Each item's box is its canvas-space bounds; each result is a canvas vector,
turned into a page delta by `canvas::moving::page_delta`.

The arithmetic is pure and lives in `pdfcer-gui-base` (`alignlayout`), taking
boxes and returning per-item deltas. It knows nothing of documents, which is
what lets every Inkscape formula be tested on numbers.

## How a result is committed

One action carries every item's own delta, in page-space points, and its arm
makes one engine call, `EditSession::move_objects_each`: one plan, **one undo
step**, all-or-nothing. The engine chooses the rewrite per object kind, so the
shell does not split paths from images.

## Item by item

| Inkscape | Here |
|---|---|
| Relative to: Last / First selected | `SelectionState::in_selection_order`. Shift-click appends; a marquee appends in document order. |
| Biggest / Smallest object | As Inkscape: compared on the dimension perpendicular to the alignment. |
| Page | The page's crop box, the sheet the operator sees. |
| Drawing | The union of every content object's box on the page. |
| Selection Area | The union of the selected boxes. |
| One item selected | The list shrinks to Page and Drawing, default Page, remembered apart. |
| Move/align selection as group | The toggle. |
| Align edges, centres, to-anchor | As the reference's table. |
| Align text anchors | Only text objects move, by their baseline origin: the first run's text matrix through the object's CTM. |
| Distribute (8) and text (2) | As the reference. |
| Exchange — selection / stacking / clockwise | Stacking order is paint order: the index in the page's object list. |
| Randomize, Unclump | Ported. Randomize is seeded per click. |
| Nicely arrange connector network | **Not offered.** A PDF has no connectors: no object records that it joins two others, so there is never a network to arrange. R9: nothing is drawn for it. |
| Remove overlaps | Ported (VPSC), with H and V gaps in points. |
| Node mode | Align and distribute the entered object's selected anchors. |
| On-canvas alignment handles | With the panel's toggle on, nine handles inside the selection box of two or more objects; click aligns to that side of the selection area, Shift+click just outside. Not a third-click cycle: this canvas shows resize grips and the rotate handle together, so there is no cycle to extend. |
| Grid tab | Ported, spacing in points. |
| Circular tab | Parameterized works as Inkscape's. *First/last selected circle/ellipse/arc* takes a selected path whose shape is an ellipse, measured from its anchors. *Rotational centers* is the box centre, because a PDF object has no stored rotation centre. |

## Staging

1. Selection order; the arithmetic for Align and Distribute; the panel's Align
   and Distribute frames; the keypad chords; one driven check.
2. Rearrange and Remove overlaps.
3. Grid and Circular.
4. Node mode and the on-canvas handles.

Each stage ships driven before the next starts.

**Built:** Stages 1–4. The panel has three tabs — Align, Grid, Circular —
and switches to node rows when anchors are selected at the Node rung. Grid
lays the selection out in rows and columns (rows 0 = as square as possible),
equal heights or widths, a nine-way anchor in the cell, fitted or set
spacing in points. Circular arranges on a Parameterized ellipse (centre,
radii, start and end angle, canvas points, y down) or on the first or last
selected object when its shape is an ellipse, anchored by object centre or
box, optionally turning each object to follow the arc — one
`transform_objects_each` call. Node mode aligns the entered object's
selected anchors on a vertical or horizontal line or spreads them, relative
to last, first, middle, min or max — one `move_nodes` call; anchors selected
on a second path are not moved, and the panel says so. *On-canvas alignment*
draws nine handles inside the selection box of two or more objects: a click
aligns to that side of the selection area, Shift+click just outside it; the
handles sit inside the box, not in a resize/rotate/align click cycle,
because this canvas shows its resize grips and rotate handle together.
The panel scrolls; Edit ▸ Arrange carries Align
and Distribute and the seven one-click aligns; Ctrl+Shift+A opens the panel;
Ctrl+Alt+keypad 4/6/8/2/7/1/5 run the aligns, keypad or top row. Rearrange
offers Exchange (selection, stacking, clockwise), Randomize and Unclump;
Remove overlaps takes H and V gaps in points. One driven check covers Align
left, keypad 5 and Exchange, each as one undo step. Remove overlaps,
Randomize and Unclump share Exchange's plan-and-commit path and are covered
by unit tests only; the three-box fixture has no overlap to remove.