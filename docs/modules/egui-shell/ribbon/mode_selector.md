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

### `struct SegmentCues`

The same shape as [`super::tabs::TabCues`] and for the same reason:
R84 is a property of the *set* of cues, and a property of a set cannot
be asserted about expressions scattered through drawing code.

### `fn move_index`

See the module header on why this clamps rather than wraps. `len == 0`
is answered with `0` rather than a panic: an empty selector is not
drawn at all, so the value is never used, and a `panic!` in the paint
loop for an empty manifest would be a poor trade.

### `fn selected_index`

An unknown id resolves to the first position rather than to "nothing
selected", because a segmented control with no segment selected shows
the operator a state that is not one of the states the control offers.

### `fn segment_widths`

Every segment is the width of the widest label, so the control reads
as evenly divided track rather than as a row of differently sized
buttons — which is what makes it look like one control with positions
rather than like three buttons that happen to be adjacent.

### `fn measure_track`

Returns `(segment_width, natural_total_width)` — the *uncompressed*
numbers. [`fit_track`] applies the compression, and it does so inside
[`render`] against the room the row actually granted.

This exists because the reservation and the rendering must agree.
[`super::plan::plan_strip_row`] subtracts this figure from the row
before the tabs are planned; if the selector then measured itself
differently it would either overhang the tabs or leave a gap, and the
tab plan would be wrong by the difference.

### `fn fit_track`

# Why this exists — the same failure mode as the overflow affordance

Two things on this ribbon must never be squeezed out by content: the
mode selector and the overflow affordance ([`super`]'s module header
owns that rule). Laying the selector out first, from the right edge,
achieves it against **content**. It does nothing about the case where
the selector alone is wider than the row.

`egui` answers `allocate_exact_size` on a right-to-left layout by
extending **leftwards past the edge of the container**. So a track that
does not fit is not clipped, not shrunk and not warned about: it is
placed with its left portion off screen. At a 180 pt viewport with real
font metrics, a three-position *Read · Review · Edit* selector measures
189 pt and the first position lands at x = −9 — present in the layout,
unreachable with a mouse, and invisible in every test that measured
text as zero-width.

So the shortfall is spent on the **segments' width** instead of on
their position: every position stays on screen and stays clickable,
and the labels crowd. That is the same trade the overflow affordance
makes (see [`super::band`]) and it is made for the same reason — a
control the operator cannot reach has failed completely, whereas a
control whose label is tight has failed cosmetically.

`room` that is not finite or not positive means "no constraint known";
the natural size is returned unchanged, because clamping to a bogus
number would shrink a control that had plenty of space.

Returns `(segment_width, total_width)`, and the caller discloses a
shrink through the verification channel — see [`render`].

### `fn render`

Returns `None` when nothing changed, or when there are no modes at all
— an application with no modes gets no selector, rather than a control
with one position that does nothing.
