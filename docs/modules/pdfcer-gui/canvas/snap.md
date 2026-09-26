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

### `const SNAP_SCREEN_TOLERANCE_PX`

Deliberately a *sibling* of [`super::mapping::SELECT_SCREEN_TOLERANCE_PX`]
rather than the same constant, and deliberately **looser** than it: snapping
and selection answer different questions and are allowed to drift apart. A
snap that grabs a nearby vertex is a helpful correction the operator can see
and cycle through with <kbd>Tab</kbd>; a selection that grabs a neighbouring
object is a silent wrong answer. The failure modes are not symmetric, so the
tolerances are not either. (That asymmetry is stated from the selection side
in `mapping`'s own docs; [`tests::the_snap_radius_is_looser_than_the_selection_radius`]
pins the direction so a future tuning pass cannot invert it by accident.)

### `const SNAP_INDICATOR_ROLE`

A snap marker is drawn while the operator is still aiming, before any click
has committed anything, so it is a proposal by definition. Naming the role in
a constant rather than spelling it at each call site is not ceremony:
[`egui_shell::theme::Overlays::get`] returns `Option` for an unknown role, so
a typo does not fail — it draws nothing, on whichever preset the typo was
written under.

### `const SNAP_COMMITTED_ROLE`

Not used by [`snap_marker_shapes`] — a snap marker is never committed state —
but named here beside its partner because the two form the **preview-vs-
committed pair** that `overlays.rs` exists to keep distinct:

> the measurement preview and the committed dimension differ because one is
> a proposal and one is document state […] a theme that merges two roles
> removes a cue that was doing work, and it would do so silently.

The measure hosting owes `Overlays::assert_distinct(&[…])` over both, once,
per preset — that is the test `overlays.rs` says the application owes and the
shell cannot write for it.

### `fn snap_tolerance`

A constant on-screen catch radius maps to a *shrinking* page-space tolerance
as the operator zooms in, so the "feel" stays constant. The `1 / zoom`
distance law itself, and its contract that a non-finite or non-positive
`zoom` yields `0.0` (snapping disabled) rather than a NaN/∞ tolerance the
engine would reject anyway, both live in
[`super::mapping::screen_tolerance_to_page`] and are **not** re-implemented
here — see this module's header on why a second divider by `zoom` in
`canvas/` would be a defect.

# Why this takes a bare `zoom` and not a [`super::mapping::PageMapping`]

`PageMapping` has no `zoom()` accessor, on purpose: its docs record that
*"the zoom's whole job here is to be divided by, and exposing it would be an
invitation to divide by it at a call site"*, and its one tolerance method
[`super::mapping::PageMapping::tolerance`] is the **selection** radius. So
there are two honest options and this is the smaller one — a caller that has
the frame's `ViewState::zoom` passes it. If the measure hosting turns out to
hold only a `PageMapping`, the right fix is a `snap_tolerance()` method on
`PageMapping` beside its selection sibling, which belongs to `mapping`'s
owner rather than here; this function then becomes its body.

### `fn snap_marker_shapes`

# The tint is an argument, and it must be a named role

`color` is supplied by the caller. The caller is the measure tool's overlay
pass, and the colour it must supply for a
pre-commit indicator is the `"preview"` role: [`SNAP_INDICATOR_ROLE`], via
[`snap_indicator_tint`]. Nothing in this function chooses a colour, which is
why `tools/gates/check-theme-colors.sh` has nothing to say about it.

The one `Color32::TRANSPARENT` below is the *absence* of a fill on an
outline-only polygon, not a choice of colour — which is precisely why the
gate's pattern deliberately excludes it.

### `fn snap_indicator_tint`

# Why this returns `Option` and does not fall back

[`egui_shell::theme::Overlays::get`] returns `Option` on purpose, and its
docs say why: *"a missing role is a programming error — a typo, or a role the
preset forgot — and returning magenta or transparent would make it a
rendering question the reader has to notice, on the frame where it happens,
on the preset where it happens."* Substituting a fallback here would undo
that, one layer further from the palette.

# The `Option` is load-bearing, and a `None` here is silent on screen

A context with no installed role map answers `None` for every role, and the
snap marker then falls back to the selection stroke — the exact shape of
failure an `Option` makes invisible, because nothing looks broken and the
cue is simply not there.

`crate::canvas::overlays::install` runs beside `Theme::apply` in
`crate::app::frame`, which is what makes the role resolve in the shipped
binary. The `Option` stays, and stays meaningful: it is the honest answer
for a role a preset forgets to define, and `overlays`' own test asserts that
none of the roles this canvas reads is one of them, on every preset rather
than on the default.
