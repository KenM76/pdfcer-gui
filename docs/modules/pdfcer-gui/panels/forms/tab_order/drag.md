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
