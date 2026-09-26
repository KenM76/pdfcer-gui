# `panels::dimension_groups::style` — a group's appearance defaults, and
the count that stops them being a surprise

## What this draws

The **middle tier** of `pdfcer-core`'s three-tier style cascade
(factory → group → ce dimension). Seven properties, each an `Option` on
`GroupStyle`, and **the `Option` is the operator's checkbox**: clear means
*this group has not spoken, so pdfcer's own default applies*, ticked means
*this group says this*.

Two of the seven — `tolerance` and `tolerance_places` — are not drawn here
and the reason is at [`show`].

## The count beside every control, and why it is not the engine's

The operator's own words, quoted in
`docs/ui_specs/tool-options-dock-and-ce-dimension-properties.md` §C.11.1:

> *"cannot change one and be surprised 40 others changed or didn't."*

`EditSession::set_group_style` returns a count, and it is **the wrong
number to show**. `docs/core-api/03-capabilities.md` §1.6 trap (a) says so
outright: the return is the number of members *regenerated*, which is every
wired member including the ones that override the very property being
changed — because regenerating an overrider is byte-identical and free in
the diff, so the engine does not bother to exclude them.

The number that will visibly **move** is the members whose
`StyleProvenance` for that property reports `follows_group() == true`, and
it has to be computed **before** the edit if it is to be shown before the
edit. [`will_move`] is that computation, and it is called every frame for
every drawn property so the sentence under a control is always about the
model as it stands.

## `Factory` counts as following the group, and this is the easy thing to
get wrong

`StyleSource::follows_group()` is `true` for **both** `Factory` and
`Group`. A property nobody has set yet *will* move when the group sets one
— the group simply has not spoken. A panel that derives the predicate by
hand and tests only for `Group` greys out, or under-counts, exactly the rows
that are about to change.

`pdfcer-core` pins that with a test named for the trap
(`factory_sourced_properties_still_follow_a_group_edit`), and this module
never re-derives the predicate: it calls `StyleSource::follows_group`.

## Item notes

### `const POINT_SPEED`

Slow enough that a drag lands on a tenth rather than skating past it. These
are typographic sizes — 10 pt text, a 0.75 pt line — where the difference
between 0.7 and 0.8 is visible on a plot.

### `const TEXT_HEIGHT_RANGE`

Bounded here rather than in the engine because the engine does not bound
it: `GroupStyle::text_height` is a bare `Option<f64>`. The floor is the
smallest size that survives a 1:100 plot; the ceiling is where a label stops
fitting between its own witness lines on an A3 sheet. Neither is a hard
refusal — an operator who needs 60 pt can set it from the CLI, which is the
right place for a value outside what a drawing normally uses.

### `fn property_row`

# Why the checkbox and the editor are one function


# Why the editor is drawn only when the box is ticked

R9: greying is for *temporarily* unavailable, and an inherited property is
not unavailable — it has a value, supplied by a tier above. A greyed
spinner showing the factory number would invite the operator to drag it and
then decline, which is the affordance-that-cannot-be-honoured shape. The
caption in its place states the inherited value in words instead, so the
information is not lost with the control.

# Why `reach` is one parameter and not two

`(moving, total)` travel together into one sentence and are meaningless
apart — *"3"* says nothing without *"of 40"*, and that is the whole point of
the disclosure. Passing them as a pair also keeps this function inside
clippy's argument budget without dropping the `describe` closure, which is
what renders the inherited value in words when the editor is absent.

### `fn will_move`

`pick` selects the property out of the engine's own
`StyleProvenance` — one field per property, and the struct is what
`style_provenance` returns, so nothing here re-derives which tier supplied
a value.

# Why the predicate is the engine's and not `== StyleSource::Group`

Because `StyleSource::follows_group()` is `true` for `Factory` as well, and
that is the whole trap. A member that has never had the property set
anywhere follows the group the moment the group speaks. Testing for `Group`
alone would report zero on a fresh document — every member `Factory` — which
is the case where the count matters most, because that is the press that
changes everything on the sheet.

### `fn color32_of`

Opaque, because `/C` on an annotation has no alpha and a picker offering one
would be a channel pdfcer silently ignores — the argument
`canvas::markup::pen` already makes for the markup swatches, applied to the
engine's `Rgb` rather than to the pen's own triple.

### `fn rgb_of`

Divides by `255.0` rather than `256.0`: the component range is inclusive at
both ends, so `255` must map to exactly `1.0` or a pure red chosen in the
picker would be written as `0.996` and round-trip to a slightly different
swatch.

### `fn a_member_that_overrides_nothing_is_counted_as_moving`

This is the test that would have caught a hand-rolled
`== StyleSource::Group` predicate, and it is written against a *fresh*
model precisely because that is the case where the wrong predicate
reports zero and the right one reports everything.

### `fn an_override_is_excluded_from_its_own_property_and_no_other`

The second half is the one worth having: a member overriding the text
height still follows the group for the line width, and a count that
excluded it from both would under-report the wider edit.

### `fn a_colour_survives_the_round_trip_at_both_ends`

`255 → 1.0 → 255` is the case the `/ 255.0` divisor exists for. With
`256.0` a pure red would be written as `0.996` and the swatch an operator
reopened would not be the one they chose — a difference small enough to
dismiss and permanent once it is in the file.

### `const REGION_TEXT_HEIGHT`

One driveable, non-popup control is named deliberately.
`NO_SURFACE.md` §4 records what a colour picker costs a harness: *"the
picker's popup publishes no regions, so a check can assert the swatch was
drawn and driven but cannot aim at a hue inside it."* Any group of controls
containing a colour picker therefore needs a numeric neighbour the harness
**can** move, and this is it.

### `fn show`

# Read-modify-write, once per frame at most

The section starts from the group's **live** style, mutates a copy as the
operator touches controls, and raises one action if the copy differs at the
end. That is the CLI's own convention (`pdfcer group-style`: setting one
property leaves the others alone) and it is what keeps a click here and a
command-line invocation the same edit.

One action per frame rather than one per control is not an optimisation: two
`SetGroupStyle` actions raised in the same frame would each carry a *whole*
tier computed from the same starting point, so the second would silently
undo the first. The queue drains in order and the last writer would win.

# Why tolerance is not drawn here

`GroupStyle` carries `tolerance` and `tolerance_places`, so a group *can*
default them — and a group-level tolerance is the rarer half of the feature.
A tolerance is a statement about **one manufactured feature**: two holes on
the same drawing routinely carry different ones even though they share the
drawing's units and precision, which is the ui-spec's own reasoning
(§C.11.1) and the reference tool's.

So tolerance belongs on the **per-ce-dimension** surface, where it is drawn,
and putting a second control for it here would invite an operator to set a
group default that almost every member then overrides — the shape that makes
the moving-count read *"no change on screen"* on nearly every press. It is
reachable from the CLI for the drawing that genuinely wants one, and this
paragraph is the record of that being a decision rather than an omission.
