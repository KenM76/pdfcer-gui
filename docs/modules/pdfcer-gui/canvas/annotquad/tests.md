# `pdfcer-gui/canvas/annotquad/tests`

## Item notes

### `fn turned`

Through `add_markup` and `rotate_annotation`, never a hand-built
dictionary: the appearance stream, its `/BBox` and its `/Matrix` are then
the ones the engine actually writes, so a change in the engine's convention
turns these red — which is the notification this shell wants rather than a
surprise on a real file.

### `fn a_thirty_degree_turn_arrives_with_the_artworks_own_dimensions`

This is the operator's sentence reduced to arithmetic, and it is the one
end-to-end assertion worth keeping here: `/Rect` after a 30° turn of a
140 × 60 mark is about 151 × 122, and the corners this module hands the
painter must still measure 140 × 60.

It is a test of the **adapter and the projection contract**, not of
§12.5.5 — if the engine's placement were wrong this would fail, but the
engine has five tests of its own saying it is not. What this catches is the
shell passing the wrong `Annotation`, the wrong view, or dropping the
corners on the way through.

### `fn an_unturned_annotation_is_upright_and_reports_zero`

The base case, and it is what makes the branch above it meaningful: the
canvas takes the cheap axis-aligned path on `is_upright`, so a build that
answered `false` here would put every ordinary mark on the turned code path
for no reason, and one that answered `None` for the angle would leave the
properties panel's Angle field blank on the commonest annotation there is.

`Some(0.0)` rather than `None` is the **engine's** deliberate choice for
an appearance with no `/Matrix` key — Table 95's default, and what the
renderer paints with. `Annotation::appearance_matrix` answers `None` for the
same annotation, because the file really did say nothing, and this shell
wants the method's answer rather than the field's.

### `fn a_quarter_turn_is_not_upright_even_though_its_rect_fits`

A 90°-turned annotation's `/Rect` bounds it exactly — the box is the
original with its sides swapped — so an outline drawn from `/Rect` looks
perfectly right. Its **corner order** has rotated, though, so a grip the
operator grabs at the artwork's own top-left is at the page's bottom-left.
Answering `true` here would put the grips back on the page's frame and
reintroduce that mismatch on exactly the rotation an operator makes most
often.

### `fn the_engine_owns_the_placement_and_this_module_only_projects`

**It is kept, inverted**, because the workaround it guarded is gone and the
opposite hazard is now the live one: this module must go on being a thin
adapter, and the way it stops being one is somebody re-deriving the angle or
the placement here *"just this once"*.

# What it asserts, and why it is keyed on the engine rather than on us

That `Annotation` **still has** `appearance_matrix`. A grep of this file for
a local decomposition would be the obvious test and it is the weaker one: it
asks whether *we* misbehaved, when the fact that matters is whether the
**engine still owns the answer**. If a future engine withdrew the field, the
honest response is a new request — not a quietly restored matrix reader —
and this failing with that instruction is how that gets decided in the open.

⚠ It **FAILS rather than skips** when it cannot find the source. The crate
could not have compiled without that checkout, so a red here means the
locating is broken and needs fixing — never ignoring. A hard-coded external
path turning a rename into a green check over an empty scan is a mistake
this project has already made once.

It reads the **cargo checkout**, located through `Cargo.lock`, not
`D:/Dev/pdfcer` — that working tree moves several times a day, often ahead
of what compiles here, so a tripwire on it fires on work that is not in the
binary.

### `fn a_clockwise_turn_is_normalised_into_the_same_range_as_an_anticlockwise_one`

The engine's `Annotation::appearance_rotation_degrees` returns a signed
`atan2`, so a quarter turn clockwise reads **`-89.15`**. This module needs
`[0, 360)` — the properties panel shows the number, and
[`OrientedBox::is_upright`] range-tests it — so it normalises with
`rem_euclid`.

# Why this test exists, in one sentence

Because the first version of this adapter did **not** normalise, and every
other test in this file used a **positive** angle, so all of them passed
while every clockwise rotation reported itself upright and the selection
outline silently went back to being axis-aligned.

⇒ **3,860 in-process tests were green.** The defect was found by
`tools/ui-verify`'s `rotating_a_markup_turns_it`, whose drag happens to be
clockwise — one driven run, ninety seconds, on a build the whole suite had
signed off.

It asserts **both signs in one test**, deliberately. A test named *"a
negative angle normalises"* sitting beside four positive ones would be as
easy to leave un-run as the case it guards; asserting the pair means the
property under test is *the round trip is sign-agnostic*, which is what was
actually assumed.

−89.15° rather than −90°: it is the angle the driven check's drag
actually produces, and a round number would sit exactly on the boundary of
the quarter-turn case that has its own test above.
