# `canvas::measure::state` — the container built on tool entry

**Salvaged** from the old shell's `measure_tool.rs`
(`D:\Dev\pdfce\crates\pdfce-gui\src\measure_tool.rs`, Pass 12.M2b), from
that file's own last section banner: *"The container tool state, built on
tool entry"*.

## Why this is a third file and not the bottom of [`super::pick`]

The old file was 2,044 lines, which breaks this project's **R2** limit
(no `.rs` file over 1,500 lines) by a wide margin, so it had to be split.
Two files were planned — the pick machines and the scale model — and that
split leaves the pick half at roughly 1,520 lines: over, by about twenty.

Rather than shave prose to fit a threshold — which is precisely the
incentive `tools/gates/check-file-size.sh` says in its own header it refuses
to build in — the file was cut once more, at a seam the original had
**already drawn for itself**: a `// ---` banner separating the three tools'
individual pick machines from the single container that owns all of them.
That is the seam, and it is a real one in the sense R2 asks for: the two
sides answer different questions. [`super::pick`] and [`super::scale`] each
answer *"what does one tool do with a click?"*; this file answers *"what
does the measure tool as a whole remember between clicks, and what is
discarded when?"* — page staleness, the active group, the snap master
toggle, the two-click confirm for a derived candidate, the one-frame queues
the Tool Options pane writes.

Like its two siblings it never sees an `egui` type, so every transition here
is unit-tested without a live frame.

## This module owns no geometry either

It owns *composition* and *lifetime*: which pick machine exists, which page
it targets, and when each is thrown away. Every number still comes from
`pdfcer-core` by way of [`super::pick`] / [`super::scale`], and the one
engine value named directly is [`pdfcer_core::dimension::DEFAULT_GROUP_ID`] —
the always-present default group a fresh state seeds itself with, which is
core's constant rather than a literal `0` written here.

## Adaptations made on the way across

1. **`GestureInterrupt` is prose.** The old shell had a
   `crate::canvas::GestureInterrupt` enum that the doc comments linked to;
   `grep GestureInterrupt` over this crate returns zero hits. The concept —
   a mid-gesture state safe to throw away — is what those comments are
   about, and it survives as words.
2. **`CanvasTool::MeasureLinear` and friends are prose.** This shell's
   [`crate::canvas::CanvasTool`] has `Select`, `Hand` and `Markup`; the
   three measure variants land with the canvas hosting in
   [`super`], not here.
3. **Nothing computational changed.** No transition, no field, no default
   was altered.

## Item notes

### `fn changing_kind_discards_a_pick_in_progress`

The failure it prevents is the one the original's docs warn about in
their own words: a carried-over pick does not raise an error, it
produces *"something strange"* on the operator's **next** click, which
is the worst place to discover it.

### `fn every_pick_kind_is_counted_as_a_gesture`

# Why this is shaped as one sub-test per FIELD

Because the thing that goes wrong is a field being added and a
disjunction not being extended, and no enum exhaustiveness check can see
that — `MeasureKind` was updated correctly in five places while this one
disjunction silently kept its old shape. The only way to catch it is to
drive each machine into a started state and assert the container
notices.

A machine added without a line here fails nothing, which is the honest
limit of this test. What it does buy is that the *existing* five cannot
regress, and that a reader adding a sixth finds a list with an obvious
hole in it rather than a boolean expression to audit.
