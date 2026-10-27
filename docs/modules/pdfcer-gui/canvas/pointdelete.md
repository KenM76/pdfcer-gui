# `canvas::pointdelete` — Ctrl+Alt+click deletes the point under it

With the Points tool armed, one Ctrl+Alt+click on a shown point deletes it, as
Inkscape's node tool does. It is the one-click route beside the slower ones:
the right-click *Remove this point* row, the Ctrl+Shift drag on a markup node,
and select-then-Delete on a page-content anchor.

## Contract

- `markup(doc, map, selection, point, actions) -> bool` — `point` is canvas
  space. Hits a node of the selected markup shape with
  `annotnodes::node_at` and raises exactly what the menu row raises
  (`annotnodes::menu::remove_node`). `false` when no node is there; nothing
  changes.
- `anchor(selection, page, targets, hit, epoch, actions) -> bool` — `hit` is
  the click ladder's probe. With `hit.node` set, selects that anchor and hands
  it to the Delete key's ladder (`deleting::subject` / `deleting::action`); a
  refusal goes through `deleting::decline`, so its sentence is the Delete
  key's. `false` with no anchor; the selection is untouched.
- Both trace `point-delete via=markup|anchor node=N outcome=raised|declined`.

## Why the chord is Ctrl+Alt

Ctrl already toggles membership of the selection and Shift extends it, so
neither alone can mean delete. Ctrl+Alt+click is Inkscape's node-tool delete,
which is the convention the operator named.

## Why each kind reuses another route's plan

A second plan for the same edit is a second set of refusals that drifts. The
markup arm reuses the menu row's plan, so a shape that would fall below the
engine's minimum vertex count is refused in the same words. The anchor arm
reuses the Delete key, so a degenerate path or an unaddressable object is
refused in the same words.

## Where it is called

`clicking::click`, in the Points-tool branch:

- the markup arm runs **before** the empty-click deselect, because that
  deselect would otherwise clear the shape whose node was hit;
- the anchor arm runs after the probe and replaces `click_direct` when it
  fires.

`interact.rs` sets `Frame::delete_point` from Ctrl and Alt both held.
`gesture::meaning::press_kind` lets the Points tool produce a click wherever
markup can be reshaped (Review included), so the click reaches this ladder at
all.

Driven by `ctrl_alt_click_deletes_a_markup_point` and
`ctrl_alt_click_deletes_a_page_point`.
