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
