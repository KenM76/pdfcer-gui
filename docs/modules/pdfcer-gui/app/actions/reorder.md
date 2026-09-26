# `app::actions::reorder` — putting a page's annotations in a new order

One verb, its own file under R2 rather than a section of
[`super::forms`]. `OPERATOR_REQUESTS.md` O99.

## Why it is worth its own file rather than a shorter comment

Because the interesting part is not the call — it is one line — but the
**three things the operator did not ask for** and which the engine reports so
they can be said. A tab order is a list of *fields*; `/Annots` order is more
than that, and every one of the three below is a consequence an operator
would not predict from the gesture they made.

## Two callers, one verb, and OPPOSITE surprises

The three disclosures below are written for the **form-field tab-order
panel**. [`arrange`] is the other caller, and it arrives from the other end
of the same array:

| the operator did | what they meant | what the engine reports | the surprise |
|---|---|---|---|
| dragged a row in the **tab-order** list | *this field comes second* | `non_widgets_moved` | the **drawing order** changed |
| pressed **Bring to front** on a mark | *draw this on top* | `moved - non_widgets_moved` | the **tab order** changed |

⇒ `/Annots` order is **two lists at once** — paint order for every
annotation and the tab sequence for the widgets among them — so whichever
one the operator was thinking about, the other moved. That is why the two
callers do not share a note set even though they share every line of the
call: the disclosure is *"the thing you were not looking at"*, and they were
looking at different things.

The second number is a **subtraction**, not a field. `AnnotsReorder`
reports `moved` and `non_widgets_moved`; how many widgets moved is
`moved - non_widgets_moved`, computed at the one call site that cares rather
than asked of the engine, because it is a fact about this caller's intent
rather than about the reorder.

## Item notes

### `fn front_is_the_end_of_the_array`

§12.5.6 paints annotations in `/Annots` order, so the last entry is drawn
last and therefore on top. "Front" and "last" are the same place, and a
reader who thinks of the array as a list will reach for the opposite
answer — which would compile, pass a review, and paint every *Bring to
front* underneath everything.

Falsified: swapping the `Front` and `Back` arms in [`plan`] leaves this
green and turns [`the_ends_of_the_array_are_the_ends_of_the_stack`] red,
which is the division of labour intended — this one pins the *vocabulary*
and that one pins the *arithmetic*.

### `fn the_four_destinations_are_distinct`

A guard against the copy-paste that gives two variants one arm — which in
this enum would be silent, because both would still produce a legal
permutation.

### `fn moved`

A free function mirroring [`plan`]'s middle, because `plan` needs an
`OpenDoc` and the thing worth pinning is the index maths. This is
**two statements of one rule**, which this crate usually refuses — it is
accepted here for `dispatch::routes`' stated reason, that the two sit in
one small file where a reader meets both at once, and because the
alternative is a rule with no test at all.

### `fn a_step_past_either_end_stays_where_it_is`

A *Bring forward* on the topmost mark and a *Send backward* on the
bottom-most both resolve to where they already are, which is what
[`plan`] then answers with a sentence rather than an edit. Without the
clamps, one is an out-of-range insert and the other is an underflow —
and `usize` underflow here would ask for index `usize::MAX`.

### `fn bring_to_front_puts_the_mark_last_in_the_file`

The oracle is `page_annotations` **after** the edit, which is not the
code under test: `plan` builds a list, `EditSession::reorder_annotations`
writes the array, and this reads what the array became. A `plan` that
returned the ends the wrong way round would satisfy every assertion in
the arithmetic tests above — they are a mirror of it — and fail here.

Falsified: swapping the `Front` and `Back` arms of [`plan`]'s `match`
turns both halves of this red, and leaves
[`the_ends_of_the_array_are_the_ends_of_the_stack`] green. That is the
point of writing both.

### `fn a_mark_already_at_the_front_is_told_so`

The sentence is the point. The engine reports `moved == 0` and
[`reorder_annotations`] calls that *"a success with nothing to say"* —
correctly, for a drag that ended where it started. A **command** is not a
drag: the operator pressed a labelled button on purpose, and a press that
neither moves anything nor says anything is indistinguishable from a
broken control.

Falsified: deleting the `if target == from` branch in [`plan`] turns
the second assertion red — the edit goes through the funnel, the engine
reports `moved == 0`, and the operator is told nothing.

### `fn an_id_this_page_does_not_list_plans_nothing`

Reachable after an undo or an external reload, when a selection names an
object the page no longer lists. Building a permutation that silently
omitted it would ask the engine to pin every entry it could not name,
which is a page whose annotations quietly stop being reorderable.
