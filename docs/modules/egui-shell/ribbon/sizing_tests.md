# `egui-shell/ribbon/sizing_tests`

## Item notes

### `fn registry`

Each carries an icon **and** a tooltip. A fixture missing either would
make every `Small` in this file silently render as `Medium`, and the tests
asserting a narrower control would fail for a reason that has nothing to do
with what they are about — see [`super::sizing::resolved`].

### `fn shell`

Deliberately minimal: every test below compares two renders that differ in
one property, and anything else on the tab would be width the comparison
has to reason about.

### `fn item_area`

**THE ORACLE FOR "SPACE IS RECLAIMED", AND THE GROUP'S WIDTH IS NOT.**

A group's **width** is only a valid oracle for reclaimed space while the
group lays its items out on ONE ROW. `band::measure_group_rows` asks every
group for the band's full row ceiling, so two equal-width controls stack
into a column — and a column is exactly as WIDE with one item as with two.
A width comparison then prints the same number on both sides, which reads
as "the property has gone" and actually means "the measurement can no
longer see it".

A hole — an item measured but not drawn — is still a hole under either
layout, and under either layout it shows up as **area**: two items occupy
twice one item's area whether they sit side by side or one above the other.
So the assertion is layout-independent, which a width comparison silently
is not.

⇒ **When a layout change turns a passing assertion red, ask whether the
assertion was measuring the rule or the arrangement.** This project has
recorded the same shape against `ui-verify` repeatedly (*ask what the check
SAMPLED*); it applies to a unit test's choice of dimension identically.

### `fn render_with_icons`

The painter is the whole reason this file has its own render function
instead of calling [`render_shell_with`] like its neighbours. `Small` is
**earned** — it needs an icon, a tooltip *and* an installed painter — and
the shared harness installs no painter, so every `Small` in every test here
would silently render as `Medium` and the assertions would fail against a
perfectly correct implementation.

That is not a flaw in the shared harness: a ribbon with no icon painter is
a working ribbon, and its tests are right not to invent one. It is a
property of what this file measures.

The painter draws **nothing**. It exists to be `Some`. What is being
measured is the space a control reserves, and a painter that filled its
rect would be measuring `egui`'s compositor.

### `fn an_icon_only_control_is_narrower_than_a_labelled_one`

The whole point of `Small`, and the measurement that moved the 884-point
number in `RIBBON_SCALING.md` §3. Asserted as a *comparison* between two
renders of the same command rather than against a number, because the
absolute width depends on the synthetic face's metrics and a literal here
would be pinning the fixture rather than the rule.

### `fn a_large_control_spans_the_rows_a_medium_one_sits_in`

Height needs no font, so this one would pass without the synthetic face —
it is here because it is the same subject, and because a reader comparing
the three sizes wants the three assertions together.

### `fn a_hidden_item_is_not_drawn_and_its_space_is_reclaimed`

Both halves, because only the first is obvious and only the second is the
operator's ask. A `visible_when` applied at draw time would satisfy the
first and leave a hole: the group would still be measured at its full
width, the groups to its right would not move left, and *"shift the space
used depending on what exists"* would be false.

### `fn a_group_with_nothing_left_is_not_drawn`

Not "drawn empty", and not "drawn with just its caption". A caption over
nothing is a promise of a control that is not there, and the separator
beside it is a rule between two things with nothing between them.

### `fn a_small_that_has_not_earned_it_renders_at_medium_width`

This is the guard that lets a manifest ask for `Small` freely. Without
it, marking a tooltip-less command `Small` would ship an unlabelled
rectangle, and the author would have no way to know except by looking.

### `fn a_large_control_in_a_popup_is_tall_enough_to_click`

The sharpest failure this file holds shut.

A group drawn in the menu uses `GroupBox::NATURAL`, whose row height is
`0.0` **on purpose** — so a one-row group in the popup has no hole beneath
it. A `render_large` that allocated exactly the height it was handed would
give a Large control in the menu a rect of **zero height**: it paints (the
icon and label are placed from the rect's centre, which still exists), it
reports its rect as required, and it **cannot be clicked**.

No band-path unit test can see that, because the band hands a real row
height and only the menu path passes a zero; the observable is the
published rect's height, which is what a driven check reads back.

This drives the same path: a band too narrow for the group, a click on the
affordance, and an assertion about the rect the menu reported.

### `fn a_hidden_custom_item_is_never_offered_to_the_renderer_and_gives_its_width_back`

# Why this is asserted through the RENDERER rather than through a rect

A [`Item::Custom`] publishes no `ribbon.item.<id>` region: the shell does
not draw it and has no id to name it by. So *"was it drawn?"* cannot be
read out of the reported rects the way the command tests above read it, and
the honest observation is whether the application's renderer was **called**.
A count is that observation, and it is stronger than a rect would be: it
distinguishes *"the shell skipped the item"* from *"the shell called the
renderer and the renderer chose to draw nothing"*, which is exactly the
difference the field exists to remove.

# And the group narrows, which is the half that is easy to leave out

`super::sizing::visible` runs **before measurement**, so a hidden custom
item must give back `plan::CUSTOM_ITEM_WIDTH` rather than leaving a hole
the band has already budgeted for. Without the field an application could
only draw nothing into a slot the band had already reserved, which is a gap
on the band with no control in it.

### `fn render_wrapping`

Goes through the same `render_with_icons` the rest of this file uses, so a
change to the harness cannot make these three tests measure something the
others do not.

### `fn a_large_control_wraps_a_long_label_instead_of_running_on`

The defect this pins is not subtle once it is drawn: `Save a compacted
copy of this document` laid out on one line is a control roughly 200 pt
wide and 56 pt tall — a letterbox with a small picture floating in the
middle of it, which is not what a Large control looks like in Word, in
Acrobat, or in the mockup. It also pushes every group to its right off the
band, so the first visible symptom is *"why is Print in the overflow
menu"*.

The vacuity guard is the second assertion and it is doing real work.
Without it the test passes trivially against any implementation whose
labels happen to be short — including one that never wraps — because the
bound would never be approached. So the unwrapped width is measured too,
and the fixture is required to be a case that actually needs wrapping.

### `fn a_large_control_never_narrows_below_the_mockups_floor`

`.rb.big { min-width: 52px }`. Without it a run of Large controls is a
ragged fence — `New` measures `max(24 pt glyph, 21 pt label) + 16 = 40`,
`Open…` measures rather more — and a row of buttons of visibly unequal
width is the thing a ribbon is not.

The pair with the test above is the point: one asserts a ceiling, the
other a floor, and an implementation that satisfied only one of them would
be broken in a way the other could not see.

### `fn a_large_control_is_shorter_than_the_row_area_it_sits_in`

The mockup draws `.rb.big` at 56 px inside a 68 px row area, top-aligned
by `.grp .items { align-items: flex-start }`. A Large control that simply
*was* the row area differs visibly the moment a group holds nothing else:
full-height plates side by side read as one block of chrome rather than as
separate buttons.

Asserted as a **relationship between the two metrics and the drawn
rect**, not against 56. A literal would pass under `Quiet` and say nothing
about `Airy`, whose own pair is 64 in 84.
