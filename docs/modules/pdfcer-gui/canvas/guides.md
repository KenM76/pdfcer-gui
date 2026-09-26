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
