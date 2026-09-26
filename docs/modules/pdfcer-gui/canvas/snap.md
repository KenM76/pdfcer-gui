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

## Item notes

### `fn the_snap_tolerance_converts_inversely_with_zoom`

The raw conversion lives in [`super::super::mapping`] and is tested
there, so re-asserting it here would be a duplicate. What is *not*
tested there and is tested here is the pairing — that the SNAP radius is
the one being converted, and that the degenerate contract survives the
wrapper.

### `fn the_snap_radius_is_looser_than_the_selection_radius`

Prose on both constants states the asymmetry; only an assertion enforces
it. A tuning pass that nudged one of the two numbers could silently
invert the relation, and the result would not look like a bug —
selection would just start grabbing neighbours while snapping got fussy,
which is the pair of symptoms the prose exists to prevent.

Both sides are constants, so this is a `const` block: the invariant is
checked when the test module is *compiled*, and an inversion fails the
build rather than one test run. Clippy insists on it
(`assertions_on_constants`) and clippy is right — but the test wrapper is
kept anyway, because a bare `const _: () = assert!(…)` at module scope
has no name, and this invariant is one a reader should be able to find by
running `cargo test canvas::snap` and reading the list.

### `fn every_snap_kind_has_a_non_empty_marker_and_the_derived_one_is_distinct`

# It consumes `SnapKind::all()` and never a hand-written list

A hand-written `let kinds = [SnapKind::Node, …, SnapKind::Axis];` under a
name promising **every** kind agrees with the real list on every day
except the one that matters, and on that day it does not go red — it goes
green over eight of nine while its name still claims completeness.

## What would otherwise protect it, and why that is not the same thing

`SnapKind` is **not** `#[non_exhaustive]` (see
`pdfcer-core/src/vector/snap.rs`), and [`snap_marker_shapes`] matches it
**exhaustively with no wildcard arm**. So a ninth variant upstream breaks
this crate's build at that match, and a stale array would be found while
fixing the error.

⇒ That is real protection. It is also **somebody else's**, and it is one
ordinary edit from being gone: the day a `_ => Vec::new()` arm is added to
`snap_marker_shapes` — a reasonable thing to write, and the exact thing
`info_label` does for `InfoField` — the compile error disappears, the new
kind silently draws nothing, and the one test whose job was to catch a
kind that renders nothing never looks at it. The two safeguards fail in
the same instant because they were never independent.

## ⇒ The general question, and it is not the one the name asks

Not only *"does this guard cover the property it is named for?"* but
*"is that coverage **its own**, or borrowed from the current shape of a
neighbouring function?"* — **a borrowed guard has no owner, so nobody is
told when it is returned.** The commit that adds a `_ =>` arm to
[`snap_marker_shapes`] is a commit *about* `snap_marker_shapes`; it has
no reason to read this test, no gate names the dependency, and nothing
anywhere goes red on the day the protection stops existing. A guard that
reads its own subject fails loudly the moment its subject changes, which
is the only difference that matters.

Consuming `SnapKind::all()` makes this test's coverage its own property
rather than a side effect of how the neighbouring function is written,
and it is why `tools/gates/check-completeness-tests.py` has no FOREIGN
row for this site. The accessor's own rationale is worth reading at
`SnapKind::all()`.

**What this still cannot catch** — a kind whose marker is
indistinguishable *on screen* from another kind's. Only the one pair below
is checked, and only by shape count. The general property needs a
rendered-pixel oracle, not a unit test.

### `fn the_indicator_and_committed_roles_are_the_pair_the_theme_defines`

Cheap, and it guards the exact failure the `Overlays` docs describe: an
unknown role is `None`, not an error, so a misspelled key draws nothing
and says nothing. Pinning the literals here means a rename in the theme
breaks a test rather than a frame.

### `fn an_uninstalled_overlay_set_yields_no_tint_rather_than_a_fallback`

A bare [`egui::Context`] has no role map, and the honest answer for a
role that is not defined is `None` rather than a substitute colour — see
[`snap_indicator_tint`]'s docs for what a `None` costs on screen. The
shipped binary installs a set in `crate::app::frame`; this asserts the
uninstalled path answers honestly rather than guessing.
