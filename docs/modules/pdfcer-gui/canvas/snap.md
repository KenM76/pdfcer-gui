# `canvas::snap` — the GUI half of snapping: the gates, the cycle, the glyph

## What this group of primitives is

The snap **maths** lives in `pdfcer_core::vector::snap` and is GUI-free: give
it a page, a query point and a page-space `SnapConfig::tolerance` and it
returns a priority-sorted list of [`SnapCandidate`]s. The engine deliberately
does **not** own two things, and those two things are this module:

1. **Zoom-invariance and the gates.** The engine takes a *page-space*
   tolerance; the operator experiences a *screen-space* catch radius. The
   conversion between them — and the decision of whether to run the query at
   all, given the persistent master toggle and the transient Alt override —
   is a GUI question about how the tool should *feel*.
2. **The fuzzy indicator.** Which glyph marks a candidate, how the operator
   cycles between competing candidates with Tab, and how many clicks it takes
   to commit one. That last is the *fuzzy-never-sneaky* rule made concrete:
   the one candidate kind that is an **inference** about operator intent
   ([`SnapKind::DerivedCenterline`]) confirms in two clicks; every kind that
   is a deterministic fact about geometry already on the page commits on one.

Everything here is a pure function over `bool`/`usize`/`&[..]`-shaped inputs
or a `Vec<Shape>` builder. Nothing reads global state, nothing mutates a
document, and every rule below is unit-tested without a window.

## What consumes it

The **measure tools**, in
[`crate::canvas::measure`](super::measure) (`canvas/measure/`). They own the
tool-mode frame the indicator draws in, so they are the ones who:

| per frame | calls |
|---|---|
| decide whether to query at all | [`snap_query_enabled`] |
| build the engine's `SnapConfig::tolerance` | [`snap_tolerance`] |
| pick which candidate is live | [`active_snap_candidate`] |
| advance the cycle on <kbd>Tab</kbd> | [`next_snap_index`] |
| decide whether a click commits or only proposes | [`snap_commit_clicks`] |
| paint the indicator | [`snap_marker_shapes`] |

The paint call is a one-liner in the measure-tool overlay handler:

```ignore
painter.extend(snap::snap_marker_shapes(screen_at, kind, tint, size));
painter.text(label_at, .., text::snap_indicator_label(kind), ..);
```

## The two conversions this module does NOT own

**Screen → page distance.** [`super::mapping::screen_tolerance_to_page`]
already exists in this shell and already carries the `1 / zoom` law with its
degenerate-input contract and its tests. This module does not re-derive it:
[`snap_tolerance`] is a one-line wrapper that pairs it with
[`SNAP_SCREEN_TOLERANCE_PX`]. `mapping`'s header states the invariant
plainly — *"there is no second place in `canvas/` that divides by `zoom`"* —
and a snap-local copy of the conversion would be exactly that second place.
What lives here instead is the snap-specific default and the test that pins
the snap radius's zoom-invariance.

**Screen → page position.** [`super::mapping::PageMapping::to_page`]. The
query point handed to `snap_candidates` is a page coordinate, and the
measure tools already hold a `PageMapping` for the frame.

## The tint comes from the caller, and which role it must be

[`snap_marker_shapes`] takes `color: Color32` rather than reaching for a
theme itself, for one reason: the marker is painted *inside* the measure tool's
overlay pass, which already knows whether it is drawing a live proposal or a
committed dimension, and a painter that resolved its own colour would have to
be told that anyway — as a second argument, in a second vocabulary.

What the caller must **not** do is invent the colour.
`crates/egui-shell/src/theme/overlays.rs` already defines the roles this work
needs, and [`SNAP_INDICATOR_ROLE`] / [`SNAP_COMMITTED_ROLE`] name them so a
call site cannot typo one into silence. [`snap_indicator_tint`] resolves the
first of them; see its docs for the honest `None` it can return and what that
`None` currently means in this shell.

## ⚠ The `#[allow(dead_code)]` attributes below are stale

Every item below carries `#[allow(dead_code, reason = …)]` describing itself
as waiting for a consumer. `canvas/measure/` is that consumer and it has
arrived: `resolve.rs` calls [`snap_query_enabled`] and
[`active_snap_candidate`], `mod.rs` calls [`next_snap_index`],
[`snap_commit_clicks`] and [`snap_marker_shapes`], `canvas::painting` calls
[`snap_marker_shapes`], and [`super::mapping::PageMapping::snap_tolerance`]
reads [`SNAP_SCREEN_TOLERANCE_PX`]. Only the free [`snap_tolerance`] has no
caller. Removing the six dead allows is a code change and is not one this
comment can make.

**A `reason` string is a live annotation, and pointing one at a milestone in
another repository is a claim nobody in this crate can check.** Each reason
therefore names the consumer in **this** shell.
