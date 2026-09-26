# `canvas::measure::scale` — scale entry, and the dimension-group actions

**Salvaged** from the old shell's `measure_tool.rs`
(`D:\Dev\pdfce\crates\pdfce-gui\src\measure_tool.rs`, Pass 12.M2b), split at
that file's own section banners — see [`super::pick`] for the pick state
machines and [`super::state`] for the tool-entry container. The reasoning
below is the original's, carried across intact.

The **pure, GUI-free** half of the scale-dimension tool: the reference-line
pick's dialog state, the two co-equal scale-entry paths and their back-calc
plumbing, and the group-panel action set. Like its siblings it is expressed
over `pdfcer-core` types and **never over egui**, so every transition is
unit-tested here without a live frame — the discipline that let the old
file be salvaged rather than rewritten.

## What this module owns vs. what the shipped engine owns (REUSE, never reimplement)

This module contains **zero** scale arithmetic, unit conversion, length
parsing or storage. It owns *which* engine call to make and *what the
operator typed*; the numbers come from the already-shipped
`pdfcer-core::dimension`:

- [`preview_group_scale`] (12.M2) — the scale back-calc for **both** entry
  paths. [`ScaleEntryFields::preview`] chooses the [`ScaleEntry`] variant
  and hands it over; it never divides anything itself. This is what makes a
  canvas-calibrated group and a CLI-calibrated group the same number.
- [`parse_length`] — `55 5/8"`, `4'-7 1/2"`. The scale field is a TEXT field
  precisely so the operator can type the dimension the way the drawing
  writes it, and the grammar for that lives in core, once.
- [`ScaleState`] / [`NumberFormat`] / [`Unit`] / [`FractionMode`] — the
  stored tri-state and display model handed to
  `EditSession::set_group_scale`. Constructed here, defined there.
- [`GroupId`] / [`DimStandard`] — the identities the group verbs name. Each
  is a **mapping onto exactly one shipped `EditSession` command**, which is
  what keeps "I made a group" and "I hid a layer" one `Ctrl+Z` each
  (ui-spec §5.4). They travel as
  `crate::app::actions::dimensions::DimensionAction` — this module builds
  the *values* they carry and never raises one itself.

The pick half of that list — `constrained_second_point`, `measured_length`,
`fit_circle_taubin`, `author_from_two_lines` and the `DimensionKind`
byte-equivalence argument — is stated in full on [`super::pick`], which is
where the only callers of those live.

## The three tools' state

Split across this file and its two siblings, but it is one model and reads
as one:

- [`super::pick::LinearPick`] — the A→B two-click state machine (ui-spec
  §2.1), shared **verbatim** by [`ScalePick`]'s reference line (§4.1). Not a
  copy and not a parallel implementation: [`ScalePick::line`] *is* a
  `LinearPick`, constructed by [`super::pick::LinearPick::reference_line`],
  which is the same machine with its third placing click switched off.
- [`super::pick::CircularPick`] — the tool's OWN object pick-set (ui-spec
  §3.1), live-refit on every toggle (§3.2).
- [`ScalePick`] + [`ScaleEntryFields`] — draw a reference line, then the
  two co-equal scale-entry paths (real-length recommended, ratio) that
  back-calc through [`preview_group_scale`] (§4).
- [`super::state::MeasureState`] — the container built on tool entry that
  holds all of the above.

Everything is `pdfcer-gui`-internal; `cargo tree -p pdfcer-core` is
unaffected (this module is not in core), and it adds no dependency.

## Adaptations made on the way across

**None that change behaviour.** No arithmetic, no transition and no engine
call was touched; the only edits are module paths for the types that now
live in [`super::pick`] and [`super::state`]. The one thing worth stating
is what was *checked*: every `pdfcer-core` item named above still exists at
the engine HEAD this workspace builds against, with the same signature, so
nothing here is an adaptation to a moved API.
