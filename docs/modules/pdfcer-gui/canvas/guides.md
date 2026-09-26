# `canvas::guides` — draggable alignment lines, whose home is a page and whose life is a file

The third of `RIBBON_IA.md` §5.2's *"Rulers · Grid · Guides"*, and the one
with a condition on it: a guide must be **draggable**, and it must
**survive a reopen**, which takes a per-document store. This header answers
both, plus the two questions they imply: *what does a guide belong to*, and
*where does it live on disk*.

---

## 1. A guide belongs to a PAGE, and that follows from the grid

[`super::rulers`]' header §2 settles the space the grid is drawn in: page
space, per page, anchored to the sheet's own corner, because a reference
that is not attached to the sheet cannot make a statement about the
drawing. A guide is the same kind of object for the same reason, only
placed by hand instead of by a ladder — so it is stored as
`(page, axis, canvas-space coordinate)` and nothing else.

The alternatives, and why each is worse:

- **Viewport-space**, a line at a window position — scrolling moves it off
  whatever it was aligned to, so it means nothing the moment the operator
  scrolls.
- **Document-wide, applied to every page** — tempting for a 36-sheet set of
  identical drawings, and wrong the first time a set mixes A3 and A1: a
  guide at *y* = 500 is inside one sheet and off the end of another, and
  there is nothing to say which.
- **Per page** *(this)* — a guide is where the operator put it, on the sheet
  they put it on, for as long as that sheet exists.

The per-page answer is also Acrobat's, which matters because an operator
coming from the comparison product should not have to relearn what a guide
is attached to. A document-wide *copy* verb ("put this guide on every
sheet") is a plausible future convenience and is a **different feature**:
it would place N guides, one per page, and every one of them would still be
a page's guide.

---

## 2. Where guides live on disk — a fourth file

`page-display.txt` sits beside `layout.ron` and `recent.txt` as a *third*
store, and [`crate::viewer::remembered`]'s header carries the argument for
it in full. Its three reasons transfer here one for one, so this is a
**fourth** file, `guides.txt`, rather than a field in any of the three:

1. **The lifetimes differ.** `recent.txt` is capped at ten because it is
   *drawn* in a menu. `page-display.txt` is capped at two hundred because
   the cap is about disk. This one is capped at [`CAP`] documents for the
   disk reason as well, but its *rows are unbounded in width* — a document
   can carry many guides — which neither of the others can express.
2. **Forgetting means different things.** "Clear recent files" must not
   delete the guides an operator ruled up on a drawing, and clearing the
   guides on a sheet must not evict it from the recent menu or reset its
   page-display mode. Separate files make that true by construction rather
   than by a rule somebody has to honour.
3. **The format cannot serve either.** `recent.txt` is one path per line
   and nothing else, deliberately; `page-display.txt` is exactly one mode
   id and one path. Adding a variable-length payload to either makes every
   existing line ambiguous with the format it replaced.

**Why guides persist when the three toggles do not.** `view.rulers`,
`view.grid` and `view.guides` are per-document view state that starts off
and is not written anywhere — see [`crate::viewer::ViewState`]. The guides
themselves *are* written. The distinction is not inconsistency, it is the
difference between a switch and a work product: switching the grid on again
costs one click, while re-placing six guides means measuring six positions
again, and losing work is a different class of loss from losing a switch.

The consequence that falls out of it, and is deliberate: **a document with
remembered guides opens with `view.guides` already on.** The presence of
the work *is* the preference, so it does not need storing separately, and
the alternative — restoring invisible guides and waiting for the operator
to discover a toggle — would be a feature that appears not to have worked.
See [`crate::app::state::OpenDoc::new`].

### The format

```text
0:h:120.5 0:v:64 3:h:200<TAB>D:\Drawings\job-4471\sheet-set.pdf
0:v:306<TAB>C:\Users\ken\Documents\report.pdf
```

(`<TAB>` stands for one U+0009; the real file carries the character.)

One line per document, most recently written first, UTF-8, no header. The
separator between the payload and the path is a **tab** — the one ASCII
character a Windows path cannot contain, so it needs no escaping — and the
payload comes **first** because the path is the part that may contain
spaces and must therefore be the whole remainder of the line. Each guide is
`page:axis:coordinate`, space separated, with the axis spelled `h` or `v`
by [`GuideAxis::id`].

A malformed guide is **dropped**, a malformed line is **dropped**, and a
document with no guides is **not written at all**. A corrupt file therefore
degrades into fewer guides rather than into an error the operator has to
dismiss about a preference — exactly as `recent.txt` and
`page-display.txt` do.

Flat text rather than RON for the reason `remembered.rs` gives: **this
crate cannot serialize.** `serde` and `ron` are dependencies of
`egui-shell`, not of `pdfcer-gui`.

---

## 3. Dragging, and why it cannot disturb the selection

Two gestures create and move guides, and both are the ones every peer uses:

* **drag out of a ruler** — the top ruler yields a horizontal guide, the
  left ruler a vertical one;
* **drag the guide itself** on the page, to move it;
* **release anywhere that is not a page** — the grey between sheets, a
  ruler, off the window — and the guide is discarded (if it was being
  created) or **deleted** (if it existed). One rule, both cases.
* **double-click a guide** to delete it without a drag, which is the only
  route that works with the rulers switched off.

### The interaction hazard, and the two-line fix

The canvas's primary button is spoken for: it selects, it marquees, and
under the hand tool it pans. A guide drag that reached
[`super::gesture::GestureState`] would be a drag that moved a guide **and**
rubber-banded a selection.

It cannot, and the mechanism is egui's own rather than a check anybody has
to remember: [`canvas_drag`] registers each guide's catch band **after**
every page widget in the same layer, so the band is the topmost widget
under the pointer and wins the interaction outright. The page's own
`Response` then reports no press at all, `interact`'s step 1 builds an
empty [`super::gesture::PointerFrame`], and the gesture machine sees an
idle frame. Nothing is suppressed, because nothing was offered — the same
shape as the hand tool's fix in [`super::interact`], where the gesture is
not offered at all rather than checked for and undone, because that is the
only version that cannot leave a half-applied selection behind.

The ruler-started drag has the same property for free: a gutter is outside
the scroll area and outside every page widget, so a press there was never
the canvas's to begin with.

### Why the in-flight drag is read from raw pointer input rather than from
a `Response`

Because the two entry points must resolve identically, and because a
`Response` is keyed on an `egui::Id` that encodes the guide's index — and
the index moves when a guide is added or removed. Reading `primary_released`
from the input state once the drag has started makes the release path one
function that neither entry point can diverge from, and makes it immune to
the widget the drag started on disappearing mid-gesture.

**Escape does not cancel a guide drag, and that is stated rather than
hidden.** `canvas::keys` owns Escape's precedence between the gesture
machine and the selection ladder, and a third claimant would need a rule
there. Releasing over a ruler or over the grey already cancels, which is
the peer convention, so the gap costs the operator nothing they cannot do
another way.

---

## 4. Rule 4

A guide is a **pre-commit affordance in `overlay`'s second category** — the
cursor, describing where the operator has decided something belongs. It is
not keyed on any property of the content, it is placed by hand rather than
inferred, it changes nothing a save would write, and it disappears the
instant `view.guides` is switched off. `overlay`'s one-line test — *would a
screenshot of the editing canvas differ from a screenshot of the same
document saved and reopened?* — answers **yes, because the operator asked
for it**, which is the answer rule 4 admits. The version that would fail is
a guide pdfcer placed *itself*, on a margin or a frame it detected. There is
no such code path and there must not be.

## Item notes

### `const CATCH_PTS`

A **screen**-space radius, like [`super::mapping::SELECT_SCREEN_TOLERANCE_PX`]
and for the identical reason recorded there: a page-space catch radius is
`radius × zoom` pixels on screen, so a guide that is easy to grab at 100 %
is un-grabbable at 25 % — exactly the zoom an operator uses to see a whole
sheet.

Deliberately smaller than the 6-point selection radius. A guide is a line
the operator can see, so aiming at it is easy; and the cost of a miss is
asymmetric — missing a guide starts a marquee the operator can abandon with
Escape, while catching a guide the operator did not aim at moves something
they had positioned deliberately.

### `const GUIDE_ALPHA`

High enough to read over dense linework — a guide the operator cannot see
on a CAD sheet is a guide that is not there — and short of opaque, because
a guide crosses the whole page and an opaque line would compete with the
drawing along its entire length. The same trade `overlay::GHOST_ALPHA`
records for the move ghost, at the same order of magnitude.

### `const DISCARD_ALPHA`

Half again fainter than a placed guide, and that difference *is* the
feedback: while the pointer is over the grey or over a ruler, the line the
operator is dragging says "release here and this does not happen". No
second colour, no second shape, no wording — emphasis, which is the same
answer `overlay`'s current-find-hit reached when it could not have a second
hue.

### `const DRAG_KEY`

In `Memory` for the reason `canvas`'s `GESTURE_MEMORY_KEY` states and not
the one the selection was moved off it for: a drag that is happening *right
now* is genuinely frame-local UI state with no meaning across a document,
and keying it here means a document change starts the next frame with no
drag in flight, by construction. What the drag *produces* is a
[`Action::SetGuides`] applied after the frame, through the one funnel.

### `fn segment`

Across the **page**, not across the viewport, and that is the visual
half of "a guide belongs to a sheet": a line that ran on into the grey
would look like a property of the window.

### `fn band`

[`CATCH_PTS`] either side of the line, and no further along it than the
page goes — so a guide cannot be grabbed from the grey beside its own
sheet, which under a continuous mode would mean grabbing a guide the
operator cannot see.

### `fn decode`

Dropping rather than failing, for the reason the module header gives:
every malformed state means the same thing to the caller — there is no
guide there to restore — and a preference is not worth an error path.

### `fn parse`

The payload is everything before the **first** tab and the path is the
whole remainder, which is what lets a path contain spaces. A line with no
tab, or with an empty path, is dropped.

### `fn absolute`

**`std::path::absolute`, and not `std::fs::canonicalize`** — the same
normalisation [`crate::viewer::remembered`] uses, and it has to stay the
same or the two stores would disagree about whether two spellings name one
document.

The difference is not cosmetic. `canonicalize` touches the filesystem, so
it **fails on a path that does not exist** — and on Windows it returns the
verbatim `\\?\D:\…` form, which is a line no operator opening
`guides.txt` would recognise as their drawing, and one that would not match
the same document's entry in `page-display.txt`.

### `struct Drag`

`Copy` and three small fields, held in [`egui::Memory`] between frames. See
the module header §3 for why the *release* is read from raw pointer input
rather than from the `Response` the press came from.

### `fn release`

**The one release path**, shared by both entry points, and the whole of the
create / move / delete rule:

| drag started as | released over a page | released anywhere else |
|---|---|---|
| new (from a ruler) | the guide is created there | nothing happens |
| an existing guide | it moves there, page included | it is **deleted** |

Raises at most one [`Action::SetGuides`] carrying the whole next
collection. One action rather than three verbs because the operand is
small, because the apply then has exactly one thing to persist, and because
"compute the next value from the previous one and hand it over" is the same
shape the canvas already uses for the selection.

### `fn cursor`

A resize cursor rather than a move cursor, and the pair of them rather than
one: a two-headed arrow across the guide says *this slides that way*, which
is the whole of what a guide drag does. `Grabbing` would say "pan", which
is what the middle button and the hand tool already mean on this canvas.

### `fn preview`

Across the page it would land on, at full strength — or across the whole
viewport at [`DISCARD_ALPHA`] when it would land nowhere, which is how the
line says *release here and this does not happen*. See [`DISCARD_ALPHA`] on
why the difference is emphasis rather than a second colour.

### `fn every_guide_round_trips_through_its_on_disk_spelling`

Negative is not an edge case invented for the test: canvas space has
its origin at the page's top-left, and a guide can legitimately sit
above or left of the sheet — the operator dragged it into the bleed.

### `fn a_corrupt_payload_drops_only_the_guides_it_breaks`

The posture the module header commits to, asserted token by token:
each of these is a different way a line can be wrong, and every one of
them must cost exactly the guide it describes.

### `fn a_non_finite_guide_is_refused`

It cannot come from a drag — the pointer is finite — but it can come
from a hand-edited file, and a NaN guide is a line that paints nothing
and a band that catches nothing: present in the count, absent from the
canvas.

### `fn a_guide_holds_still_on_the_page_at_every_zoom`

The property `GUI_ROADMAP.md` names for the selection — identity, not
position — applied to a guide: the stored value is identical at every
zoom, and the *screen* line it produces tracks the page. A guide stored
in screen coordinates would pass no part of this.

### `fn the_catch_band_is_the_same_width_at_every_zoom`

The law `canvas::mapping` exists to enforce, applied to a guide: a
page-space catch radius would be un-grabbable at exactly the zoom an
operator uses to see a whole sheet.

### `fn guides_survive_a_reopen_and_stay_with_their_own_document`

The whole of `PLANNED`'s *"need a per-document store to survive a
reopen"*, driven through the real reader and writer against a real
file.

### `fn a_path_with_spaces_survives_the_format`

Not hypothetical: every Windows operator has
`C:\Users\<name>\My Documents`, and a drawing office names job folders
after the job.
