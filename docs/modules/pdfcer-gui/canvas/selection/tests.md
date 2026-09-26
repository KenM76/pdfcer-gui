# `pdfcer-gui/canvas/selection/tests`

## Item notes

### `fn selecting_an_annotation_and_selecting_content_replace_each_other`

The invariant this type owns, and the reason the annotation lives here
as a field rather than beside this state on `OpenDoc`. A build where
both could be non-empty would draw two kinds of outline at once and
leave `format.delete` and the Delete key with two plausible meanings —
one of which removes page content the operator did not point at, which
is the loss `deletable_objects_on`'s own guard calls *"one line and the
whole view"*.

Both directions, because the two writers are different code and a build
that cleared only one way is the more likely mistake.

### `fn a_selected_annotation_is_not_an_empty_selection`

Pinned separately from the exclusion test because it is a different
failure: a build that kept the two exclusive and still answered
`is_empty() == true` over a selected stamp would hide the contextual
Format tab — the one surface the whole feature exists to reach — while
the outline sat on the page saying something was selected.

### `fn navigating_the_view_never_alters_the_selection`

The acceptance criterion, as close to literally as a headless test can
state it: select a node, then perform every navigation the roadmap
names — zoom out three rungs, pan, change fit mode, rotate the view,
change page-display mode, switch ribbon tab — and assert the selection
is byte-identical afterwards.

# Phase 3's gestures were added to THIS sweep, not to a parallel test

The hand-tool pan, the anchored discrete zoom, the marquee zoom and
zoom-to-selection are navigation, so they belong to the invariant that
already governs navigation. A second test asserting the same property
about four more operations would be a second place for the property to
be stated — and the first one to be forgotten when a fifth arrives.

**Zoom-to-selection is the interesting addition**, because it is the
only navigation that *reads* the selection: it resolves the selection's
bounds and frames them. Reading is exactly where a "helpful" edit —
normalise the entries, collapse to the outlined ones, drop what has no
bounds — would creep in, and it would be invisible until the operator
zoomed to a node and found they had selected the object instead.

What this cannot reach is the *wiring*: that a released
`MarqueeIntent::Zoom` never calls [`SelectionState::marquee`] at all.
That is structural in `canvas::interact` — the two intents are separate
match arms over an exhaustive enum, and only one of them names the
selection — and it is asserted from the gesture side by
`canvas::gesture`'s `a_zoom_marquee_is_the_same_band_with_the_other_intent`.

It is expressed as *"drive the view state and then compare"* because
that is the honest model of what navigation is: those operations act
on [`crate::viewer::ViewState`], and the property being asserted is
that no route exists from there to here. The test would fail the
moment somebody gave `SelectionState` a screen coordinate to keep in
step, which is the defect it guards.

### `fn paging_away_and_back_keeps_the_selection`

Paging away rebuilds the provider for the new page. A `resolve` that
pruned everything it could not find would wipe the selection on the
way past, and coming back would find nothing.

### `fn only_the_object_rung_offers_anything_to_delete`

The canvas keys and the ribbon's `format.delete` both read
[`SelectionState::deletable_objects_on`], so this test covers both. It
is the destructive case: the only wired verb removes whole objects, and
one measured CAD export holds an entire drawing view as a single path
object with 1,194 subpaths.

The page filter is asserted too, because a paint-order index is a
position on **one** page and handing `delete_objects` an index from
another one would remove whatever happens to sit at that slot.

### `fn shift_picking_a_second_anchor_adds_it_rather_than_replacing`

# Why this test was written


> *"TWO MARKED ANCHORS WERE CLICKED, THE SECOND WITH SHIFT, AND 1 ENDED UP
> SELECTED."*

It had SKIPPED on every earlier run, for want of a `--doc-point` whose
subpath carried more than one anchor, so the rung had never actually been
exercised — by it or by anything here. `a_plain_click_replaces_and_a_shift_
click_toggles` covers the **Object** rung only, and nothing covered this one.

So the question was whether the *model* is wrong or the driven path is, and
the two need very different fixes. This is the cheap half of that question.

It asserts through [`SelectionState::selected_nodes_on`] as well as
through the entry count, because that accessor is what
`canvas::moving`'s multi-node drag actually reads. A model that held two
entries but reported one node would satisfy a length check and still fail
the operator.

### `fn shift_picking_a_second_chunk_adds_it_rather_than_replacing`

The Part rung has held several entries since normalisation was written:
entries differing only by `subpath` survive, because the collapse to
`Object` fires only when they differ by object or page. Nothing read it that
way, so `canvas::moving` asked for the first entry and a set of four chunks
moved one. This asserts the accessor the plural move and the plural delete
refusal both read, not the entry count, for the reason the anchor twin
gives: a model holding two entries and reporting one chunk satisfies a
length check and still fails the operator.

### `fn a_band_sets_several_chunks_of_one_object_at_once`

[`SelectionState::select_parts`] is the only writer of a Part-rung set that
did not come from a click, and the three properties asserted here are the
three its callers depend on: the rung is entered, the set is ascending and
unique whatever order the band reached them in, and an empty answer leaves
the **ground state** rather than an entry-less Part rung — which would put
the next click's first selection into a rung the operator never entered.

### `fn escape_ascends_one_rung_per_press`

The old shell shipped Escape as "clear everything", so an operator two
rungs inside a drawing found one press putting them back at the page.
Asserted as a sequence of outcomes rather than a boolean, so a
regression that collapsed the ladder cannot pass by clearing on the
first press and reporting `true` three times.

### `fn a_subtracting_band_removes_only_what_it_hit`

The operator, 2026-09-03: *"I can't unselect things once I have selected
them for redaction."* This is the half a click could not do — on a sheet of
overlapping strokes, removing one object from a selection of twenty by
clicking it precisely is often not practical, and a band is how the work is
actually done.

### `fn a_subtracting_band_that_hits_nothing_does_not_clear_the_selection`

The asymmetry with a plain band is deliberate and is the whole safety of the
gesture. A plain band that encloses nothing means *"select nothing"*, which
is how every editor cancels a selection. A **subtracting** band that hits
nothing means *"remove nothing"* — clearing there would make a mis-aimed
Ctrl-drag destroy the very selection the operator was trying to refine,
which is the opposite of what they asked for.

### `fn a_placed_object_replaces_the_selection_at_the_object_rung`

The operator, 2026-08-26: *"if I add an image I Expect to click on it to
resize but dragging doesn't resize."* He was right about the symptom and it
was never the resize — a driven check had already proved a selected image
resizes from a corner grip (`resize-commit grip=SouthEast sx=0.6810`). It
arrived **unselected**, so his first press was a press on unselected paper,
which `gesture::meaning` reads as a marquee. He watched a rubber band.

Three properties, and each is a separate way to get this wrong: the new
object is selected; it is the ONLY thing selected; and the rung is Object,
not a rung inside it.

### `fn a_leaf_is_never_a_delete_operand`

The consequence if this ever returned the leaf's number: `delete_objects`
would resolve it against the **page's** paint order, find a real object
there, and delete the wrong thing — silently, because the index is in
range. That is the file-corruption failure `TargetId` exists to make
unrepresentable, asserted at the place it would have happened.

### `fn a_second_click_on_a_boxed_text_block_descends_to_the_chunk_under_it`

The first click selects the block; the second, landing on a chunk of the
same block, narrows. `hit.chunk` is what carries the permission: it is
`true` only where the operator can see a box to aim at, which is
[`crate::canvas::clicking`]'s job to establish and this function's to
obey.

### `fn a_part_that_is_not_a_visible_chunk_never_narrows_the_selection`

The honest half of the rule. Narrowing to a unit nothing drew is the
unpredictability O215 reports, so a `part` with `chunk: false` — a path's
subpath, a one-line text object, or the switch turned off — changes the
verb set for nobody.

### `fn a_click_on_one_of_several_selected_blocks_selects_it_rather_than_descending`

With two blocks selected, a click on one of them means *now just this one* —
the ordinary Object-rung answer — and skipping that step would descend into
a block the operator had not yet singled out.

### `fn every_click_after_the_descent_repicks_the_chunk_under_the_pointer`

O215 ask 1 in one sentence: *"selecting one chunk is repeatable."* The
third click here returns to chunk 2 and must land there, which is also why
a driven check of this behaviour has to **alternate** rather than click one
chunk twice: [`crate::diag::trace_changed`] emits only on change.
