# `dialogs::scale` — what a dimension's number *means*

The Set-scale dialog. `measure.set_scale` was registered, drawn on
Measure ▸ Scale, and inert; `shell::commands::reach` recorded it as *"the
clearest statement of a missing arm in the crate"*, and the block on it was
never the model — it was this window.

## ★ Why this is the sharpest gap in the measure feature

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

**A cold-opened dialog offers the ratio path.** ★ This paragraph used to
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
