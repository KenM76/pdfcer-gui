# `app::actions::strokestyle` — line width, dash and opacity of page objects

Applies `Action::SetObjectStrokeStyle` through the Document-scoped
`vector_edit` funnel with `EditSession::set_object_stroke_style`.

## Units

The action carries width and dash in **points**, because that is what the
operator types. The verb takes them in each path's **own user space**
(`PathObject::line_width` and `PathObject::dash` are user-space values). A
path's points-per-unit is `canvas::shapes::average_scale` of its `ctm`, the
same measure the canvas uses to draw its stroke.

So the arm reads every object's scale from `EditSession::page_objects`, groups
the objects whose scales agree within a relative `1e-6`, and makes one verb
call per group with width, dash array and phase divided by that group's scale.
A style carrying neither width nor dash (opacity only) is unitless and goes in
one call.

Every call's style is validated before the first commits, so a value one
group cannot take leaves the page untouched.

## One undo step

Several calls are several commands. `EditSession::coalesce_last(n,
CommandKind::SetObjectStrokeStyle)` folds them into one. When it declines, the
outcome says how many steps the undo takes rather than hiding it.

This grouping is a workaround: the verb has no way to take points or a
per-object width. It is reported to the engine as G174.

## What the engine refuses

Text, and any image or form the style would not change, comes back in
`refused`. The outcome sentence counts them; nothing on the canvas marks them.
Width and dash never reach an image or a form; images read only the fill
alpha.

## Shared resources

The alphas land in a new `/ExtGState`. When the page's resources are shared
with other pages the engine says so in a disclosure, which the funnel shows.

## Trace

`stroke-style-calls calls= commands= folded= changed= refused= user_widths=`
— `user_widths` is the width each call sent, in user units, `-` for none.
