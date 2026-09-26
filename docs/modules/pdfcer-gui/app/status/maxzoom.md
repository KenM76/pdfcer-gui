# `app::status::maxzoom` — the maximum-zoom popup, behind the zoom readout


> *"put the max zoom setting on the bar at the bottom."*

## Why the readout rather than a new control

The status bar already has a zoom readout — a fixed 46 pt label showing the
current percentage, with a tooltip explaining the ladder. It is the one
control on the bar that is *about* zoom and does nothing when clicked, and a
label that turns out to be a button is the standard shape here: the page box
two groups along is a `TextEdit` that looks like a readout for the same
reason.

Adding a separate control would have cost horizontal space on a bar whose
own module documents a fixed 30 pt height and a right-hand cluster that must
not move, to say something the readout is already the natural home for.

## It writes the preference itself, and the caller persists it

Same seam as [`super::filter`]: this mutates a `Copy` value, and
[`crate::app::frame`] compares before and after and writes the file when it
moved. The argument is in that module's header — a comparison at the call
site cannot be forgotten by a future row added here, where a dirty flag can.

## What it is not

Not a warning, not a confirmation, and not advice. The operator settled the
performance question in his own words — *"it is up to the user to determine
how much of a performance hit they want to take"* — so the popup states
where the crossover is and offers no opinion about it. See
[`crate::text::maxzoom`] for the copy and why it is that plain.

## Item notes

### `const PRESETS`

The top entry is [`MAX_MAX_ZOOM_PERCENT`] rather than a literal `1e12`, so
the label says what is actually stored — see
[`crate::text::maxzoom::preset`]'s test on why that distinction is kept.

### `fn every_preset_is_a_value_the_preference_accepts`

A preset the parser would clamp is a row that silently does something
other than what it says — the operator picks a billion and the file
records something else, with nothing reporting the substitution.
