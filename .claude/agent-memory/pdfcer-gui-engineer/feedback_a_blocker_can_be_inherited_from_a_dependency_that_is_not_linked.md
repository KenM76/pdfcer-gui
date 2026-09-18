---
name: a-blocker-can-be-inherited-from-a-dependency-that-is-not-linked
description: Before believing a documented blocker, ask which library's property it turns on — one blocked a feature for months on egui_tiles, which this shell never links
metadata:
  type: feedback
---

**A documented blocker is a claim about a mechanism. Find the mechanism and ask
whose it is — a verdict reached while evaluating a dependency survives the
decision not to adopt that dependency.**

**Why:** `MODES_AND_PANELS.md` capability (d), its "What is left to build",
`PROJECT_PLAN.md` §4.2 and `dock/mod.rs`'s header all said cross-compartment
panel dragging needed one wide tree spanning left ▸ canvas ▸ right, that the
wide tree puts the canvas in a resizable pane, that a pane around the canvas
fires the R128 fit-zoom loop, and that the fit-zoom cache therefore had to land
first. Four documents, one chain, and every link sound except the first: *drag
identity is tree-scoped* is a property of `egui_tiles`, where two docks are two
`Tree`s. This shell never links it — `Cargo.lock` has no such crate, and
`UI_TOOLKIT_PINS.md` already carried the general warning verbatim. One
`DockState` owns both sides here, so the blocker was never real and the feature
sat behind an unrelated rendering landing.

**How to apply:** when a document says feature X is blocked on landing Y, do not
schedule around it — name the mechanism in one sentence and check it against
*this* codebase's `Cargo.lock`. The tell is a blocker phrased in a library's
vocabulary (tile, tree, arena, handle, scene) when the corresponding crate is
pinned-but-unused or absent. A refusal written while evaluating a dependency
gets re-read later as a property of the problem. Correct every instance in one
sitting — this one had four, and prose citing prose is how a phantom survives.

Related: [[feedback_a_limitation_sentence_is_a_citation_with_an_hours_long_shelf_life]],
[[feedback_a_backlog_row_is_a_record_not_evidence]],
[[feedback_a_measured_limit_belongs_to_a_revision_not_a_design]].
