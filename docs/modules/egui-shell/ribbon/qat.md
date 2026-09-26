# `egui-shell/ribbon/qat`

## Item notes

### `fn control_pieces`

# ⚠ The icon slot is allocated whenever the command **names** an icon

Not whenever one can be painted. [`super::band::command_button`] pushes
an `egui::Atom::custom` of `icon_pts` for any command with an icon key,
and only *paints* into it if the application supplied a painter — so a
command with a key and no painter draws an empty square, and that
square has width.

Budgeting it on `ctx.icons.is_some()` instead is the subtle way to get
this wrong: the measured QAT comes out narrower than the drawn one by
`icon_pts + icon_spacing` per control, the row grants it that narrower
figure, and the controls then overflow their own region and are drawn
across the first tab. `the_tab_strip_never_runs_under_the_mode_selector_or_the_qat`
catches it at 128 pt.

The rule this restates is [`super::band::measure_item`]'s, which has
always had it right: **measure what the renderer draws, not what the
application intended.**

### `fn a_control_goes_icon_only_only_when_it_has_a_tooltip`

The tooltip *is* the accessible name of an icon-only control. A
command with an icon and no tooltip, drawn icon-only, would
announce nothing to a screen reader and explain nothing on hover
— and would look completely correct in a screenshot, which is why
this needs a test rather than a review.

Falling back to the label converts an accessibility failure into a
cosmetic one. That is the trade this rule makes, deliberately.

### `fn a_control_keeps_its_label_when_the_application_cannot_paint_icons`

Registering an icon key is an *intention*; supplying an
[`super::Ribbon::with_icon_painter`] is the *capability*. Only the
capability may be traded against the label, because trading on the
intention alone produces a control with no glyph, no text and no
explanation — a blank box.

This is not hypothetical: an application can register a full set of
icon keys and no painter, pass every unit test, and draw a QAT of
empty rectangles. It is the rule `DEFECTS.md` D2 states — a property
that only exists once something is *rendered* cannot be asserted
from the values that went into it.

### `fn every_qat_control_has_an_accessible_name_whatever_the_rule_decides`

Stated as a joint property because the two functions are only
correct *together* — either alone can be changed into something
that leaves a control anonymous.
