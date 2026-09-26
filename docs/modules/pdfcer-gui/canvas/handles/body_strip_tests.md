# `pdfcer-gui/canvas/handles/body_strip_tests`

## Item notes

### `fn the_centre_of_a_short_field_is_the_body`

The measured case: a 160 × 20 pt form field at the operator's fitted
29.55 % zoom is 47.3 × 5.9 px. Before this rule, dead centre answered
`Grip::North` — so dragging the field to move it committed a degenerate
resize the engine then refused, and the operator's field did not move
and did not say why.

The numbers are the real ones from `widget-move.trace.txt` rather than
round ones, because the defect is a threshold and a rounded fixture can
sit on the comfortable side of it without anybody noticing.

### `fn a_short_box_keeps_its_whole_body_and_its_grips_sit_outside_it`

So the promise is restated at the level it was always really about:
**the body belongs to the body.** That is falsifiable against both
mechanisms, which the old wording was not.

Asserted separately from `the_centre_of_a_short_field_is_the_body`
because the two are different promises: one is about where a press lands,
the other about what the operator is shown. A grip painted where it
cannot be aimed is the affordance R9 forbids, and the painter reads this
same list.

### `fn the_smallest_object_the_shell_can_draw_is_still_grabbable`

The report, verbatim: *"zoom in on the atoms of the banana pdf file and
see what happens when you try to draw a box around a molecule and move
it, or select the ion and move it."* Driving it produced
`resize-declined reason=Degenerate` on every press, because the box was
floored to [`crate::canvas::overlay::MIN_OUTLINE_EXTENT_PX`] = 6 pt and
four corner grips reaching 6 pt each covered all of it.

The fixture is the **floored** box, not the 0.85 pt one, because the
floor is what the operator's pointer actually meets — testing the
un-floored rect would test a rectangle nothing on screen corresponds to.
