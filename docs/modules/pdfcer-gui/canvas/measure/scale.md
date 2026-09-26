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

## Item notes

### `fn in_display_unit`

# Why this exists: "Show dimensions in" was a dead control


The mechanism, in one paragraph. [`Self::entry`] builds a
[`ScaleEntry`], and on the **ratio** path that variant has no unit field
at all — it carries `paper`, `real` and `basis`, because a ratio is
unitless and the basis is what turns it into a length. `Self::unit` is
therefore **discarded on that path**, and
`preview_group_scale` returns `unit: basis`. So an operator who typed
`1 : 100` and chose *Metres* got a preview labelled in **inches** and a
group committed in inches, with nothing anywhere saying so. Both
controls sit at their defaults on every open from the ribbon, which is
why it survived: the two agree until somebody touches one.

# Why the conversion is here and not a request to the engine

It looked like one. `NumberFormat.unit` might have *converted* the value
or merely *labelled* it, and guessing between those is how a drawing
gets dimensions that are plausible and wrong. Reading the engine settles
it with no ambiguity left:

- `ScaleState::Calibrated { scale }` — `effective_scale` **ignores the
  unit it is passed** and returns `scale` unchanged.
- `format_measurement` computes `measured_points × effective_scale` and
  hands the result to `NumberFormat::format`, whose own parameter is
  named `value_in_top_unit`.

⇒ **`scale` carries the unit; `format.unit` only names and shapes it.**
Setting `format.unit` to metres while `scale` is in feet would label
feet as metres — confidently, at every dimension in the document. The
conversion has to happen to the *scale*, and the scale is ours to build.

So: no engine change, and the row that suspected one was wrong to.

# The arithmetic

`baseline_per_point(u)` is how many `u` there are in one PDF point at
1:1, so one `from` is `bpp(to) / bpp(from)` of a `to`. `scale` is
`from`-per-point; multiplying by that factor makes it `to`-per-point.

On the **real-length** path this is the identity, because `entry`
passes `self.unit` straight through and `raw.unit == self.unit`. It is
applied uniformly anyway rather than branching on the path: a branch
here would be a second place that has to know which variant carries a
unit, and that knowledge going stale is exactly how the defect arrived.

`ratio_label` is left alone. On the ratio path it is `1:100` — a
pure ratio, true in any unit. On the real-length path it is
`25 ft = 42.3 pt`, and that path is the identity, so it is never a
sentence about a unit that has been converted underneath it.

### `fn the_display_unit_converts_the_scale_rather_than_relabelling_it`

The strongest available assertion, and it is deliberately not *"the
preview says metres"*. A build that set `format.unit = Metre` and left
`scale` in inches would pass that, and would then label inches as metres
at every dimension in the document — the confidently-wrong outcome, and
worse than the silent dropdown it replaced.

So this asserts the **physical length is unchanged**: the same page
distance, committed under two different display units, must describe the
same real-world length. That can only hold if the conversion reached the
scale.

### `fn what_the_preview_shows_is_what_the_commit_stores`

Pinned because the defect this pair replaces was exactly a divergence
between what a surface said and what a verb did, and the cheapest way
for it to come back is a second conversion added to one of the two.

### `fn an_explicit_fraction_choice_is_what_commits`

Without this the display type was pinned to `Unit::default_format()`,
so a drawing dimensioned in inches always read `55.63"` and could never
read `55 5/8"` — the notation the drawing uses, and the notation the
scale field already ACCEPTS as input.

### `fn scale_ratio_path_needs_no_drawn_line`

⇒ A6 was not merely untested. It was **tested, and the test agreed with
it.** A value asserted from the implementation rather than from the
contract pins whatever the implementation did, including its defects,
and it does so with all the authority of a green suite. The number here
is now derived from what the operator asked for.

1:100 on an inch basis is `100/72` inch per point; the same scale in
metres is that times `0.0254`, i.e. `2.54/72`. Both spellings appear
below on purpose — the first is the engine's own arithmetic and the
second is the conversion, and writing them as one collapsed constant
would hide which half a future failure is about.

### `fn the_seeded_ratio_reads_one_to_one_hundred_for_a_group_stored_at_that_scale`

`preview_group_scale` documents `Ratio { paper: 1, real: 100, basis:
Inch }` as `100/72` inches per PDF point, and that doctest is in the
engine. So a group STORED at `100/72` in inches is, by that same
definition, `1:100` -- and the seed has to say so with no arithmetic of
this crate's own in the way.

This test exists because the round trip below cannot fail on a sign
error or a reciprocal: invert the formula wrongly in one direction and
apply it wrongly in the other, and seed -> preview still lands back on
the number it started from. Only a hand-written expected value can see
that, which is why this one is first and the round trip is second.

### `fn a_calibrated_group_round_trips_through_the_seed`

An operator who opens the Set-scale window on a calibrated group and
presses Accept without touching a control must write back the number
that was already there. Not a number a few parts in 10^16 away from it:
that is a document edit, with an undo step and a re-propagation of every
dimension's baked appearance stream, in response to the operator
changing nothing.

Swept across every unit the engine offers, so a unit whose
`baseline_per_point` is an awkward number cannot be the one that breaks
it unnoticed.

### `fn a_never_set_group_is_seeded_with_the_default_ratio_and_not_a_guess`

Pre-filling `1:100` over a group nobody has calibrated converts *"nobody
has said what this drawing is at"* into *"this drawing is at 1:100"*.
The engine keeps those distinguishable with a tri-state and a disclosure
string it owns; a seed that guessed would throw that away in the one
window where it matters most.

The assertion is that the fields are the DEFAULT ratio -- not that they
are any particular number -- because the default is what the window has
always shown for an uncalibrated group and this change must not move it.

### `fn a_one_to_one_group_seeds_as_one_to_one_in_its_own_unit`

`ScaleState::OneToOne` answers `baseline_per_point(unit)` from
`effective_scale`, and `Ratio { paper: 1, real: 1, basis: unit }`
back-calculates to exactly that -- so this arm is the one place the
inversion is a stated identity rather than a division, and it is worth
its own test for that reason: an inversion that is wrong everywhere
except where it is trivially right would still pass the round trip
above for `OneToOne` alone.

### `fn a_degenerate_stored_scale_seeds_the_default_instead_of_a_nan`

Unreachable through this application -- `commit` refuses a degenerate
entry and `preview_group_scale` refuses a non-finite one -- but reachable
through a hand-edited or truncated `/PieceInfo` sidecar, which is a file
pdfcer is expected to open rather than to trust. A NaN written into an
`egui::DragValue` is a control the operator cannot type their way out
of, so the seed declines and shows the default instead.

### `struct ScaleEntryFields`

- **Real length (recommended, default):** the operator typed the drawn
  reference line's real length + unit; back-calc `scale = real /
  drawn_pdf_length` — needs a drawn line, so it is offered only where one
  exists ([`ScalePick`]).
- **Direct ratio:** `paper : real` on a disclosed paper-unit basis;
  needs no drawn line, so it is the path the group panel uses to set a
  scale by typing alone (ui-spec §7.2 accessibility win).

### `fn sync_real_length`

Returns the parse error for display, or `None` when it parsed. Called
on every keystroke, so the operator sees what pdfcer understood while
they are still looking at the field rather than after committing.

# Why a failed parse leaves the previous value alone

Mid-typing, `55 5/` is not a length. Zeroing the value on every
intermediate keystroke would make the live scale preview flicker
through garbage, and — worse — would leave a *stale* preview looking
authoritative if the operator stopped typing at that moment. Instead
the last good value is held and the error is shown, so the preview and
the message never disagree about whether the input is usable.

# Why the unit dropdown moves only when the text names a unit

Typing `55 5/8"` says inches; the dropdown should follow, or the
operator has to say the same thing twice. Typing a bare `55.625` says
nothing about units, and moving the dropdown then would be the tool
second-guessing a choice the operator already made.
