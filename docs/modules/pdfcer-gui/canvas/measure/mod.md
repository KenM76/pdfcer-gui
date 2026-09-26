# `canvas::measure` — the dimensioning tools

Phase 7. Placed under `canvas/` rather than at `tools/measure/`, following
the precedent this shell sets: [`crate::canvas::markup`] is the other
on-canvas
authoring tool, it lives here, and a measure tool is the same kind of thing
— a gesture that reads the page and raises an `Action`.

## The parts

| module | what it holds | egui? |
|---|---|---|
| [`pick`] | the pick state machines — linear, circular, two-line | **no** |
| [`circular`] | the radius/diameter tool: its pick, its **two endings**, its outlines | yes |
| [`scale`] | the scale-entry and dimension-group model — salvaged, **not yet reachable**; see the note on `MeasureKind` | **no** |
| [`state`] | the container built on tool entry, and what it discards | **no** |
| `draw` | what the tools show before a click commits, and the one projection every mark goes through | yes |
| this file | the canvas hosting: picks, overlay, disclosure | yes |

[`circular`] is the newest row and the only one that hosts a single tool.
**R2** forced it out of this file when the radius/diameter tool was armed
(1,617 lines), but the line count only says that *something* had to move;
that module's own header carries which subject was separable and why —
briefly, it is the only measure gesture that does not end itself, so the
machinery for *saying when it is over* has nothing corresponding to it in
the other two tools.

[`state`] is a third row rather than the two the salvage planned, and the
reason is **R2**: the old `measure_tool.rs` is 2,044 lines, and the planned
two-way split still leaves the pick half about twenty lines over the
1,500-line limit. It was cut once more at a seam the original had already
drawn for itself — a `// ---` banner separating the three tools' individual
pick machines from the single container that owns all of them — rather than
by shaving doc comments to fit a threshold, which is the incentive
`tools/gates/check-file-size.sh` says in its own header it exists to refuse.
That module's own docs carry the full argument.

**`pick`, `scale` and `state` never see an `egui` type**, and that is carried
across deliberately from the old shell's `measure_tool.rs`, whose header
makes the argument: every transition is unit-testable without a live frame.
It is also what let the whole file be salvaged rather than rewritten.

## This module owns no geometry

Every load-bearing computation is a call into the already-shipped
`pdfcer-core::dimension` / `pdfcer-core::vector` — the Taubin best-fit circle,
the axis-constrained projection, the measured length, the scale back-calc,
and `author_from_two_lines`. The rule the old shell stated and this one
keeps: **reuse, never reimplement**, so a dimension authored on the canvas
is byte-for-byte the one `pdfcer dimension-add` writes.

## Item notes

### `fn trace_pick`

A function rather than the `format!` written twice, because the circular arm
returns before the tail of [`click`] and a harness reading this channel must
not have to know which arm produced its line. Two spellings would drift on
the first field anyone added.

### `fn arming_another_measure_tool_discards_the_circle_fit`

`MeasureState::set_kind` owns the rule and has its own tests; this
asserts the *hosting* applies it, because `load` is what calls it and a
hosting that skipped the call would carry a fit set into the linear tool
— where it would sit invisible, unfinishable, and would reappear the
moment the operator came back.

### `fn every_variant_is_either_offered_or_deliberately_excluded`

`MeasureKind::ALL` stopped being exhaustive over the enum when
[`MeasureKind::Scale`] arrived — it is armed from the Set-scale dialog,
not from a ribbon control, so listing it there would fail
`every_measure_kind_has_a_registered_command` for a kind that correctly
has no command.

An inventory with a silent exception is how a future kind ships armed by
nothing, which is the exact failure `ALL` was written to prevent. So the
exhaustiveness is moved here and made a **compile-time** obligation: the
`match` below has no wildcard, so a new variant does not build until
somebody decides which list it belongs in.

The run-time half then checks the two lists are disjoint and complete,
so a kind cannot be quietly in both or in neither.
