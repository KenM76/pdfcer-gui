# `pdfcer-gui/panels/pages/thumbnails/tests`

## Item notes

### `fn settled`

Textures need an `egui::Context` and a live renderer; the *policy*
does not, and the policy is what these tests are about. So the state
is set through `unavailable`, which produces the same "not pending"
answer from [`ThumbnailCache::state`] by a route a headless test can
take.

### `fn the_current_page_is_drawn_before_its_neighbours`

It carries the highlight ring, so it is the tile the operator is using
to answer "where am I" — and a ring around a tile reading "not drawn
yet" answers that with the page number they already had.

### `fn a_skipped_page_leaves_the_feature_on_and_its_neighbours_drawn`

> *"the drawing page previews checkbox should never automatically turn
> off."*

This is the regression test for the whole change, and it is written
against the two things that were true before it and must never be true
again: the tick going out on its own, and every other page losing its
picture because one page was expensive.

⚠ Note what it does NOT assert — that page 2 gets a picture. It does
not; the budget abandoned it, and `Unavailable::Abandoned` is the
honest record of that. What it asserts is that page 2's misfortune is
**page 2's alone**.

### `fn raising_the_limit_retries_what_it_skipped_and_nothing_else`

The trap this avoids: a dial that can only ever remove pictures. The
operator's sole reason for touching the box is the tile reading "Not
finished", so if raising the number left that tile exactly as it was,
they would reasonably conclude the box does nothing.

It also asserts the half that must NOT happen — a page the renderer
*refused* is not retried, because it will be refused again and a
`DragValue` reports a change on every pixel of a drag.

### `fn the_limit_cannot_be_set_outside_what_is_useful`

A control narrower than what the value may legally hold silently
rewrites it — the argument `canvas::markup::swatch` makes for taking
the pen's own range. Here the risk runs the other way: a future caller
that is not the `DragValue` could set a budget of a millisecond, which
is a build where every page is abandoned while the checkbox reads
"on".

### `fn zero_milliseconds_means_no_limit_and_survives_the_clamp`

The round trip is asserted in both directions: what the file holds
becomes what the cache holds, and what the cache holds becomes what the
file holds. A one-way test would let the write-back turn *no limit*
back into the default on the next gesture, which the operator would
experience as the box refusing to keep the zero he typed.

### `fn setting_the_same_limit_again_changes_nothing`

`DragValue` reports a change on every pixel of a drag. If `set_budget`
dropped the abandoned entries unconditionally, a drag across the box
would re-queue — and therefore re-render — the expensive page dozens of
times, on the UI thread, while the operator was still choosing a number.

### `fn turning_previews_off_by_hand_stops_explaining_a_skip`

The tick is now the only thing that stops the grid, and when it is
clear the skip note must go quiet: every tile is blank for a reason the
operator already knows, and a sentence about page 3's time limit beside
eleven other blank tiles points at the wrong cause.

### `fn only_the_operator_may_untick_previews`

The tripwire for O151 rather than a test of behaviour, and it is
written as a source scan because the defect it guards against is a
*future* line of code, not a current output. Two green tests sat beside
the old rule and neither could have caught it: they asserted that the
automatic stop worked, which it did.

`force_on` is the one sanctioned writer. Any other assignment to
`self.on` is pdfcer deciding on the operator's behalf again.

What this test does NOT constrain is how many places CALL `force_on`,
and that distinction is load-bearing. There are three (see its own doc),
and all three carry the operator's instruction rather than forming one.
The sentence here used to say *"called from exactly one place — the
checkbox"*, which was true when it was written and was still sitting in
the file hours after O187 added a second and a third.

# ⚠ It failed on its first run by reading its own assertion, and the
fix is why this file exists where it does


The R2 split solved it structurally: the harness is a different file now,
so `include_str!` cannot reach these strings at all. That is a stronger
guarantee than the string-cutting it replaced, and it is **asserted rather
than assumed** — a scan over a file that no longer contains the subject
finds no violations and reports success, which is exactly what "the gate
stopped running and nobody noticed" looks like from the outside.

⇒ If this test is ever moved back into `thumbnails.rs`, the first
assertion below fires immediately and says why.

### `fn the_four_undrawn_states_say_four_different_things`

The no-placeholders rule for pictures. Four distinct states must map
to four distinct sentences, or the tile is guessing on the operator's
behalf. Asserted against the catalog itself, so a future edit that
makes two of them read alike fails here.

### `fn navigating_keeps_the_cache_and_editing_drops_it`

The invalidation key is the edit epoch, not the page index. Keying on
the page would re-rasterize the visible grid on every Page Down — a
second of frozen UI per keystroke, to redraw pictures that were
already right.

### `fn an_edit_on_one_page_leaves_the_other_pages_pictures_alone`

`OPERATOR_REQUESTS.md` O74 — *"all of the page previews get re-rendered
instead of just the one that is being changed"*. The old `sync` keyed
the whole cache on a document-wide epoch and cleared it wholesale, so
this test could not have been written against it: there was no per-page
input to vary.

### `fn a_document_wide_edit_still_drops_every_picture`

Without this, the test above passes on a build that never invalidates
anything — which would show the operator pictures of content he had
already changed. That is rule 4's "sneaky" and it outranks the slowness
the per-page key exists to fix, so both directions are asserted.

### `fn an_undated_entry_is_dropped`

Unreachable today — every insertion stamps `built_at` — and asserted
because the polarity is the whole safety argument. "Keep what you
cannot date" shows the operator stale content; "drop what you cannot
date" costs one render.

### `fn the_operators_settings_survive_an_edit`

Three instructions and one measurement, and none of them is invalidated
by a change to a page's content: an edit does not make an expensive
document cheap, and an instruction that evaporates on the next edit was
not honoured.

### `fn the_shipped_defaults_draw_something`

Asserted because `#[derive(Default)]` was removed to get them, and a
hand-written `Default` that drifts from its own doc comment is the kind
of defect only an operator finds — a build that draws nothing and
blames a time limit for it.

The budget is asserted as `None` **spelled out**, not as
`PAGE_BUDGET_DEFAULT`: written the second way the assertion is
`x == x` and holds whatever the constant becomes, which is exactly the
drift it exists to catch.

### `fn a_thumbnail_scale_is_always_finite`

An infinite scale reaches `pdfcer-render`'s pixmap guard and comes back
as a refusal, so the tile would read "would not draw" for a page whose
only fault is a malformed `/CropBox` — blaming the render for a
division this function is responsible for.
