# `ui-verify/checks/rotate`

`rotate_handle_turns_a_selection` — **the ninth grip**, driven end to end.

# What this is for

`ui-conventions/handles.md` H2 says the standard set is *"eight resize
grips, a body, and a rotation handle offset outside the top edge"*, and it
quotes the operator's own report as the failure mode:

> *"unfortunately there was no way to reposition, resize, or rotate it on
> the screen. Can I please please please have that too?"*


# Why this cannot be a unit test

`canvas::rotating`'s arithmetic is pure and has eight of them. What they
cannot reach is the chain:

| # | link | its own test |
|---|---|---|
| 1 | a click selects an object and the outline draws **nine** affordances | `canvas::handles` — the layout, not the hit |
| 2 | a press above the top edge finds `Grip::Rotate` rather than empty page | `handles::grip_at` — the geometry, not the routing |
| 3 | it becomes `DragKind::Rotate` and **not** `DragKind::Move` | `gesture::meaning` — and this is the link that would fail silently |
| 4 | the bearing is measured from the selection's centre in screen space | `canvas::rotating` — yes, and it is pure |
| 5 | the sign survives the screen → page crossing | **nothing** |
| 6 | every selected index reaches `transform_objects` as one command | **nothing** |


Link 5 is the one that would look like a feature: an object that turns the
wrong way is not obviously a defect to anybody who did not watch the
pointer.

# The oracle carries DEGREES, and a signed number

`rotate-commit deg=… px=… py=… objects=… constrained=…`. A line saying only
*"a rotation committed"* would be identical for a build that turned the
other way, pivoted about a corner instead of the centre, or snapped when it
should not have. This project's standing rule, stated by `resize.rs` and
earned by `DEFECTS.md` D14: **a trace line must carry the number a wrong
build would get wrong.**

# What it drives, and why that shape

A quarter turn clockwise: press on the handle, then release **due east of
the selection's centre**. Two properties make that the right gesture to
drive rather than a small nudge:

* the expected answer is a round number a human can check by reading the
  trace, and
* it crosses no quadrant boundary, so a build with a broken wrap still
  produces the right answer here — which means a failure of *this* check is
  about the routing rather than about the arithmetic the unit tests already
  cover.
