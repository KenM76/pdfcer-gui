# `app::documents` — more than one document open at once

The operator's request, verbatim:

> *"make it so we can open multiple PDFs at once and drag and drop pages
> from one thumbnail image sidebar to another or onto the canvas to add
> pages and insert them in between the pages we've dragged to on the canvas
> or the thumbnail preview area."*

This file is the first half of that: **what it means for several documents
to be open**. The tab strip that shows them is [`crate::app::doctabs`], the
page drag between them is [`crate::panels::pages`], and the drop onto the
page view is [`crate::canvas::pagedrop`].

---

## 1. The shape: one active document, and the rest parked

[`PdfcerApp::status`] is unchanged and still means *the document the
operator is looking at*. Everything else — every panel, the canvas, the
status bar, the ribbon's condition set, the find bar — reads that one field
and did not have to change. Beside it now sits `PdfcerApp::parked`: the
other open documents, in tab order with the active one removed, and
`PdfcerApp::active_slot`, the position the active document occupies in that
order.

So the operator's tab strip, left to right, is

```text
parked[0] … parked[active_slot-1]   status        parked[active_slot] … parked[n-1]
 slot 0        slot active_slot-1   slot           slot active_slot+1     slot n
                                    active_slot
```

### Why this and not `Vec<Status>` with an index

Because the alternative costs a hundred edits to buy nothing. `self.status`
is named across the crate, and a large number of those sites are **split
borrows** — `let Status::Open(doc) = &mut self.status` taken in the same
expression as `&self.find`, `&mut self.dialogs`, `&self.commands`. Rust
splits borrows of struct *fields*; it does not split the borrow a
`fn active_mut(&mut self)` accessor takes. Replacing the field with a
method turns every one of those sites into a borrow error to be worked
around individually, and a borrow-checker workaround written a hundred
times is a hundred chances to change behaviour by accident. Parking the
other documents in a sibling field is also the growth
[`crate::app::PdfcerApp`]'s own header describes.

The one thing it costs is that the tab order is expressed in two fields
rather than one, so every reordering operation goes through
[`PdfcerApp::take_slots`] / [`PdfcerApp::put_slots`], which flatten to a
single `Vec` and rebuild. Those two functions are the only code in the
application that knows the encoding, and they are a handful of lines each.

---

## 2. What counts as a tab

**Every [`Status`] except [`Status::Empty`].** A file that failed to open,
one pdfcer does not support, and one waiting for a password each get a tab
that says so — the same way a browser tab survives a failed page load. The
alternative (only `Status::Open` gets a tab) would mean an operator who
opened four files and had one fail would find themselves looking at an
error with no way back to the other three, because the error would have
replaced whatever was active.

[`Status::Empty`] is therefore not a document but the **absence of all
documents**, and the invariant that makes the encoding total is:

> If `parked` is non-empty, `status` is not [`Status::Empty`].

[`PdfcerApp::document_count`] is the one predicate that reads it, and it is
what every other function here asks rather than testing the fields.

---

## 3. Opening the same file twice activates the tab it is already in

Acrobat, Word, VS Code and every browser do this, so pdfcer does. The
alternative is two tabs over one path, two independent `EditSession`s, two
undo stacks, and a save from either silently discarding the other's work —
which is a correctness problem wearing a usability problem's clothes.

Matched on the path as stored, which is already absolutised for anything
that came through the recent list. A created document
([`crate::app::state::Origin::Created`]) never matches, because its path is
a *name* rather than a location and two `Untitled 2.pdf` documents cannot
exist anyway — the counter never repeats within a session.

---

## 4. What switching documents forgets, and what it must not

Exactly what [`PdfcerApp::close_document`] forgets, minus the document:

| forgotten | why |
|---|---|
| the panels' view state | expansion sets and the Properties focus are **paint-order indices** into one page of one revision. Carried to another document they name different objects, confidently. |
| the find hits | a hit is a page index and a page-space rectangle. The epoch test that catches an *edit* cannot catch a *different document* — the other one's `edit_epoch` may match by coincidence. |
| the de-duplicated trace slots | so the document switched *to* re-declares its canvas line and its regions instead of inheriting them because the numbers happened to agree. |

And what it must **not** forget, which is the part that makes tabs worth
having at all:

- **The parked document's view.** Its page, zoom, scroll, fit and overlay
  state live on its own `OpenDoc` and are simply moved aside. Coming back
  to a tab puts you where you left it. This is why switching does **not**
  call `Prefs::seed_view` the way [`PdfcerApp::open_path`] does — that seeds
  a *new* document from the opening preferences, and applying it here would
  throw away the operator's place every time they glanced at another sheet.
- **The parked document's rasters.** A parked `OpenDoc` keeps its page
  texture and its strip cache. That is memory spent deliberately: a
  full-page render of the benchmark CAD drawing costs the better part of a
  second (`BENCHMARK.md` carries the measurement), so dropping the texture
  on park would make every tab switch a visible stall — which is the one
  thing a tab strip promises not to be. If this ever needs bounding it
  should be bounded by a *count of
  parked documents that keep rasters*, not by dropping them all.
- **The recent list, the dock arrangement and the mode**, for
  [`PdfcerApp::close_document`]'s reasons, unchanged.

---

## 5. Closing

[`PdfcerApp::close_slot`] removes one tab. Which tab becomes active
afterwards is the browser rule, because every operator already has it:

- closed a tab **left** of the active one → the same document stays active
  (its index shifts down by one)
- closed a tab **right** of it → unchanged entirely
- closed **the active** one → the tab that was to its right takes its
  place, or the new last tab if it was the rightmost

Closing the last document leaves [`Status::Empty`], which is exactly where
the application starts, so nothing downstream needs a second empty state.

**The unsaved-edits question is asked by the caller, not here.** See
[`crate::app::actions::document`], whose header carries the guard table and
whose test enumerates the arms that must ask. This module moves documents
around; it does not decide whether the operator meant it.

## Item notes

### `fn take_slots`

Half of the only code that knows the encoding. Always paired with
[`Self::put_slots`] inside the same function — leaving the application
in the state this returns would show an empty shell with the documents
still alive on the stack.

Returns an **empty** vector when nothing is open, rather than
`[Status::Empty]`: the caller wants a list of documents, and a list
containing "no documents" is the bug this early return removes.

### `fn put_slots`

The other half. An empty vector is the legitimate way to say *"nothing
is open now"* and restores [`Status::Empty`] — which is what makes
closing the last tab need no special case anywhere else.

`active` is clamped rather than asserted. Every caller computes it from
a length that has just changed, and an off-by-one there should land the
operator on the last tab rather than panic in the middle of a close.

### `fn forget_previous_documents_view`

§4's table, as three statements. Deliberately the same three
[`PdfcerApp::close_document`] makes, and deliberately *not* `adopt` —
see §4 on why re-seeding the view would be wrong here.

### `fn tab`

Using `Failed` rather than `Open` is not a shortcut around the real
type — §2 makes a failed open a first-class tab, so this *is* one of
the states the encoding has to carry, and the tests below are testing
the tab arithmetic rather than anything about documents.

### `fn the_strip_order_is_independent_of_which_tab_is_active`

The property that makes `parked` + `active_slot` safe: whichever tab is
active, the strip reads the same left to right. A naive encoding that
pushed the outgoing document onto the end of `parked` would pass with
`active_slot == 2` and reorder the operator's tabs on any other.

### `fn reordering_tabs_never_changes_which_document_is_on_screen`

The property that makes `move_slot` correct and the one a naive
implementation gets wrong: dragging a tab is tidying, not navigation, so
the active document has to follow its own tab through the permutation
rather than staying at an index.

Swept across **every** `(from, gap)` pair on a four-tab strip with each
of the four active in turn — 4 x 5 x 4 = 80 cases — rather than spot
checked, because the arithmetic has two adjustments that compose and the
composition is where an off-by-one hides. Three hand-picked cases would
very likely all miss it.

### `fn dropping_a_tab_where_it_already_is_does_nothing`

`gap == from` is *before itself* and `gap == from + 1` is *after
itself*; a strip that treated the second as a real move would shuffle
the document one place every time an operator picked a tab up and put it
back.
