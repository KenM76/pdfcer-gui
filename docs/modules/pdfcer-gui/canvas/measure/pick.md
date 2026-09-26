# `canvas::measure::pick` — the measure tools' pick state machines

The **pure, GUI-free authoring-state logic** the three measure tools drive
on the canvas (decision 011 §2.3/§2.4): the pick state machines and the
circular fit-set. The scale-entry and dimension-group model lives in
[`super::scale`], the tool-entry container in [`super::state`].

Everything here is expressed over `pdfcer-core` types and never over egui,
so every transition is unit-tested without a live frame — the same
discipline that keeps [`crate::canvas`]/[`crate::viewer`] headlessly
testable while `main.rs` stays a thin, compile-and-launch-only shell.

## What this module owns vs. what the shipped engine owns (REUSE, never reimplement)

This module contains **zero** dimension geometry, Taubin math, scale
arithmetic, or storage. Every load-bearing computation is a call into the
already-shipped `pdfcer-core::dimension` / `pdfcer-core::vector`:

- [`constrained_second_point`] / [`measured_length`] — the H/V/aligned
  projection and the measured page-space length.
- [`fit_circle_taubin`] — the best-fit circle over a sample set.
- [`author_from_two_lines`] — the entire reading of a picked PAIR of
  lines: parallel-vs-angled, which of the four angles, whether the apex is
  virtual, whether the pair is collinear and must be refused.
- [`DimensionKind`] — the immutable geometry the GUI hands to
  `EditSession::add_dimension`, **byte-for-byte the same value the CLI's
  `dimension-add` builds** (`pdfcer` stores `Linear { a: *a, b: *b,
  constraint }` from its two raw `--points`, and `Circular { fit,
  show_diameter }` from `fit_circle_taubin(&pts)` — so this module stores
  the **raw** snapped picks, NOT the constrained projection, matching the
  CLI exactly; the constrained segment is a *display-only* preview,
  ui-spec §2.5). The equivalence tests pin this —
  [`tests::gui_linear_kind_equals_cli_linear_kind`] here, and
  `gui_circular_kind_equals_cli_circular_kind` in [`super::circpick`]:
  identical `DimensionKind` ⇒ identical `add_dimension` call ⇒
  identical additive `/Line`+`/Measure`+`/PieceInfo`+`/OCG` bytes (rule:
  same engine path).

The scale half of that list — [`pdfcer_core::dimension::preview_group_scale`]
and [`pdfcer_core::dimension::parse_length`] — is stated in full on
[`super::scale`], which is where the only callers of either now live.

## The three tools' state

Split across this file and its two siblings, but it is one model and reads
as one:

- [`LinearPick`] — the A→B two-click state machine (ui-spec §2.1),
  shared verbatim by [`super::scale::ScalePick`]'s reference line (§4.1).
- [`CircularPick`] — the tool's OWN object pick-set (ui-spec §3.1, NOT
  `canvas_selection`), live-refit on every toggle (§3.2), with the
  display-only radius/diameter toggle (§3.4).
- [`LinearPickMode`] + [`TwoLinePick`] — which geometry the linear tool's
  clicks target: two snapped POINTS, or two picked LINES that
  the engine reads into whichever ce dimension the geometry calls for.
- [`super::scale::ScalePick`] + [`super::scale::ScaleEntryFields`] — draw a
  reference line, then the two co-equal scale-entry paths (real-length
  recommended, ratio) that back-calc through `preview_group_scale` (§4).
- [`super::state::MeasureState`] — the container built on tool entry that
  holds all of the above plus the shared snap controls.

Everything is `pdfcer-gui`-internal; `cargo tree -p pdfcer-core` is
unaffected (this module is not in core), and it adds no dependency.

## Item notes

### `const ARC_PREVIEW_STEPS`

Twenty-four over the full turn is smooth at any zoom pdfcer offers, and an
angular ce dimension's wedge is a fraction of that — so the drawn arc is
visually smooth while staying a handful of segments.

### `fn linear_pick_needs_a_third_placing_click_then_resets`

SolidWorks dimensions in three steps, and the third is what says how
far off the drawing the dimension sits. Committing on the second click
would land every ce dimension on top of the geometry it measures, at a
zero standoff, to be dragged clear afterwards.

### `fn a_reference_line_pick_still_commits_on_the_second_click`

`ScalePick` reuses this state machine for a line that is never drawn as
a dimension, so asking where to place it would be ceremony with no
meaning. The opt-out is what keeps one state machine serving both.

### `fn the_override_survives_a_clear_like_the_other_tool_preferences`

The instinct is the opposite — it is an assertion about two specific
lines. What settles it is the friction the override exists to remove:
`linepick.rs` documents that without it the remedy would be changing a
global setting per dimension, *"which is how a setting becomes a thing
people fight"*. Resetting per pair recreates that at smaller scale for
anyone dimensioning a whole drawing out of a sloppy exporter, and it is
safe to persist because the verdict says "forced" before any Accept.

### `fn a_circular_preview_is_the_fitted_circle`

The assertion is on the **radius of every drawn point**, not on the
segment count: a count would be satisfied by twenty-four segments of
any shape at all, which is the trap of checking a relation rather than
a magnitude. A circle drawn at the wrong radius, or centred on the
origin instead of on the fit, fails here.
