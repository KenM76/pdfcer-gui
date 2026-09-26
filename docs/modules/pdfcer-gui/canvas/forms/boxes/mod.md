# `canvas::forms::boxes` — where a form's widgets are, and what a click on
one would mean

The **pure half** of filling a form on the page. Everything here is a
function of the document and a rectangle; nothing here needs an
`egui::Ui`, a laid-out scroll area or a live pointer, and every rule this
surface is judged on is therefore something a unit test can hold rather
than something a running window has to be trusted to demonstrate.

That split is the seam `canvas/mod.rs` and `canvas/keys.rs` already draw
between themselves — one side is drivable by a headless `egui::Context`,
the other needs a window — applied one level down. It is a subject and not
a line count: [`super`] contains no decision at all, only the wiring that
spends the decisions below.

## "Redraw appearances" is only half the remedy for an undrawn field

[`super`]'s §5 reason 1 offers `RegenerateAppearances` to an operator whose
field draws nothing. `EditSession::regenerate_appearances` writes an `/AP`
for a text field only when that field holds a `/V`, so it does nothing at
all for an **empty** undrawn one. The remedy that always works is a fill:
`fill_text_field` writes the value and regenerates every widget's `/AP`, so
filling once in the panel makes the field clickable on the page from then
on. [`crate::text::forms::forms_canvas_undrawn_note`] is where the panel
says that to the operator.

Read [`super`]'s header first. It carries the whole argument — why this is
not a [`CanvasTool`] variant, why the panel is not replaced, what the
editor cannot promise, how input layers, why the hit test takes no
tolerance, and the four reasons a field is routed to the panel instead.
This file is where those four reasons are actually decided
([`classify`]), where the geometry is done
([`crate::canvas::mapping::annot_canvas_rect`], which serves annotation
selection too) and
where the hit test lives ([`hit`]).

## Item notes

### `const MIN_EDITOR`

A form field is whatever size its author made it, and at 25 % zoom a
perfectly ordinary 12 pt field is three pixels tall. An editor that small
is an editor nobody can read what they typed in, so the box is grown about
its own centre until it reaches this — which means it can overhang the
field it is editing.

That overhang is the deliberate half. The alternative is an editor that
sits exactly on a field the operator cannot see into, which trades a
visible, self-explaining imprecision for an invisible, silent one. It also
has an obvious operator-side remedy that needs no code: zoom in.

### `const EDITOR_TEXT_RATIO`

A glyph box is taller than its letters, and a font size equal to the box
height clips descenders. 0.62 is the ratio at which an ascender-plus-
descender line fits inside the box with the padding `egui` adds, measured
against the theme's own text style rather than derived.

### `const EDITOR_TEXT_RANGE`

The lower bound is legibility; the upper bound stops a full-page field —
a signature block, a comment box — from being typed into at 40 pt, which
reads as a bug rather than as fidelity.
