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

### `fn shows_label`

`true` unless **all three** hold: the command names an icon, it has a
tooltip to serve as that icon's accessible name, and the application
actually supplied a painter that can draw the icon.

The first two are the accessibility rule described in the module
header. The third is a *rendering* rule: an application that registers
icon keys but supplies no [`super::Ribbon::with_icon_painter`] would
otherwise get a row of blank boxes — controls with no label, no glyph
and no explanation. That is precisely the placeholder the shell's
no-placeholders rule forbids, and it looks to an operator exactly like
the application is broken.

Declaring an icon is an intention; being able to paint one is a
capability, and only the second may be traded against the label.
Falling back to text is always safe — it is what a command with no
icon does anyway — so the degraded state is a slightly wider button
rather than an invisible one.

### `fn measure`

# Why an unmeasured QAT is not an option

The tab-strip row reserves space outermost-first
([`super::plan::plan_strip_row`]), and a reservation you cannot measure
is not a reservation. Emitting the QAT into a left-to-right layout and
taking whatever it takes is the immediate-mode spelling that produces
`MODES_AND_PANELS.md` failure mode #8: measured at a 180 pt viewport
with real font metrics, the two-control QAT of the test manifest runs
from **x = −6** to x = 160, with the tabs behind it entirely off
screen.

The measurement mirrors [`render`] line for line, and it has to:

- the same `shows_label` decision, because an icon-only control is
  much narrower than a labelled one and getting that backwards
  mis-budgets every QAT that has a painter;
- the same skip for an unknown command id, because a control that is
  not drawn must not be budgeted (the same rule
  [`super::band::measure_item`] follows, for the same reason);
- the trailing `ui.separator()`, which is the visible divider between
  the QAT and the tab strip and is part of the QAT's cost.

Returns `0.0` for an absent or empty QAT, so a manifest without one
costs the row nothing at all rather than a stray separator.

### `fn min_control_width`

[`super::measure::min_button_width`] is the text-only case; a QAT control
may also carry an icon slot, which `truncate()` cannot shrink at all.
So the floor is *per control*, and it is what [`render`] tests each
control's remaining room against before drawing it — see that
function's header on why drawing it anyway is not an option.

### `fn min_width`

[`super::plan::plan_strip_row`] uses this as the QAT's `grant` floor:
below it the region cannot hold even one control, and granting a
sliver produces a button drawn outside its own rectangle rather than a
narrower one.

Zero for an absent or empty QAT, so a manifest without one is not
charged a floor it will never use.

### `fn render`

Ordering is the manifest's; unknown ids are skipped with a disclosure
by [`Ctx::command`].

# How a QAT that does not fit degrades, and why it is not "truncate"
alone

The caller lays this out inside a `Ui` whose `max_rect` is the width
[`measure`] asked for and [`super::plan::plan_strip_row`] granted. When
those differ — a QAT wider than the row can afford — the controls
truncate, *and the ones that still do not fit are not drawn at all*.

The second half is not belt-and-braces; it is required, and the reason
is measured in [`super::measure::min_button_width`]:
**`Button::truncate()` stops shrinking** at padding-plus-ellipsis. Ask
a button to lay itself out in 6 pt and it lays itself out in 19.7 and
overflows — silently, because `egui` does not clip children to a `Ui`'s
`max_rect`. A loop that merely truncated would therefore walk straight
out of the QAT's rectangle and draw over the tab strip, which is the
defect [`super::strip`] exists to retire, reached by trying to be
accommodating.

So each control is drawn only while a whole button still fits, and the
ones dropped are disclosed as `ribbon-qat-controls-dropped`. Dropping
is a real loss — the QAT is *"the handful of controls that must never
sit behind a tab switch"* — and it is why the row reserves the QAT
first and why the disclosure exists. It happens only at widths where
the alternative is a control drawn on top of another one.

The trailing `ui.separator()` is subject to the same rule: it is the
divider between the QAT and the tabs, and a divider drawn past the end
of the QAT would be a rule through the middle of the first tab.
