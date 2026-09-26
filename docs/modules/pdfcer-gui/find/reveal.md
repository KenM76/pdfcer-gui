# `find::reveal` — bringing a hit onto the screen

The second half of what "go to this hit" means. The first half is a page
change, which [`super::apply`] performs directly through
[`crate::viewer::ViewState::go_to_page`]; this module is everything after
that — the pending intent, the gate that decides when to spend it, the
scroll solve, and the projection from PDF geometry into the space the
canvas paints in.

## Why it takes two frames, and why that is not avoidable

The request is made during the **apply phase**, which runs *after* the
canvas has drawn ([`crate::app`]'s frame order, step 3). The page change
it carries is applied in the same phase. So the earliest frame on which
the target page's real drawn size is known — and the drawn size is what a
scroll offset is solved against — is the **next** one.

That is exactly the situation [`crate::app::state::ZoomAnchor`] documents
for a zoom, and the shape of the answer is the same: *record the inputs
now, solve later*. [`Reveal`] is therefore a fraction of the page rather
than a canvas point, because a fraction is independent of the zoom and
survives the operator zooming in between the two frames.

## Why it cannot simply ride the zoom anchor's handshake

Because that handshake is gated on the page's **drawn size changing** —
`zoom::anchor_step` compares `display_now` against the size recorded when
the anchor was armed, and treats "unchanged" as *the zoom has not landed
yet*. A find reveal changes the scroll offset and nothing else, so under
that gate it would `Hold` for one frame and then `Drop`, every time,
having moved nothing. A search that navigated to a hit and then did not
scroll to it is precisely how a plausible implementation of this feature
ships doing nothing.

So the gate is different — **the page index**, not the drawn size — and
the *solve* is shared: [`take_reveal_offset`] asks
[`crate::canvas::geometry::offset_holding_anchor_at`], which is the single
owner of *"put this page point at this screen position"* and is the same
function `canvas::zoom::place_centred` uses to centre a framing zoom. One
arithmetic, two callers, two gates.

## Why this is a module rather than a section of [`super`]

Rule R2's 1,500-line ceiling requires the split, and the seam it forces is
a real one: [`super`] answers *what is a search, and what does its answer
mean* — the query, the options, the wildcard trap, staleness, the readout
— while this file answers *how does one hit get in front of the operator*.
The two change for different reasons and are read at different times, and
the coordinate-space reasoning below has nothing to do with the search
semantics above.

## Item notes

### `const REVEAL_GRACE_FRAMES`

A page change raised during the apply phase lands on the *next* frame, so
one would very nearly do. Four, because a reveal that is never spent is
worse than one that is spent late: it would sit on the document waiting
for the page index to coincide by accident, and would then scroll the view
somewhere the operator did not ask for, minutes later, in response to an
unrelated page step. This is the same hazard `canvas::zoom`'s
[`crate::canvas::zoom::AnchorStep::Drop`] exists for, and the same answer.

### `fn hold_the_zoom_if_asked`

# What actually changes the zoom, because it is not this module

Nothing in `find` has ever called `set_zoom`, and this function does not
either. The zoom changes on a jump because **a fit mode is still switched
on**: [`crate::viewer::ViewState::apply_fit`] runs on *every* frame from
`canvas::present`, recomputing the zoom from the extent of whatever page is
showing. Land on a sheet of a different size and it re-scales, by exactly
the ratio of the two sheets.

That is not a hypothetical. This repository's own `fixtures/four-pages.pdf`
carries three page sizes: page 1 is 2384×1684 pt, pages 2 and 3 are
612×792, page 4 is 306×396. Under the shipped default of
[`FitMode::Page`] a jump from page 1 to page 2 moves the zoom by 3.9×, and
from page 1 to page 4 by 7.8×. A drawing set with a letter-size cover sheet
in front of A1 sheets is the same document. The operator's report was of a
search that "changed the zoom"; the cause is a fit doing precisely what a
fit is for, on a page he did not choose to go to.

The **engine's** `pageops/four-pages.pdf`, which most of the tests in
this module open through `open_fixture`, is four *identical* 612×792 pages
and cannot show the effect at all. The tests for this function therefore
open the local file through `open_local_fixture` instead — a distinction
that is easy to lose because the two fixtures share a name.

# So the intervention is to stop following the fit, not to set a number

`set_fit(FitMode::None)` leaves [`crate::viewer::ViewState::zoom`] exactly
as it is and stops `apply_fit` from touching it again. The operator keeps
the size they were reading at, on the new page.

**This deliberately changes a visible setting**, and that is disclosure
rather than a side effect: the status bar's zoom readout stops saying *Fit
page* and starts showing the percentage. A build that held the zoom while
still *claiming* to be in Fit page would be lying about its own state in
the one place the operator can check it. The control's tooltip says this
will happen, in [`crate::text::find::find_zoom_tooltip`].

# Three guards, and each one is a case where doing nothing is correct

- **The control is on** — the shipped default. Nothing happens, so an
  operator who never opens the options menu is unaffected.
- **The page is not changing.** Stepping between two hits on the same sheet
  cannot re-fit, because `apply_fit` would compute the identical scale from
  the identical extent. Dropping the fit anyway would take the operator out
  of Fit width for pressing *Next* — a setting silently changed for
  nothing.
- **No fit is active.** The zoom is already pinned; there is nothing to
  hold off, and assigning `FitMode::None` over `FitMode::None` would still
  emit a `find-zoom-held` trace line saying an intervention happened — a
  harness reading that line would believe it.

Called from [`reveal_current`] **before** `go_to_page`, because the second
guard is a question about the page the operator is leaving.

# Why it returns a `bool` nobody in the shipped path reads

Because the third guard is otherwise **unobservable to a test**, and an
unobservable guard is one that can be deleted with every check still green.
Deleting it changes exactly one thing: a trace line on stderr that a unit
test cannot capture. Under that guard's own conditions the acting branch
would assign `FitMode::None` over `FitMode::None` and leave `zoom` alone, so
*every* piece of document state a test could assert on is identical either
way. Measured, not assumed: with the guard removed, all ten tests in this
module still pass.

So the function reports what it did, the tests read the report, and
[`reveal_current`] discards it. That is the cheapest way to make a real
guard falsifiable without a stderr capture harness.

### `fn a_quad_projects_into_canvas_space_with_the_y_axis_flipped`

Driven against a real page so the bridge under test is the one the
canvas paints through — `viewer::pdf_space_to_canvas`, which inverts
the renderer's own device transform. A hand-built transform here would
prove that this module agrees with itself.

The property asserted is the **Y flip**: PDF user space is Y-up from
the CropBox's lower-left, canvas space is Y-down from the page's
top-left, so a quad near the top of the page in PDF terms (a large Y)
must land near the top in canvas terms (a small Y).

### `fn a_reveal_whose_page_never_arrives_is_abandoned`

Otherwise it would sit on the document until the page index coincided
by accident and then scroll the view somewhere the operator did not
ask for — the hazard `canvas::zoom::AnchorStep::Drop` exists for,
which is why the answer here is the same one.

### `fn land_on`

This reproduces the part of the frame the defect lives in rather than
asserting around it. Checking only that `fit` became `FitMode::None`
would prove this module agrees with itself; running the fit afterwards
proves the number the operator reads actually held still, which is what
he asked for.

### `fn settled`

`open_local_fixture`, **not** `open_fixture`: the engine fixture of the
same name is four identical pages and cannot show this effect at all.
See [`hold_the_zoom_if_asked`].

### `fn the_control_off_holds_the_zoom_across_a_jump`

Page 1 of the fixture is 2384×1684 pt and page 4 is 306×396, so under
Fit page the zoom would otherwise move by 7.8×. The assertion is
**exact equality**, not a tolerance: nothing is supposed to touch the
number at all, so any drift at all is a second code path that should
not exist.

### `fn a_step_within_one_page_leaves_the_fit_alone`

`apply_fit` would compute the identical scale from the identical
extent, so there is nothing to hold off, and dropping the fit anyway
would take the operator out of Fit page for pressing *Next*: a setting
silently changed for no benefit at all.

### `fn no_fit_active_means_no_intervention`

Asserted rather than assumed because the acting branch would otherwise
assign `FitMode::None` over `FitMode::None` and emit a `find-zoom-held`
trace line claiming an intervention that did not happen — which a
harness reading that line would believe.

### `fn fit_width_is_held_as_well_as_fit_page`

The guard tests `fit == FitMode::None`, so every other mode is meant to
be covered by the one branch. This is the check that it was not written
as `== FitMode::Page`, which would pass every test above.

### `fn the_shipped_default_leaves_the_zoom_free`

[`FindState`] carries a hand-written `Default` for precisely this;
`#[derive(Default)]` would give `false` and quietly change the
behaviour of everyone who has no preference file yet.

### `fn the_solve_puts_the_hit_in_the_middle_of_the_viewport`

Asserted as the *outcome* — where the point ends up on screen —
through `geometry::anchor_screen_pos`, rather than as an offset, so
this checks the framing and not that the code agrees with itself.

### `struct Reveal`

Lives on [`OpenDoc`] beside `zoom_anchor`, and for the identical reason:
**it has to span two frames.** The request is made during the apply phase,
which is after the canvas has drawn; the page change it carries is applied
in the same phase; so the earliest frame on which the target page's real
drawn size is known is the next one. Recording the intent and solving it
later is the same handshake `ZoomAnchor` documents, minus the zoom.

### `fn reveal_current`

Two halves, and both are needed. `go_to_page` is called directly rather
than raised as [`crate::app::actions::Action::GoToPage`] because this
**is** the apply phase — the funnel's rule is that no widget mutates a
document, not that the apply phase may not — and it is the same method
that action's arm calls, so the clamp has one owner either way.

### `fn take_reveal_offset`

Called from [`crate::canvas::offset`]'s scroll-priority chain, once per
frame while a reveal is pending: below the fit and the zoom anchor, which
are explicit instructions about the view, and above the plain page-change
scroll, which would otherwise satisfy the page change without centring the
hit.

# It reuses the anchoring solve rather than writing a second one

[`crate::canvas::geometry::offset_holding_anchor_at`] is the function
`canvas::zoom::place_centred` uses to express *"put this page point at
this screen position"*, and it is the single owner of that arithmetic.
Asking it for `target = viewport centre` is exactly what a framing zoom
asks for; the only difference is that this one does not change the zoom,
so it cannot ride `ZoomAnchor`'s handshake — that handshake is gated on
the page's **drawn size changing**, which is precisely what a scroll does
not do. Hence a separate pending value and a separate gate, and a shared
solve.

# Why the gate is the page index rather than a frame count alone

The reveal is spent on the first frame that is *showing the hit's page*.
Spending it earlier would scroll the outgoing page to a fraction that
means nothing on it; spending it on a frame count would be a guess about
how long a page change takes. The frame count is the *abandon* rule, not
the spend rule — see [`REVEAL_GRACE_FRAMES`].

### `fn quad_to_canvas`

All four corners are mapped and bounded, rather than mapping two: the page
transform may rotate (`/Rotate 90` is ordinary on a landscape drawing
sheet), and a rotation sends the quad's `ul`/`lr` pair to two corners that
are no longer the extremes. Bounding all four is correct under every
rotation and costs three extra multiplications.

`None` when the page's device transform will not invert, which is
[`crate::viewer::pdf_space_to_canvas`]'s own decline for a degenerate
page. The hit is still counted and still navigable — see [`Hit::canvas`].

`pub(crate)` rather than `pub(super)` so that canvas text selection can
project its line boxes through **this** function rather than mapping two
corners of its own. That is a correctness requirement rather than
tidiness: a selected word and the same word *found* are two washes over
the same glyphs, and on a `/Rotate 90` sheet a two-corner projection puts
one of them somewhere else. One projection, two surfaces — the same discipline
`canvas::mapping` applies to the screen⟷canvas hop.
