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

### `fn reorder_annotations`

# Three disclosures, and two of them are about things the operator did
not ask for

A tab order is a list of *fields*. `/Annots` order is more than that, and the
engine reports the difference rather than letting it happen quietly:

* **`non_widgets_moved`** — `/Annots` order is **paint order** for every
  annotation, so moving a widget past a `/Link` or a markup changes which is
  drawn on top where they overlap. The operator arranged a tab order and got
  a z-order change; that has to be said.
* **`pinned`** — entries written as direct dictionaries have no object id to
  be named by, so they cannot be moved and stay at their index while the rest
  flow around them. A list that did not fully take, disclosed rather than
  discovered.
* **`array_copied`** — the page's `/Annots` was shared with another page and
  had to be copied first. Nothing is wrong, and it is a structural change to
  the file that nobody asked for.

`moved == 0` is a **success with nothing to say**: the order given was the
order the page already had, the engine recorded no command, and there is
nothing to disclose. It is the common case for a drag that ends where it
started, and it must not read as a refusal.

### `enum ArrangeTo`

# Why all four, when only two were asked for

The brief for this work said to ship the two ends *"and consider Bring
forward / Send backward too if the verb supports a single-step move cheaply;
if it only takes a whole array, say so and ship the two that are honest."*

`EditSession::reorder_annotations` **takes a whole array** — and that is
exactly what makes the single step cheap rather than what forbids it. A
one-place move is a permutation like any other: the same list with two
entries exchanged. There is no per-step verb to be missing, no second engine
call, and no partial support to disclose. What a whole-array verb would have
made expensive is the *opposite* pair — a move that could not name every
entry — and this shell has to name them all anyway, because the engine
refuses a list that is not a permutation of the page's indirect entries
(`AnnotsNotAPermutation`) rather than silently dropping the ones a caller
forgot.

⇒ So all four ship, and the honest statement is the one in this paragraph:
nothing about the single step is approximated.

### `fn arrange`

# The order is computed HERE, at apply time, and not at the press

The obvious arrangement is for the dispatcher — which has the selection and
the document in front of it — to work out the new array and put it on the
action. It is wrong, and the reason is the action queue itself: an action is
raised on one frame and drained on another, with every action queued ahead of
it applied first. A permutation computed at the press is a permutation of the
`/Annots` the page had **before** whatever ran in between, and the engine
refuses a stale one by name (`AnnotsNotAPermutation`) rather than applying it
approximately.

So the action carries the **intent** — this mark, that end — and the array is
read from the revision the edit is actually applied to. That is the same rule
[`crate::app::actions::annot::AnnotAction::Move`] follows by carrying a delta
rather than a rectangle, and for the same reason: *a value resolved at the
press is a value that may have moved under you.*

# What is held still, and it is not the operator's choice

A `/TrapNet` annotation **shall be the last element** of `/Annots`
(ISO 32000-1 §12.5.6.21, restated §14.11.6.2 — the trap network prints after
everything else). The engine enforces it with `TrapNetMustStayLast` for any
list that tries to move it. This shell never builds such a list: the trap
network is lifted out of the permutation, everything else is arranged, and it
is put back on the end.

⇒ A *Bring to front* on such a page therefore puts the mark in front of
everything the operator can see and behind one thing they cannot, and
[`crate::text::arrange::trap_net_stays_last`] says so. Saying nothing would
leave a command that visibly worked and technically did not.

**Entries with no object id are not listed at all**, which is how a caller
asks the engine to pin them — they are written into the page as direct
dictionaries, nothing can name them, and they keep the index they had while
the rest flow around them. That is a list which *did not fully take*, and it
is disclosed rather than discovered.
