# `egui-shell/ribbon/mode_selector`

## Item notes

### `fn arrow_movement_clamps_rather_than_wrapping`

The positions are ordered by capability. A wrap would turn one Right
press at the most capable stance into the least capable one — in the
control whose entire premise, per `MODES_AND_PANELS.md` Part 1, is
that the ordering is the information. A slider does not wrap, and a
slider is the chosen metaphor.

### `fn every_mode_gets_its_own_labelled_segment`

`MODES_AND_PANELS.md` Part 1 forbids a bare track with a knob, whose
available positions are invisible until you drag. The checkable form
of that is: N modes produce N segments, each with a non-empty label
and a positive width, and the total is exactly the sum. A
knob-and-track implementation fails on the segment count.

### `fn the_selector_is_generic_over_the_manifests_modes`

The point of the test is the *absence* of Read/Review/Edit from
this crate. `SHELL_FRAMEWORK.md` §3 requires it, and a shell that
hard-coded three stances would be un-reusable in exactly the way
the whole design exists to avoid.

### `fn a_segment_label_is_never_empty`

An unlabelled position in a segmented control is indistinguishable
from a gap in the track, which is the exact failure the "all
labels visible" rule names.

### `fn an_unknown_selection_falls_back_to_the_first_position`

A segmented control with no segment selected shows a state that is
not one of the states the control offers, and the operator has no
way to find out which one they are actually in.

### `fn a_track_that_does_not_fit_is_compressed_rather_than_pushed_off_screen`

`MODES_AND_PANELS.md` Part 1 requires every position to be visible
and operable; [`super`]'s header adds that the selector is one of
the two controls that must never be squeezed out. `egui` answers an
over-wide `allocate_exact_size` in a right-to-left layout by
extending past the container's left edge, so without this clamp the
first position lands at a negative x — drawn, reported and
unclickable.

With no font data every label measures zero, the track is always
3 × `MIN_ITEM_WIDTH + SEGMENT_PADDING`, and no realistic row is
ever narrower than that — which is why this needed a *pure* test
rather than only a rendered one.
