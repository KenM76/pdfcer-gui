# `panels::forms::tab_order::drag` — reordering the tab list by dragging

`OPERATOR_REQUESTS.md` O99, and the operator named the reference himself:

> *"the tab order list is supposed to be able to be reordered by dragging
> and dropping rows around **like we can with pages in the page preview**,
> and have **clear markers** of where the field is going to move to."*

So this is deliberately [`crate::panels::pages`]' insertion caret, one panel
over: the same gap model (`0` is before the first row, `len` is after the
last), the same `Rect`-carrying [`DropTarget`], the same full-strength /
dimmed pair for *"releasing here changes nothing"*, and the same rule that a
release is read from raw input rather than from a `Response`. Two panels
that both mean *"it will go here"* and draw it two different ways is a
discoverability defect, and the second one is where it gets introduced.

## The one thing that is genuinely different, and it is the hard part

**A row is not an array entry.** The page rail reorders pages, and a page is
a page — the list the operator drags and the array the engine permutes are
the same sequence. Here they are not:

* The list holds **widgets a field claims**. The array holds those, plus
  unclaimed widgets, plus anonymous ones, plus every `/Link`, `/Text`, stamp
  and markup annotation on the page.
* `EditSession::reorder_annotations` takes *"the page's indirect `/Annots`
  entries, each once, in the wanted order"*. A list built from the rows is
  not that; it is a permutation of a **subset**, and the engine refuses it
  by name rather than quietly dropping the rest.

So [`reordered`] permutes the rows **through their slots**: the widget rows
move among the positions widget rows already occupied, and every other entry
keeps its index. See that function for why that is the right rule and not
merely the convenient one.

## Rule 4

The caret is the **cursor**, in the class the rule permits by name — *"snap
indicators, hover highlights, rubber-bands and selection handles are the
cursor and are welcome"*. It marks no content, tints no field, draws nothing
onto the canvas, and is gone the instant the pointer is released.

## Item notes

### `const CARET_PTS`

[`crate::panels::pages`]' `CARET_PTS` verbatim. The two panels draw the same
mark and an operator who has learnt one has learnt the other; a caret that
were a hair thinner here would read as a different, weaker kind of promise.

### `const CARET_DIMMED`

**Dimmed, not hidden** — the page rail's argument, unchanged and load
bearing: drawing nothing over a boundary that would not land cannot be told
apart from the panel having stopped tracking the pointer, and the no-op
boundary is where **every** drag begins, because a row starts out hovering
over its own slot.

### `fn an_annotation_that_is_not_a_row_never_moves`

Slots `0` and `2` are widget rows; slot `1` is something else. Dragging
the first row past the second must swap entries 0 and 2 and leave entry
1 exactly where it was — which is `non_widgets_moved == 0` at the
engine, by construction.

### `fn a_drag_whose_row_has_vanished_does_nothing`

Not defensive decoration: the rows are rebuilt from the document every
frame, and an edit landing between the frame a drag began on and the
frame it is released on can shorten the list under it. The honest answer
to "the row you were dragging is gone" is to do nothing.

### `struct Drag`

# Why it carries the page and not just the row

The list is per page and there are as many blocks as the document has pages.
A drag that began on page 3 must not be answered by page 4's block — which
is not hypothetical, because every block runs the same code in the same
frame and would otherwise all believe the drag was theirs.

Reordering **across** pages is deliberately not offered: moving a widget to
another page is a different edit (it changes which sheet the field is on,
not merely when it is reached) and `reorder_annotations` cannot express it.
A drag that leaves its own block simply finds no gap and lands nowhere.

`Default` is derived for one reason, and it is the same one
[`crate::pagedrag::PageDrag`] records: `egui::IdTypeMap::remove_temp`
demands it of anything it can take back out. A defaulted `Drag` — page 0,
row 0 — is never constructed here and is not a state the application can
reach; [`current`] answers `Option`, so "no drag" is `None` and never a
drag of the first row of the first page.

### `struct DropTarget`

[`crate::panels::pages`]' `DropTarget`, with the same three fields and the
same reasons: a gap has no position until the rows have been laid out, so it
is resolved during the layout pass and carried out; the caret is a `Rect`
because a line is two endpoints and the layout pass knows nothing about
colour or width; and `lands` is computed where the row set is in scope
because the paint pass no longer has it.

### `fn reordered`

`slots` are the rows' indices into `annots`, ascending — that is,
`page.rows.iter().map(|r| r.slot)`. `from` and `to_gap` are in **row**
space: `from` is the row being dragged, `to_gap` is the boundary it is
dropped at, where `0` is before the first row and `slots.len()` is after the
last.

# The rule: widgets move among widget slots; nothing else moves at all

The alternative — permuting the array wholesale, so a widget travels with
whatever entries happen to sit beside it — was considered and rejected. Two
reasons, and the second is the stronger:

1. **It is not what the gesture says.** The operator dragged a row in a list
   of form fields. Moving a `/Link` because a text box passed over it is a
   consequence of the implementation, not of the request.
2. **`/Annots` order is paint order.** Moving a non-widget changes which
   annotation is drawn on top where two overlap. That is a visible change to
   the rendered page, produced by a gesture whose entire subject was the
   order boxes are *reached in*. The engine discloses it
   (`non_widgets_moved`) precisely because it is surprising — and the right
   response to a surprising consequence you can avoid is to avoid it, and
   keep the disclosure for the cases you cannot.

So this route reports `non_widgets_moved == 0` on every call, by
construction. That is not the disclosure being dead code: it is this route
being the one that does not need it.

# Returns the whole array, not the changed part

Because that is what the verb takes. `annots` in, `annots` permuted out,
same length, same multiset — which is exactly the property the engine
validates and refuses (`AnnotsNotAPermutation`) rather than trusts.

# A drag that lands where it started returns the input unchanged

`to_gap == from` and `to_gap == from + 1` are both the row's own boundaries.
The output is then equal to the input, the engine's `moved` is `0`, and
nothing is disclosed — which is the common case for a drag an operator
thinks better of, and it must not read as a refusal.

### `fn lands`

Its own function, and named, because it is the *same* question
[`crate::panels::pages`]' `ops::drag_is_a_no_op` answers for pages, and
because it is what decides whether the caret is drawn at full strength or
dimmed. Getting it wrong in the dim direction makes a working drop look
refused; getting it wrong the other way promises an edit that will not
happen.

### `fn paint`

The colour is [`egui_shell::theme::Theme::canvas_selection_ink`] — the
theme's, never a literal, and the same source the page rail's caret, the
current-page ring and the canvas guide preview all take, so a preset that
changes the accent changes every one of them together. **Not
`visuals().selection.stroke`**: that is `egui`'s selected-*widget* channel,
which a preset is free to move independently, and a caret drawn from it
drifts away from every other selection mark in the application.

### `fn consider`

Called once per row during the layout pass, with the row's rectangle and the
pointer. The row's own midpoint splits it: above means *before* this row,
below means *after* it.

### `fn settle`

# Why the release is read from raw input

[`crate::panels::pages`]' `settle_drag`, and its reason applies here
unchanged: a drag that began on a row may end anywhere — over the page
heading, past the last row, outside the scroll area, or after the pointer
has left the window. A `Response` reports releases only inside the widget
that produced it, so a release elsewhere would strand the drag in flight
with a caret nobody could dismiss.

# A drag that lands nowhere raises NO action, and says so

Released over no gap, over its own boundary, or in another page's block: the
drag ends, the caret goes, and nothing is committed. Traced, because
"nothing happened" and "something happened that did nothing" are the two
readings a check has to be able to tell apart.
