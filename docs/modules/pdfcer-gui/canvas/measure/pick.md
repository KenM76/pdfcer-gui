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

### `struct LinearPick`

[`Self::first`] being `None` means "awaiting point A"; `Some(a)` means "A
is set, the next commit is point B." [`Self::commit_point`] on the second
pick returns the authored [`DimensionKind::Linear`] and resets, so the
tool is immediately ready for the next dimension (the operator draws a
run of dimensions without re-entering the tool).

**Raw-second-point storage (byte-equivalence, module docs):** the authored
`b` is the RAW snapped second pick, exactly as the CLI stores it; the
constraint is recorded alongside so `measured_length`/`author_dimension`
apply the H/V projection at value/appearance time. The on-canvas preview
segment ([`Self::preview_segment`]) is the *constrained* line — display
only, so "what you see is what's measured" (ui-spec §2.5) without diverging
the stored geometry from the CLI's.

### `enum LinearPickMode`

A real change in what a click MEANS: [`Self::Points`] resolves any snap
candidate anywhere on the page, while [`Self::TwoLines`] calls
[`pdfcer_core::vector::linepick::pick_line_in_page`] and requires landing on
straight, already-drawn geometry — refusing curves and misses rather than
inventing a point.

# Why this is a mode and not a fourth tool

Because the operator declares it explicitly, in a control that is visible
the whole time the tool is armed. That is the test the canvas already
applies to `MarkupKind`'s ten kinds: a click's meaning may vary by mode, so
long as it never turns on state the operator cannot see. The carried-kind
argument itself is on [`crate::canvas::tool::CanvasTool::Measure`].

Switching mode discards any in-progress pick first — free, because nothing
has committed.

### `struct TwoLinePick`

The operator's request, verbatim: *"dimensioning tool should
allow the selection of two lines. if those lines are parallel it makes a
linear dimension between them like SolidWorks would, if they are at an
angle it makes an angle dimension."*

# Nothing here classifies anything

This struct holds two picks and a checkbox. Every geometric question — are
these parallel, which of the four angles did the operator mean, is the apex
virtual, is the pair collinear — is answered by
[`pdfcer_core::dimension::author_from_two_lines`], which is the same
function `pdfcer`'s `dimension-add --kind two-lines` calls.

That is deliberate and load-bearing. A second classifier living at the
canvas is exactly how the two shells acquire the disagreement
`Settings::parallel_epsilon_degrees` was introduced to prevent: the setting
centralises the *threshold*, and duplicating the code that consumes it
would reintroduce the divergence one level above the value that stops it.
The GUI owes a gesture and a disclosure surface, not a second reading of
the geometry.

# Why this is NOT stored in `MeasureState::pending`

`pending` looks like the obvious home — it is already documented as "the
linear tool's completed-but-not-yet-authored dimension". It is the wrong
home, and the reason is decision 031's commit-on-interrupt path. A
completed *two-point* pick is safe to auto-commit when something else
interrupts the gesture, because nothing about it is inferred: it is exactly
what the operator clicked. That is why the rule may be expressed as a bare
"is `pending` populated?" test, and why it excludes the circular tool,
whose best fit is inferred.

A two-line verdict is inferred in precisely the circular sense:
parallel-vs-angled, which of four angles, whether the apex is virtual. Put
it in `pending` and a populated-field test — which cannot tell an ordinary
pick from an inference — would quietly make it interrupt-committable,
reopening for this gesture exactly the hazard decision 031 closed. A
sibling field keeps that rule correct without teaching it a new
distinction.

# Why the verdict is DERIVED on every read instead of cached

The ui-spec proposed a `verdict` field recomputed "whenever `second` or
`force_parallel` changes". This implementation deliberately has no such
field, and the reason is not a preference — it is that the proposed write
list was already incomplete by one when it was written. **The epsilon can
change too**: `Settings::parallel_epsilon_degrees` has a slider in the
settings panel, and moving it re-reads the same two lines into a different
answer (pinned by
[`tests::changing_the_epsilon_setting_re_reads_the_same_pair`]). A cache
listing two of its three producers is the failure mode recorded in
`D:\dev\rag\egui\a_derived_value_with_one_producer_cannot_drift_a_cached_copy_with_n_producers_will.md`,
and the fix it names is deleting the cache rather than hunting for the
missing reset site.

The recomputation is a few dozen floating-point operations on two stored
segments, so the cache buys nothing and costs a synchronisation obligation.
[`Self::authoring`] re-derives on every call: there is nothing to keep in
sync, and the verdict the operator reads is definitionally the one that
will commit (R85).

### `fn new`

Off is the honest default: the override is the operator asserting
something about the drawing that pdfcer's own reading disagrees with, so
it cannot be on until they say so.

### `fn offer_line`

The three cases, and why each is what it is:

- **Awaiting line A** — take it.
- **Awaiting line B** — take it; the pair is now complete and the
  verdict is on screen for review.
- **Pair complete** — depends on the verdict, which is why this needs
  `epsilon_degrees`:
  - a valid verdict ⇒ **ignored**, matching `pending`'s own documented
    "further picks are ignored, the operator is reviewing" rule. An
    inference under review must not be replaced by a stray click.
  - a REFUSED pair (collinear, or a degenerate line) ⇒ the new line
    **replaces line B**. That is the natural "try a different second
    line" recovery, and it needs no special case beyond reassigning
    `second`. Line A is kept, so recovering costs one click rather
    than two.

Returning `false` for the ignored case lets the caller leave the
disclosure untouched rather than re-announcing an unchanged verdict.

### `fn authoring`

`None` while the pair is incomplete. `Some(Err(..))` when the pair
cannot yield a ce dimension — collinear, or a zero-length line — which
the caller is expected to disclose by name rather than swallow.

`epsilon_degrees` comes from `Settings::parallel_epsilon_degrees` and is
never a literal at the call site, so this tool and the CLI cannot
disagree about when two lines count as parallel.

### `fn clear`

[`Self::force_parallel`] deliberately SURVIVES, mirroring how
`snap_master` and the active group survive
[`super::state::MeasureState::clear_gesture`] — see that field's own
docs for why.

### `fn dimension_preview_segments`

# Why this is a function and not two painter loops

Two places preview a ce dimension before it is committed: the two-line
authoring gesture (previewing what Accept would author) and the placement
drag (previewing where a release would put it). They must draw the SAME
shape for the same kind, or the operator sees one thing while authoring and
a different thing while adjusting it.

Keeping it here rather than in `main.rs` also makes it testable — the arc
decomposition in particular has a wrap-around case (a wedge crossing ±π)
that is easy to get wrong and invisible until an operator picks the two
arms that trigger it.

Returns page-space pairs; the caller supplies the projection to screen.

# Why the circular arm draws the fitted circle

Outlining the picked objects says **which objects are in the fit**; it
cannot say **what circle those objects imply**, and the circle is the
entire output of the tool. An operator looking at three outlined arcs has
no way to tell a fit that lands on their hole from one that has caught a
leader line and bulged — the residual is a number nobody sees until the
dimension is on the page.

The circle is drawn **from here** rather than from a loop in the canvas
hosting, for this function's own stated reason one paragraph up: the fit is
an *inference*, and a pre-commit affordance only means anything if what is
previewed is derived from what will be committed.
`canvas::measure::preview` hands this the identical `DimensionKind` that
`circular::commit` raises on the action — so the circle on screen and the
circle in the file are one derivation, not two that agree.
