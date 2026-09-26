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
