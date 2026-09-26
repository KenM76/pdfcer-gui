# `egui-shell/ribbon/plan/mod`

## Item notes

### `const PACK_SLACK`

[`wrap_group`] builds its candidate widths by summing a slice and packs
by accumulating item by item. The two orders of addition are equal in
exact arithmetic and can differ by an ULP or two in `f32`, which would
make the intended candidate look infeasible by a hair and push the
answer onto the next-widest one. A thousandth of a point is far below a
physical pixel and far above the rounding.

### `fn pack`

An item wider than `target` on its own still gets placed — on a row by
itself, over budget — because refusing it would be refusing to draw a
control.

### `fn a_group_is_as_wide_as_its_caption_when_the_caption_is_wider`

`RIBBON_IA.md` §5 is full of two-word captions over one-glyph
controls — "Page display" over a single icon button — so this is
the common case, not the corner. Planning such a group at its
control's width would overflow the band by the difference, every
time, and the symptom would be a clipped caption rather than
anything that looks like a layout bug.

### `fn a_group_that_asks_for_rows_wraps_when_it_would_otherwise_fit`

`OPERATOR_REQUESTS.md` O97 — four square icon buttons that are a strip in
one row and a control in a 2 x 2 block. Without the hint the planner
short-circuits on *"it fits"* and never looks for a narrower shape.

The four widths are comfortably inside [`GROUP_WRAP_WIDTH`], so the
`None` case is the regression guard as much as the `Some` case is the
feature: **if the default ever starts wrapping, every group in every
application changes shape at once.**

### `fn the_row_hint_is_a_preference_and_the_bands_ceiling_still_wins`

Three properties, and each is a way the feature could have been built
wrong:

* asking for **one** row changes nothing — that is already the default,
  and a `Some(1)` that started wrapping would be a footgun in a manifest;
* the **band's ceiling wins** — a group asking for four rows in a
  two-row band gets two, because the band's height is fixed (R128) and a
  manifest must not be able to break it;
* asking does not FORCE the count — the planner still returns the
  narrowest packing, so two items whose one-row form is narrowest keep it.

### `fn a_group_that_fits_the_cap_stays_on_one_row`

The half of the mockup's rule that is easy to lose: `max-width` is a
*trigger*. Wrapping every group would turn a two-control group into
a column of one control per row, which is narrower and is not a
ribbon. Most groups in a real manifest are under the cap, so this is
the common path and not the corner.

### `fn a_group_over_the_cap_is_split_evenly_and_costs_less`

The evenness is asserted as a bound rather than as an exact split,
because the optimum depends on the item widths — seven items of 75
divide 4/3, seven of 20 with one of 300 do not divide at all. The
bound that always holds is *"the widest row is no wider than it has
to be"*, and the strongest cheap statement of that is: no row may be
wider than the widest single item plus the balanced share.

### `fn the_split_is_a_contiguous_partition_of_every_item`

The manifest's order is the operator's order — the same rule
[`BandPlan::shown`]'s prefix property exists for. A wrap that
reordered items to pack them better would rearrange the ribbon as
the theme's gutter changed, which is exactly the class of surprise
this module refuses elsewhere.

### `fn one_item_and_one_row_are_both_left_alone`

`max_rows == 1` is the old, one-row band expressed in the new
arithmetic. Keeping it reachable is what makes "two rows" a decision
this module records rather than a behaviour it merely has.

### `fn one_oversized_control_still_lets_the_rest_wrap`

The interesting shape, because the naive answer — "this cannot be
split, give up" — would leave a group at the sum of every item when
one oversized control is present, and oversized controls are exactly
what a long label produces.

### `fn a_band_that_fits_reserves_nothing`

The second half matters — a reservation that persisted when
nothing was hidden would be a permanent tax on every band that
fits.

### `fn the_overflow_affordance_survives_a_band_too_narrow_for_any_group`

This is the invariant the whole module exists for. The observed
defect it guards against is `MODES_AND_PANELS.md` Part 2, #8: past
a certain count the overflow button *itself* gets hidden, leaving
no route to what it was hiding. The plan must degrade to "no
groups, one working affordance", never to "some clipped groups, no
affordance".

Checked at a series of widths on the way down, because the
interesting failure is not at zero — it is at the width where a naive
implementation still has *just* enough room for a group and
therefore spends the overflow control's space on it.

### `fn the_overflow_affordance_is_reserved_exactly_when_it_is_needed`

A biconditional rather than one implication, because both
directions are real defects. Reserving with nothing hidden wastes
band width forever; hiding with nothing reserved is #8 itself.

### `fn widening_the_band_never_hides_a_group_that_was_visible`

Monotonicity is what makes a window resize feel like a resize.
A greedy fill has it by construction; a later "pack the widest
first" optimisation would not, and this test is what would refuse
that change.

### `fn the_visible_groups_are_a_prefix_and_nothing_is_lost`

The prefix property is what stops the visible ordering of the
ribbon from depending on the window width — the manifest's order
is the operator's order, and a plan that dropped a middle group to
fit a later narrow one would rearrange the ribbon as the window
moved.

### `fn a_non_finite_width_degrades_safely`

`egui` hands out `f32::INFINITY` for available width inside an
unbounded container. `INFINITY - overflow_width` is still
infinity, so an unguarded implementation would conclude that
everything fits and emit no affordance — the #8 defect arriving
through arithmetic rather than through ordering.

### `fn the_reservation_covers_the_widest_label_by_width_not_by_digit_count`

The trap this pins is the one real text springs and zero-width
text cannot: in a face whose digits are not tabular, `"⏷ 8 more"`
can be wider than `"⏷ 9 more"` even though the counts and the
lengths say otherwise. Reserving for `N = total_groups` alone —
which reads as obviously sufficient — then draws a control wider than
the space held for it, and the affordance overhangs the band's right
edge.

The `measure` below is deliberately perverse about exactly that:
every character costs 7 pt except `8`, which costs 40. A
reservation that consults only the largest count fails here; one
that takes the maximum over every reachable label passes.

### `fn the_overflow_label_uses_the_pinned_chevron`

Whether the character is *drawable* is asserted in the consuming
application (`pdfcer_gui::shell`'s
`the_ribbon_overflow_chevron_has_a_glyph`), because `cargo test -p
egui-shell` compiles without egui's `default_fonts` — a `has_glyph`
call here would answer about a font set no real build has, and would
pass for the whole life of a defect.

So this test does the half it *can* do honestly: pin the codepoint,
so that changing it is a deliberate act which fails a named test and
sends the next reader to the other half. Both docks and both rows
are covered, because [`crate::dock::plan::overflow_label`] promises
to stay identical and identical wording means identical codepoints.

### `const MIN_ITEM_WIDTH`

A control narrower than its own height reads as a clipping artefact
rather than as a button. This is also the floor that keeps the
arithmetic meaningful with no fonts installed — see the module header.

### `const CUSTOM_ITEM_WIDTH`

The shell does not draw custom items and cannot measure them: the
application is handed a `Ui` and draws a colour swatch, a zoom slider
or a gallery into it. Four control-widths is a generous guess at a
compound control.

Being wrong here costs a clipped group, never a lost overflow
affordance — see the module header on why that asymmetry is the point.

### `const GROUP_PADDING`

The mockup's `.group { padding: 0 13px }`, **both budgeted here and
drawn** by [`super::band::captioned_group`].

# Budgeted and drawn must stay together

Whether a ribbon group has internal padding is a design decision, not an
arithmetic one, and it has exactly two consistent answers: draw it and
budget it, or set this constant to zero. Splitting them is what hurts.

Budgeted-and-not-drawn adds 12 pt to every group's planned width that the
renderer never uses, and the symptom is not a layout fault — it is that a
group's first control sits flush against the group boundary and against
the rule separating it from the next group, which is most of what the
operator meant by *"cluttered"*. Drawn-and-not-budgeted is the opposite
error and clips the last group in the band.

Drawing it is the cheaper of the two answers because the plan reserves
the space either way: **drawing it costs zero additional width budget**,
so no group moves into the overflow menu that was not already there. The
band spends on padding exactly what it would otherwise spend on nothing.

# 6 pt here against the mockup's 13 px, and why they agree anyway

The two numbers look like a disagreement and are not, because the mockup
and this build decompose the space *between* two groups differently:

| | mockup | this build |
|---|---|---|
| group's own inset | 13 px each side | `GROUP_PADDING` = 6 pt each side |
| the rule between them | `border-right: 1px` — **costs no width** | `ui.separator()` — [`super::measure::separator_width`] = 6 pt of line plus one `item_spacing.x` each side |
| total, group edge to group edge | 13 + 0 + 13 = **26 px** | 6 + 14 + 6 = **26 pt** at the quiet preset |

So the inter-group breathing room already matches the specification to the
point; what was missing was only that none of it was on the *inside* of
the boundary. Raising this constant to 13 would make that gap 40 pt, which
is not what the mockup shows — and it would cost 14 pt of planned width
**per group**, which across a full tab's band groups is enough to push a
group into the overflow menu. Overflow is a measured
property of this band (see `super::width_tests` and the counts recorded in
[`super::band`]'s header), so that is a regression, not a refinement.

The one place the two genuinely differ is the band's outer edge, where the
mockup adds its own `padding: … 10px` before the first group's 13. This
build's band has no horizontal padding of its own, so its first control
sits `GROUP_PADDING` from the band edge rather than 23 px. That is a
separate decision about the *band*, not about a group, and it is not taken
here.

### `const GROUP_ROWS`

# Why this is a constant and not a parameter

`PROJECT_PLAN.md`'s **R128** is this project's most-repeated bug: a
*content-driven* height next to a fit-to-viewport zoom is a feedback
loop, measured at 230 % → 224 % → 215 % zoom drift from a status line
that grew by one row. The ribbon sits in a top panel above the canvas,
so the band's height is exactly such a term — and a band that were one
row tall on File and two on Markup would re-fit the page on **every tab
click**.

A constant makes *"the band is the same height on every tab"* true by
construction rather than by an invariant somebody has to enforce. Every
alternative spelling reintroduces the loop or the enforcement:

| Spelling | What breaks |
|---|---|
| per-**group** row count | the tallest group on a tab decides the band, so the band is content-driven again — R128 exactly |
| per-**tab** row count | the manifest can now say "File is one row, Markup is two", which is the drift stated as a feature |
| per-**manifest** / theme field | safe for R128, but it is a knob with one sane value that every consumer must set identically, and the first one that does not gets the drift |

A [`crate::theme::Preset`] change *does* alter the band's height,
through [`crate::theme::Metrics::control_height`], and that is fine and
unavoidable: it is a deliberate, global, one-off event, not something
that happens when the operator clicks a tab.

# Why THREE

Where `mockups/ribbon.html` — the ribbon study — and
`mockups/pdfcer-shell.html` — the whole-shell mockup the operator approved
under O123 — disagree, the approved one wins. This is a place they
disagree, and the approved mockup's own arithmetic says what the band's
68 pt is made of: `.rb { height: 22px }` and `.grp .col { gap: 1px }` give
`3 × 22 + 2 × 1 = 68`. A band 68 pt tall holding two rows of controls is
not that band; it is a two-row band with twelve points of air in it, and
the operator's word for the result was:

> *"it looks like the edits to the ribbon got halfway done… the mockup GUI
> pdfcer-shell.html looks much cleaner and we should follow it's format
> exactly"*

Measured against the mock at 1400 px, `View` alone: the mock lays Zoom
2 × 3, Display 2 × 3, Panels 3 × 3 and Window 2 × 3, filling 1379 px of a
1400 px window. At two rows those four groups need roughly 460 px more than
they are given, so groups the mock shows on the band collapse or scroll at
that width.

The two arguments that sound like grounds for two rows do not hold here:

1. **Screen budget: the third row is FREE.** A third row costs another
   `control_height + gutter` only while the band's height is *defined by*
   its row count. It is not: the band's height is
   [`crate::theme::Metrics::ribbon_rows`], a fixed budget the rows are laid
   into, so the third row costs **nothing at all** — 68 pt either way, with
   the controls coming down from 24 pt to 21.67 to suit. See
   [`super::rhythm::band_row_height`]. Failure mode #4 is untouched.
2. **The caption does not drift from its rows.** In the mock the third row
   ends at y = 138 and the caption sits at y = 138.9: the caption hangs off
   the bottom of the same fixed area whether the group used one row or
   three, which is the baseline invariant
   `height_tests::every_caption_in_a_band_shares_one_baseline` asserts.

Three is also what the product class does. Word's ribbon lays small
buttons three rows deep at every width; so does Acrobat's. The convergence
of the class is the specification.

## The number is in scope for a redesign; its scope is not

This is a **constant**, and every reason the table above gives for refusing
a per-group, per-tab or per-theme row count holds whatever the number is.

### `const MAX_GROUP_ROWS`

Three, from Word: its Font group is two rows at 1900 pt and **three** at
1000 pt. It never goes to four at any width in the series
`tools/word-ribbon-study.ps1` photographs, including 460, where it
collapses instead.

Why the ceiling is not simply "as many as it takes": the band's HEIGHT is
fixed and must stay fixed — R128, and the reason [`GROUP_ROWS`] is a
constant at all. A fourth row would either grow
the band, which moves the canvas under a fit-to-page zoom, or shrink the
controls below the size at which their icons are legible. Word reached the
same answer, and reaching it independently is worth more than copying it.

Note this is a **ceiling, not a target**. `wrap_group` searches for the
narrowest packing that fits within the row limit it is given, so a group
that reaches its narrowest at two rows stays at two even when three are
permitted, and `collapse::Candidate::gains_from` then declines to spend a
rung of the ladder on it.

### `struct GroupRows`

Produced by [`wrap_group`] and consumed **twice**: once by
[`super::band::measure_group`], which needs the width the wrapped group
will occupy, and once by [`super::band::captioned_group`], which needs
the same split in order to draw it. Passing the split rather than
recomputing it at the second site is the point — two greedy fills that
agreed *by construction today* would be a plan and a renderer that
disagree about the band's width the first time either is edited, and
that disagreement is a clipped group.

### `fn row_width`

`pub(crate)` because [`super::band`]'s measurement and this module's
wrap have to agree on what a row costs down to the last gutter; two
spellings of the same sum is how a plan and a renderer drift apart.

### `fn wrap_group`

# The rule, in two sentences

A group whose items fit within `wrap_at` on one row is left on one row.
Otherwise it is split into contiguous runs, at most `max_rows` of them,
chosen to make the **widest** run as narrow as possible.

# Why "minimise the widest row" is the right objective

Because the widest row is what the group *costs*: [`group_width`] takes
the maximum, [`plan_band`] spends the band's budget on it, and a group
that costs less is a group the band can fit beside another one. The
mockup's greedy fill optimises nothing — it fills the cap and lets the
remainder fall where it falls — so a seven-control group 548 pt wide
becomes 440 + 100 under the mockup's rule and about 280 + 270 under this
one. Both are two rows; only one of them gets a fifth group into a
1,100 pt window.

# The algorithm, and why it is exhaustive rather than clever

The optimum's widest row is, necessarily, the width of *some contiguous
run of items* — it is one of the rows. So the candidate set is every
contiguous run, `n(n+1)/2` of them, and the answer is the narrowest
candidate that a greedy fill can meet within `max_rows`. Greedy is
monotone in the target width (a wider row never needs more rows), so the
first feasible candidate in ascending order is the optimum.

A ribbon group holds single digits of items, so `n²` candidates is at
most a few
dozen f32 sums per group per frame, against a measurement pass that has
already asked `egui` for a galley per label. A binary search over a real
interval would be *less* exact for no measurable saving, and "exhaustive
over the runs" is a sentence a reader can check against the code.

# Degenerate inputs, all of which are reachable

- **Empty group.** `counts` is empty and `width` is zero; a manifest may
  legally declare a group with no items (a layer patching a caption, for
  instance), and it must not produce a row of nothing.
- **One item.** Never split — a column of one control per row is not a
  ribbon group, it is a rendering fault that happens to be deliberate.
- **`max_rows <= 1`.** Wrapping disabled; one row, whatever its width.
  This is the escape hatch that keeps the *old* behaviour expressible
  and testable rather than merely deleted.
- **An item wider than `wrap_at` all by itself.** It takes a row of its
  own and that row is wider than the cap. There is no other answer: the
  shell cannot make a control narrower, and clipping it would hide a
  command's name. The cap is a wrap trigger, never a clip.
- **A non-finite or negative `wrap_at`.** Treated as "never wrap", which
  is the same safe direction [`plan_band`] takes for a non-finite width:
  the group is as wide as it says it is and the band's overflow menu —
  whose reservation is taken first — still reaches everything.

### `fn group_width`

The caption is part of the *width*, not only of the height, and that
is deliberate. A group whose caption is wider than its single control
— "Page display" over one icon button — is as wide as its caption, and
planning it at the control's width would overflow the band by the
difference. `RIBBON_IA.md` §5 is full of two-word captions over
one-glyph controls, so this is the common case rather than the corner.

`content_width` is what the group's controls occupy: its **widest row**
for a group that wraps, plus the Large run that leads it if it has one.
A wrapped group costs its widest row and not the sum of its items — that
substitution is the entire width benefit of wrapping.

Why this takes a number and not a [`GroupRows`]: a group may have a
Large run *beside* its rows, so the content width is a sum of two things
and only the caller knows both. The guard that a width is never asked for
before the wrap is decided is therefore structural rather than typed —
`measure_group` is the one caller, and it computes the content width
immediately above this call.

### `struct BandPlan`

Returned by [`plan_band`]. Every field is a decision the renderer then
obeys without re-deriving anything, so that the arithmetic exists in
exactly one place and can be tested without a window.

### `fn plan_band`

# Arguments

- `available` — the band's usable width in points. A negative,
  infinite or NaN value is treated as zero; a layout pass can hand out
  `f32::INFINITY` for available width inside an unbounded container,
  and `INFINITY - overflow_width` is still infinity, which would
  silently disable overflow rather than show everything.
- `group_widths` — each group's planned width, in manifest order.
- `separator` — the width of the vertical rule drawn between adjacent
  groups, including its own spacing.
- `overflow_width` — the width the overflow affordance needs. Computed
  for the **widest label it could ever show** (see
  [`overflow_label`]), so the reservation can never turn out to be too
  small once the hidden count is known.

# The algorithm, and the one line that matters

1. If everything fits, show everything and reserve nothing. An
   overflow control that took space when there was nothing to overflow
   into it would be a permanent tax on the band.
2. Otherwise `group_budget = available − overflow_width − separator`,
   **clamped at zero**. This is the line the whole module exists for.
   The separator is subtracted too, because a rule is drawn between
   the last visible group and the overflow control.
3. Fill `group_budget` greedily from the front.

Step 3 can place **zero** groups — at a width narrower than one group
plus the reservation, `shown` is 0 and every group is in the menu.
That is the correct answer and it is the case the reservation exists
for: the band degrades to a single "⏷ N more" control that still
reaches everything, rather than to a band of clipped groups with no
route to the rest.

### `fn overflow_label`

The chevron is part of the label rather than a separate glyph so the
control is one measurable string, and so a build with no icon set
still shows an affordance rather than an empty button.

## The chevron is `⏷` U+23F7, and the obvious choices are all tofu

egui's bundled font stack (Ubuntu-Light + NotoEmoji + emoji-icon-font)
has no face for most of the downward chevrons somebody reaches for first,
and a codepoint with no face draws as an empty box — `□ 1 more`. The near
misses are worth naming because each is a plausible substitution:

| codepoint | in the font? |
|---|---|
| `⌄` U+2304, `▾` U+25BE, `▼` U+25BC, `⌃` U+2303, `˅` U+02C5 | **no** |
| `⏷` U+23F7 | **yes** — and its siblings `⏴` U+23F4 / `⏵` U+23F5 are already in use |

Measured with `Fonts::has_glyph`, not assumed.

**Why this crate cannot test it and the application must.** `cargo test
-p egui-shell` compiles without egui's `default_fonts`, so `has_glyph`
here would answer about a font set that does not exist in any real
build — the test would pass, vacuously, for the whole life of a defect.
The assertion therefore lives with the fonts, in the application, beside
the ones that already guard the status bar and the find bar. The crate
that owns the string is structurally unable to check it.

### `fn overflow_width`

# Why every reachable label is measured, not just the largest count

The hidden count is not known until [`plan_band`] has run, and
[`plan_band`] needs this number as an input. The circularity has to be
broken by reserving for a label that has not been chosen yet, and the
only safe direction is the worst case.

Measuring `"⏷ N more"` for `N = total_groups` alone is the obvious
answer and is wrong. More hidden groups means a longer *character count*
and not a greater *width*: with no font installed every label measures
zero and the two agree, but with real metrics `"⏷ 8 more"` is wider than
`"⏷ 9 more"` in any face whose digits are not tabular, so a band of nine
groups showing one hidden would draw a control wider than the space
reserved for it — the affordance overhanging the band's right edge, which
is failure mode #8 with the control present but partly unclickable.

So the reservation is `max` over every label the control can ever
display: `1..=total_groups`. That makes the claim *"the drawn control
never exceeds its reservation"* a proof rather than an argument about
digit shapes. The cost is one memoized galley lookup per group per
frame — `egui` caches layout jobs, and a band has single-digit numbers
of groups.

`measure` is the caller's text-measuring function, so this stays free
of `egui`.
