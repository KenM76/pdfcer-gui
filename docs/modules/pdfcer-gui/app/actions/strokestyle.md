# `app::actions::strokestyle` — line width, dash and opacity of page objects

Applies `Action::SetObjectStrokeStyle` through the Document-scoped
`vector_edit` funnel with `EditSession::set_object_stroke_style`, or, when
the action's `leaves` is set, with `EditSession::set_object_stroke_style_in_form`
on leaves of one placed drawing (`PageObjects::leaves`). `object_of` picks an
object by index from the right list; Properties reads through it.

## Units

The action carries width and dash in **points**, because that is what the
operator types. The arm sends the style with `units: StrokeUnits::Page`; the
engine divides the width, dash array and phase by each path's own
`PathObject::stroke_scale` before writing, so one call strokes a selection
under mixed CTM scales alike, and one call is one undo step. A leaf's scale is
taken from its page-space CTM by the engine, so the in-form twin needs nothing
different.

`PathObject::line_width_pt` and `dash_pt` read the value back with the same
measure, which is why Properties shows those and not a scale of its own.

A path whose CTM scales its axes differently has no single width in points:
the engine uses the geometric mean and says how many paths that was in a
disclosure, which the funnel shows.

## What the engine refuses

Text, and any image or form the style would not change, comes back in
`refused`. The outcome sentence counts them; nothing on the canvas marks them.
Width and dash never reach an image or a form; images read only the fill
alpha.

## Disclosures

The page verb's `PaintOutcome::disclosures` and the in-form twin's
`FormSurgeryOutcome::disclosures` are appended to the outcome's notes: the
non-uniform-scale sentence above, and the shared-resources sentence when the
new `/ExtGState` lands in a `/Resources` dictionary other pages share.

## Trace

`stroke-style-calls calls=1 units=page width_pt= changed= refused=` —
`width_pt` is the width sent, in points, `-` for none.
