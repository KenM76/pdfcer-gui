# `dialogs::scale` — what a dimension's number *means*

The Set-scale dialog. `measure.set_scale` was registered, drawn on
Measure ▸ Scale, and inert; `shell::commands::reach` recorded it as *"the
clearest statement of a missing arm in the crate"*, and the block on it was
never the model — it was this window.

## Why this is the sharpest gap in the measure feature

Phase 7 shipped three tools that place dimensions: Linear, Two-line, and
Radius/diameter. All three work. **None of the numbers they produce means
anything until a scale is set**, because a fresh group's scale is the
tri-state's *never-set* value — so every label reads in PDF points, which is
a unit nobody's drawing is in.

An operator could therefore place a dimension and read a number, and the
number was a measurement of the *paper* rather than of the thing drawn on
it. That is worse than a missing feature: it is a plausible answer.

## What was already built, and what was missing

`canvas::measure::scale` came across whole in the Phase 7 salvage and is
**pure, GUI-free and unit-tested**: [`ScaleEntryFields`] holds the two
co-equal entry paths, back-calculates through the engine's own
`preview_group_scale`, and hands back a `(ScaleState, NumberFormat)` ready
for `EditSession::set_group_scale`. It contains *zero* scale arithmetic of
its own — deliberately, so that a canvas-calibrated group and a
CLI-calibrated group are the same number.

What was missing was somewhere to type into. This module is that and
nothing else: it owns no arithmetic, no parsing and no units. It draws
fields, calls `sync_real_length`, shows `preview`, and raises one `Action`.

## The two entry paths, and why both exist

| path | what you give it | needs |
|---|---|---|
| **Real length** | *"this line I drew is 4'-7 1/2\""* | a drawn reference line |
| **Ratio** | *"1:100"* | nothing |

The real-length path is the recommended one and the one a drafter reaches
for, because it needs no arithmetic from them: point at a dimension the
drawing already states, type what it says, and the scale falls out. The
ratio path exists because it needs no drawn line — which makes it the only
path reachable from a dialog opened cold, and, as the source notes, an
accessibility win: a scale can be set entirely by typing.

**A cold-opened dialog offers the ratio path.** This paragraph used to
end *"drawing one is a canvas gesture (`ScalePick`) that is not yet armed by
any command"*, and that stopped being true on 2026-08-17: the **Measure it
on the drawing…** button in this very window arms it, and the dialog
re-opens on the real-length path with the measured length in it.

What survives is the reason the radio is **absent rather than greyed** when
no line has been drawn: greying is for *temporarily* unavailable, and with
no reference line there is nothing for the real-length path to be about. The
window says which path it is offering and why, because an operator who has
read the manual will look for the other one — and now the window is also
where they find it.

## Why the real-length field still accepts `4'-7 1/2"`

It is not offered here yet, and the parsing belongs to
`pdfcer_core::dimension::parse_length` either way — the grammar lives in core
once, so the GUI and the CLI cannot come to disagree about what `55 5/8"`
means. That is the same rule the print dialog follows about range parsers,
and it is why the field is text rather than a numeric spinner: a spinner
forces the operator to convert to a decimal and pick a unit by hand, which
is two chances to enter a number that is plausible and wrong.

## Item notes

### `fn reseed`

The one place [`Self::fields`] is rebuilt from the document, called from
both constructors and from the group picker. Centralised rather than
inlined three times because the thing that must not drift is *which*
fields survive a reseed.

# What is DELIBERATELY carried across

`use_real_length`, `real_length` and `real_length_text` belong to the
**reference line**, not to the group. An operator who picked two points,
typed `25 ft`, and then realised they had the wrong group selected must
not lose the measurement by fixing the selection -- that would make the
picker a control that punishes its own correct use.

Everything else -- the ratio pair, the basis, the display unit and the
number style -- is a property **of the group** and is replaced, because
carrying those across a group switch is the O192 defect one row down:
controls describing a group the operator has navigated away from.

A missing group is a no-op rather than a fallback. The caller that
can encounter one ([`Self::show`]) has already redirected
[`Self::group`] to [`DEFAULT_GROUP_ID`] and called this again, so a
second policy here would be a second answer to one question.

### `fn group_row`

# Why the two live in one function

They are one disclosure. The scale phrase is meaningless without the
name above it -- *"Currently: 1:50"* about an unnamed group is the same
defect O193 reports, restated -- and the name is thin without the
phrase. Drawing them together is also what keeps them **reading the
same `group`**: two functions each resolving the id separately is one
refactor away from a window that names one group and describes another.

# The reseed is here, not in the picker's closure

`selectable_value` writes through a `&mut GroupId`, so the change is
detected by comparing against the value from before the combo rather
than by a click handler. That is deliberate: a click handler would miss
a change made by the keyboard, and the whole point of publishing
[`REGION_GROUP`] is that a driven check can reach this control without
a mouse.

# Rule 4 -- fuzzy, never sneaky

The current-scale line is **disclosure, off-canvas, non-blocking**, and
it is phrased as a statement of fact rather than as a caution. A group
with no scale renders the engine's own `NO_SCALE_DISCLOSURE` verbatim
through [`crate::text::dimension_groups::scale_phrase`] -- which is a
disclosure the engine requires be shown rather than paraphrased, and
which this window must not "improve" into a warning. Nothing about any
of this marks the page.

### `fn commit`

# Why a single action and not a call

`set_group_scale` **re-propagates every member's baked appearance
stream** — a dimension's label is drawn into its `/AP`, so changing the
scale rewrites every dimension in the group. That is a document edit
with an undo step, and the funnel exists so that every such edit is
ordered against every other and appears once in the command log.

One `Ctrl+Z` undoes a recalibration, whatever it touched. That is the
group model's whole promise — *a group exists so its members agree* —
and it would be broken by a dialog that issued one call per member.

### `fn unit_combo`

`Unit::ALL` rather than a hand-written list, so a unit the engine gains
appears here without anybody remembering — the same rule
`MarkupKind::ALL` and `Preset::ALL` are read under elsewhere in this crate.

### `const FRACTIONS`

`FractionMode`'s variants carry data — `Decimal { places }`,
`Fraction { denominator, reduce }` — so there is no finite set to enumerate
and this is a **curated** one rather than an exhaustive one. That is the
right shape: the useful decimal places are one to three and the useful
denominators are the binary ones a drawing writes, and offering a spinner
over every `u32` would be a control whose range is mostly nonsense.

`reduce: false` throughout, which is the architectural convention the
engine's own docs name: `6/8"` rather than `3/4"`, because a drawing
dimensioned to eighths writes eighths.

### `fn fraction_combo`

The `None` entry is first and is what an operator who never opens this
control gets. It is a real choice rather than an absence: an explicit
selection must survive a unit change, which is why the field stores
`Option<FractionMode>` rather than re-deriving from the unit — an operator
who asked for eighths does not want them silently reverted by switching from
inches to feet.

### `const REGION_CURRENT`

A region rather than only a trace line, because the defect being closed is
that the operator could not **see** the scale. A trace proving the string
was computed would be the same evidence the old window could have produced;
what has to be asserted is that it was drawn, with a rect, inside the
window's body.

### `fn open`

# It reads the document now -- O192

This constructor took a bare [`GroupId`] until 2026-09-13 and seeded its
fields from [`ScaleEntryFields::for_group_panel`], which is seeded from
nothing. So the window opened reading `1:100` in metres over a group
calibrated to `1:50` in inches, and an operator who pressed *Set scale*
without touching a control silently recalibrated the whole drawing to a
number the window had invented. The operator's report named the visible
half -- *"does not show me the scale that is already set"* -- and the
invisible half is the one that could have damaged a file.

[`ScaleEntryFields::for_group`] carries the whole inversion and the
proof that it is exact. The only thing done here is choosing the path:
**ratio**, because a cold-opened window has no drawn reference line and
the real-length path cannot produce a scale without one.

### `fn calibrated`

The calibration path's constructor. Raised by the application when
`ScalePick::dialog_open()` turns true — i.e. on the click that completes
the two-point pick.

# It seeds the REAL-LENGTH path, not the ratio one

`ScaleEntryFields::default()` rather than `for_group_panel()`, and that
is the whole difference between the two constructors. `for_group_panel`
exists to pre-select **ratio** because its situation is "no reference
line was drawn"; here one was, so the path the operator just did the
work for is the one that should be waiting for them.

The ratio path stays available in the same window. An operator who
picks two points and then decides they would rather type `1:100` can,
and nothing is lost — `ScaleEntryFields::entry` chooses on the radio,
not on whether a length exists.

It also seeds from the group's stored scale (O192), for the same
reason [`Self::open`] does and with one extra consequence: the unit the
group is already in becomes the unit the real-length field is read in,
so an operator calibrating a drawing that is already in inches types
`25 ft` and gets feet, rather than typing into a field that silently
meant metres.

⚠ **This is now the FALLBACK entry point, not the usual one.** The
ordinary route is [`Self::deliver_measured`] on a window that never
closed. This constructor is what runs when a pick completes with no
window waiting -- which this application cannot currently reach, and
which is kept rather than removed because removing it would make the
two-point gesture depend on a window's continued existence for its
result to be usable at all.

### `fn deliver_measured`

The pick completed; this is the answer coming home. Everything the
operator had already entered is still here, **because the window was
never closed** -- it was only [`hidden`](Self::hidden).

`use_real_length` is set because the operator has just done the work
that path exists for; the ratio entry stays available in the same
window, exactly as it does on the [`Self::calibrated`] path.

Nothing here un-hides. Un-hiding is what the caller does by disarming
the tool, and it is derived rather than done -- see [`Self::hidden`].

### `fn hidden`

# Hidden is DERIVED, and that distinction is the third defect


# Why a flag was written first, and then deleted

The obvious repair is an `awaiting_pick: bool` set when the button is
pressed and cleared on delivery, plus a once-a-frame invariant in
`app::frame` to clear it when the operator abandons the pick. That was
written, and then removed, because `canvas::placing`'s header already
contains the ruling against it **and names this window as the broken
precedent it was generalising away from**:

> With a stored `hidden: bool` this arm would inherit that, five times
> over: a mode change through `tool::arm::retire_forbidden`, the Tool
> panel putting the pen down, a ribbon control arming a different tool,
> the document closing, Escape. Every one is a route somebody has to
> remember to clear a flag on. With `hidden` derived, **stranding is
> unrepresentable**.

So it is derived. The window is hidden for exactly as long as the scale
pick is armed, and for no other reason. Every route that disarms the
tool -- Escape, a mode change, the Tool panel, a ribbon control, the
completed pick itself, and any route added next year by somebody who
has never read this file -- brings the window back with every entry
still in it, because there is no flag to forget.

The derivation is only sound because `MeasureKind::Scale` has exactly
**one** arming site in the crate: `app::frame`, on this window's own
button. That is not an accident anybody has to maintain by hand --
`canvas::measure`'s `every_variant_is_either_offered_or_deliberately_excluded`
is a wildcard-free `match` that classifies `Scale` as `"elsewhere"`
rather than `"ribbon"`, so a future ribbon control for it does not
compile until somebody moves it between the two lists and reads why.

[`crate::canvas::tool::selected`] and not `active`: `active`
resolves the space-bar's temporary Hand override, and an operator who
pans the page mid-pick must not have this window flash back over the
drawing they are panning to look at.

### `fn take_calibrate_request`

Consumed by the application, which arms `MeasureKind::Scale`. Read-and-
clear rather than a returned flag, so the caller cannot forget to reset
it and re-arm on every subsequent frame.

### `fn show`

# Screen-anchored, like every dialog here

A surface an operator is typing into must stay where they put their
eyes, and a position derived from the page moves on every zoom and
scroll. `default_pos` rather than `anchor` so it can be dragged aside —
this one sits over a drawing the operator may want to look at while
deciding what the scale is.

# The early return says `true`, and the `true` is load-bearing

[`Self::hidden`] returns **without drawing**: the window still exists,
it is simply out on the page while the operator points at a line. A
`false` here would destroy exactly the state this change exists to
preserve -- it is the old `close_scale()` behaviour, spelled one layer
further in.
