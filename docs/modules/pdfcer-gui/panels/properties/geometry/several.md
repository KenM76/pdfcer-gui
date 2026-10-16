# `panels::properties::geometry::several`

X and Y for a selection of more than one item: page objects, parts of a
placed drawing, form-field boxes and annotations, in any mix Align measures.

- **Subject.** `align::measure` over the selection; taken whenever it returns
  more than one box, before the single-annotation and single-object arms.
- **Fields.** X and Y are the page-space lower-left corner of the union of
  the measured boxes, converted corner by corner through the page transform.
  W, H and Rotate by are not offered: a typed size or angle for a group has
  no single meaning (scale each about what?), so the group fields stop at
  position.
- **Apply.** The typed corner minus the union's corner, converted to a canvas
  delta, goes to `align::shift`, which hands every box the same delta to
  `moves_action`: one `MoveEach`, `MoveEachInForm` or `MoveMixed`, so one
  Undo. Gesture `typed`.
- **Draft key.** `Subject::Several(hash of count and union)`: a different
  set, or the same set moved, reseeds the fields.
- **Trace.** `geometry-several n= x= y= bx= by= bw= bh= changed=`, the union
  in page space.

Driven by `ui-verify/checks/typedseveral.md`.
