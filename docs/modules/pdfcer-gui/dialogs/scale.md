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
