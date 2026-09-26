# `egui-shell/ribbon/band`

## Item notes

### `fn measure_group`

The two answers come from one call because they are one decision: a
group's width *is* its widest row, and its widest row is decided by the
wrap. Returning them together is what makes it impossible for
[`render_band`] to plan against one split and draw another — the failure
that would show up as a clipped group and never as anything a reader
would recognise as a bug.

### `fn measure_group_rows`

S5's whole implementation on the measuring side. `wrap_group` already
searches for the **narrowest** packing that fits within the row limit it is
handed, so asking it for three rows instead of two returns the three-row
layout when that is narrower and the two-row one when it is not — no new
packing logic, and no way for the two answers to disagree about how a group
wraps.

Called twice per group per frame, which is affordable: it is arithmetic over
a list of item widths that were going to be measured anyway, and the
alternative — caching last frame's answer — is the feedback-loop shape this
project has paid for three times.

### `fn partition`

Two rules in one pass, because both change what the group measures and a
second pass could apply one of them and not the other:

* an item whose `visible_when` does not hold is **dropped before
  measurement**, so its space is reclaimed rather than reserved for a
  control that never draws;
* a `Large` item **leads**. See [`sizing`]'s header for why a Large control
  cannot live inside the row wrap, and why leading costs nothing an author
  wanted.

### `fn measure_item`

# Two corrections that only matter once text has a width

**The icon/label gap is `icon_spacing`, not `gutter`.** A control with
both halves is drawn as an `egui::Atoms` row, and `AtomLayout` spaces
its atoms by `ui.spacing().icon_spacing`
(`egui-0.35.0/src/atomics/atom_layout.rs`), which this crate's theme
does not set and therefore leaves at `egui`'s 4 pt. Estimating the gap
as the theme's `gutter` agrees with that by coincidence at the compact
density and **under**-estimates by 4 pt per control at the comfortable
one. Under-estimating is the dangerous direction — it is the direction
that lets a group spill into space the band has already promised to
something else — so the estimate asks `egui` for the number `egui` will
use.

**A separator inside a group costs its line, not its line plus two
gaps.** [`separator_width`] is the cost of a rule *between two groups*,
which includes the `item_spacing` either side of it. Inside a group,
[`plan::group_width`] already adds one gutter between every adjacent
pair — including the pairs the separator forms with its neighbours — so
charging the full inter-group figure here counts those two gaps twice.
It over-estimated rather than under-estimated, so it hid a group early
instead of clipping one, which is why nothing caught it; it was still
wrong by 2 × `gutter` for every separator in the manifest.

### `fn a_caption_is_never_empty_even_for_an_unvalidated_group`

The fallback chain — caption → id → a literal — is what makes
"every rendered group emits a caption" a *total* claim rather than
one that holds only for validated manifests. An unvalidated
manifest is exactly the input this module has to survive, because
the defect it exists to prevent is a band of unlabelled controls
and an empty caption reproduces it precisely.

Falling back to the id is also diagnostic: `page_display` sitting
in a caption slot names the manifest entry that needs fixing,
where a blank names nothing.

### `fn the_selected_condition_name_is_namespaced`

The `:` is load-bearing: with a `.` an application could
accidentally define a real condition called
`selected.view.single` and turn a toggle on from a distance.

### `const CAPTION_GAP`

Small on purpose: the caption must read as belonging to the row above
it rather than as a line of its own. The salvage source used the same
constant for the same reason.

**2 → 3 on 2026-09-04**, from `mockups/pdfcer-shell.html`'s
`.grp .cap { padding: 3px 0 5px }` — the first figure. One point, and it
is here rather than left alone because the operator's instruction was
*"exactly like that including sizing"* and because the caption's font
went up two points in the same pass: a 9 pt caption 2 pt below its row
and an 11 pt caption 2 pt below its row are not the same optical gap.

### `const BAND_PADDING_BOTTOM`

`mockups/ribbon.html`'s `.band { padding: 8px 10px 4px }` — the third
figure. Measured in the running application before it existed: the group
captions ended at y = 103 and the dock's tab bar began at y = 105.3, so a
10 pt caption drawn `weak()` and `small()` was separated from the panel
header below it by rather less than a line of its own leading. The caption
is the one piece of text that says what a block of controls is *for* (see
this module's header on why it is beneath the controls at all), and a
caption sitting on the seam reads as a label for the thing below it.

# Why this is added to [`band_height`] and not to the group loop

`PROJECT_PLAN.md`'s **R128**: the band's height must be a function of the
theme and the font and of nothing a tab can vary. Padding drawn as "space
after the last group" would be exactly such a variation — a tab whose
groups all went into the overflow menu draws no group and would get no
padding, so the band would be 4 pt shorter on that tab than on the one
beside it and the canvas below would move on a tab click. Folding it into
the derivation instead keeps one number, reserved before anything is
drawn, spent identically whether the band holds five groups or none.

# Why the mockup's top and side padding are not here

Only the bottom edge was measured as wrong. The band's top is already
separated from the tab strip by [`super::tabs::strip_underline`] and the
enclosing layout's own spacing, and the band's horizontal padding is a
decision about the *band's* left edge that would shift every group in it —
see [`plan::GROUP_PADDING`]'s closing note. Adding either one would be
visual churn beyond the defect, and churn is harder to review than the
change it is mixed into.

### `const SELECTED_CONDITION_PREFIX`

# Why toggles are expressed as a condition rather than as a field

A ribbon has toggles: "Single page" is either the current page-display
mode or it is not, and a control that cannot show which is a control
the operator has to test by clicking. But *which* toggle is on is
application state, and [`crate::commands::Command`] deliberately holds
no state — it is a registration, built once, shared, `Clone`.

The [`crate::commands::ConditionSet`] already exists, is already
republished every frame, and already carries exactly this kind of
fact. So a command with id `view.single` renders selected while the
condition `selected:view.single` is set. No new type, no new manifest
field, no per-frame registry mutation, and the state is inspectable in
the same place every other piece of frame state is.

The prefix uses `:` rather than `.` so it cannot collide with an
application's own dotted condition names.

### `fn caption_text`

The manifest's caption is optional because a *layer* may omit it (a
layer that says `Group(id: "render")` is reordering a group, not
blanking its caption). A complete manifest is required to have one by
[`crate::manifest::Shell::validate`].

This is what happens when an application renders a manifest it did not
validate. Falling back to the **id** rather than to `""` is the whole
point: an empty caption reproduces the exact defect this module exists
to prevent, whereas `page_display` in the caption slot is visibly
wrong, unmistakably diagnostic, and names the group whose manifest
entry needs fixing.

### `struct GroupBox`

Two numbers rather than one, because they pin two different things and
a group that satisfied only the first would still make the band ragged:

- [`Self::rows`] pins the **captions** to one baseline — the mockup's
  `justify-content: space-between`. A one-row group is padded out to the
  height two rows would have taken, so its caption lands where its
  two-row neighbour's does.
- [`Self::total`] pins the **band** to one height — R128. It closes the
  gap between what the reservation promised and what the caption
  actually measured, so that a band showing five groups and a band
  showing none (everything in the overflow menu) come out identical
  rather than identical-to-within-a-caption's-rounding.

[`Self::NATURAL`] — both zero — means "as tall as your content", which
is what the overflow menu wants: the band's height is a fact about the
band, and padding a popup entry out to it would put a hole under every
one-row group in the menu.

### `fn entitled_bounds`

Shared by the band and by [`super::strip`], because both rows have the
same obligation and the same trap: whatever a row is *offered* by the
layout is not necessarily a width the window has.

Three candidates, and the row gets the **narrowest** of them, because
each is an upper bound on where a control can be both drawn and
clicked:

1. `ui.available_rect_before_wrap()` — where this `Ui`'s cursor is now.
   Supplies the top edge and the left edge in the ordinary case.
2. `entitled` — the rectangle the application handed
   [`super::Ribbon::render`], captured **before** anything was drawn
   into it. This is the one that matters: see the module header on
   `max_rect` growth. Nothing a sibling row does can inflate it,
   because it was read before the sibling existed.
3. `ui.clip_rect()` — what is on screen. `egui` never grows a clip rect
   to fit overflowing content, so it is the honest answer to "would the
   operator see a pixel painted here", which is what failure mode #8 is
   ultimately about.

Only the horizontal extent is negotiated. The vertical extent is the
caller's — [`render_band`] replaces the bottom edge with
`top + `[`band_height`] immediately, and clamping the height to the clip
rect instead would make a partially-scrolled ribbon lay its captions out
differently from an unscrolled one.

A degenerate result (right ≤ left) is returned as a zero-width rect at
the left edge rather than as an inverted one: [`plan::plan_band`] reads
zero as "nothing fits, everything goes to the menu", which is the safe
answer, whereas an inverted rect would produce a negative width and a
nonsense plan.

### `fn render_band`

`entitled` is the rectangle the application handed the ribbon, read
before any of the ribbon was drawn. It is a parameter rather than
something this function derives because by the time the band runs, the
`Ui` it is given can no longer report it — see [`entitled_bounds`] and the
module header.

### `fn captioned_group`

Lays one group out as Office lays one out: its controls in up to
[`plan::GROUP_ROWS`] rows, its **caption beneath them, centred on the
widest row**. The body is a closure rather than a predicate for the
reason the salvage source records: to put the caption *under* the
controls, the controls must be emitted inside a vertical container that
is still open when the caption is written, and a predicate returning
`bool` has already returned before the body runs.

See the module header for why this being the only such function is the
point.

# `rows`

The split [`plan::wrap_group`] decided, handed in rather than recomputed
— see [`GroupRows`] for why the plan and the renderer must not each own
a copy of that arithmetic.

# `box_`

The heights this group is padded out to — see [`GroupBox`], and
[`GroupBox::NATURAL`] for the overflow menu, where neither applies.

The padding is measured against the `Ui`'s own **cursor** rather than
predicted, so a control that turned out taller than
[`crate::theme::Metrics::control_height`] shortens the gap instead of
pushing the caption out of the band. The cursor, specifically, and not
`min_rect`: `egui` advances the cursor past a laid-out rect by
`item_spacing`, so the two differ by exactly one gap after every row and
padding against the wrong one leaves each group a gap taller than the
height the band reserved for it. That is a 3 pt discrepancy that shows
up only when a tab has *no* group in the band to compare against — which
is to say, only in the R128 case.

# The horizontal inset

The mockup's `.group { padding: 0 13px }`, drawn at
[`plan::GROUP_PADDING`] — the width [`plan::group_width`] has budgeted for
it all along, so this inset is paid for out of a reservation that already
existed and costs the band nothing. See the module header.

The reported rectangle **includes** the inset, deliberately: the group box
is what the operator perceives as the group, and a report that named only
the content would make "is there padding" unanswerable from outside the
process, which is the question that produced this change.
