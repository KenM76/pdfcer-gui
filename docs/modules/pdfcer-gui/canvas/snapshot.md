# `canvas::snapshot`

The View > Snapshot tool on the canvas (O272): arming it, laying, moving and
resizing the box, clearing it, and drawing it through the frame's mapping.

## Route

`view.tool_snapshot` (View > Navigate, beside the hand) → `toggle` arms
`CanvasTool::Snapshot`, or returns to Select when it is already armed. A press
with the tool armed means `DragKind::Snapshot` in every mode — a snapshot reads
the page and authors nothing, so Read may use it. The drag becomes
`GestureOutcome::Snapshot`. In `canvas::interact` each frame of it asks `band`
for the rubber band to paint, and hands the frame to `dragged` once the frame's
`page_objects()` borrow is dropped, the first point where `&mut doc` is
available.

## What a drag does

Decided once, on the drag's first frame, and held in egui temp memory keyed by
the press point (a different press point is a new drag):

| Press lands | Drag does | Trace `part=` |
|---|---|---|
| outside the box, or no box on this page | draws a new box; the band is painted until release | `new` |
| inside the box | moves it whole | `move` |
| on one of the eight grips | moves the edges that grip holds | `resize` |

The hit test is `handles::grip_at` with `GripSet::scale_only()` against the
box's **screen** rect, so the grips have the same grab size as every other
object's. Move and resize work in **canvas** space — whose axes are the
screen's on every page rotation — and convert back to PDF user space through
`canvas::markup::band::endpoints`, so a box on a turned page resizes along the
edge the operator sees. A box follows the pointer live; a resize past the
opposite edge flips the box. A box narrower than `SnapshotBox::MIN_SIDE_PT`
on either side is not kept: the last one stays.

## Cursor

`cursor` overrides the tool's crosshair over the box: a grip's resize cursor
on a grip, `Move` over the body, and `Grabbing` while a move is held. Outside
the box, or while a new box is drawn, the crosshair stays.

## Clearing

The box lives while the tool is armed. `settle`, called once a frame from
`app::frame` beside the text draft's settle, clears it whenever the selected
tool is anything else — Escape, the button, another tool, a mode change —
because the condition is a property of the state, not of any one exit.

## Drawing

`paint` runs after the marquee, on every page drawn. It converts the box's
corners back with `viewer::pdf_space_to_canvas` and `PageMapping::rect_to_screen`
— the same two steps every page-space overlay takes — so the box follows the
page through zoom and scroll by construction. It strokes 1 px in the theme's
canvas selection ink and declares the screen rect as region `canvas.snapshot`.

The box is a pre-commit affordance (R8b): it marks a choice the operator is
making, not applied content, and a save never contains it.

## Trace

| Line | When |
|---|---|
| `snapshot-tool armed=true\|false` | the command toggles the tool |
| `snapshot-box page= llx= lly= urx= ury= part=new\|move\|resize` | a drag is released |
| `snapshot-cleared reason=tool` | the tool is put down with a box laid |
| region `canvas.snapshot` | every frame the box is drawn |
