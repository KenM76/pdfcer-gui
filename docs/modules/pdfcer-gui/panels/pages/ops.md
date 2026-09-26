# `panels::pages::ops` — what a page verb acts on, and what a move means

The **rules** behind the six page verbs, with no `egui`, no document and no
engine call anywhere in the file. Two questions, each with exactly one
right answer:

1. **What is the operand?** — [`operands`]. The Pages panel's multi-select
   when there is one, the current page when there is not.
2. **What does "move up" mean for a set?** — [`move_order`]. A permutation
   of `0..page_count`, or a refusal, and the refusal is the interesting half.

## Why this is a module and not two `if`s in a dispatch arm

`crate::app::dispatch`'s header states the standing rule — *"the arms route;
they do not compute"* — and both of these are computations that can be wrong
in ways an operator would notice and a compiler would not. The move
permutation in particular is the kind of small index arithmetic that looks
obviously right and is off by one at the boundary: `EditSession::reorder_pages`
refuses anything that is not a permutation of `0..count`, so a bug here is a
verb that silently declines rather than one that mis-orders, which is the
quieter and therefore worse failure.

Pure, so every one of those boundaries is asserted headlessly below. That is
`crate::viewer`'s standing split — *"this module is unit-testable and the
widget code is not"* — applied to the one part of a page verb that carries a
rule.

## The operand rule, and why it is not "the selection"

[`crate::panels::PanelsState::selected_pages`]' own documentation already
settled this before anything read it:

> Empty is a defined answer, not a missing one: with nothing picked those
> commands act on the current page.

and `crate::shell::commands`' Pages band says the same from the registry's
side — the verbs *"respect the thumbnail rail's selection when there is
one"*, and with none they *"act on the current page, which is a defined
answer and not a disabled state."* That is why none of the `pages.*`
commands is gated on a selection condition and why adding one would be
wrong: a rotate with nothing picked is not a refused command, it is a
rotate of the sheet the operator is looking at.

[`operands`] is the single place that rule is written down, so the six verbs
cannot come to disagree about what they act on — the same argument
`crate::canvas::selection::SelectionState::deletable_objects_on` makes for
the object verbs, and it is made here for the same reason: two statements of
a destructive rule is one too many.

## The move rule, in one table

`pages` is the operand list, `n` the page count, and the result is what
`new_order[i] = ` for each new position `i`.

| operand | `n` | up | down |
|---|---|---|---|
| `{1}` | 4 | `[1,0,2,3]` | `[0,2,1,3]` |
| `{1,2}` | 4 | `[1,2,0,3]` | `[0,3,1,2]` |
| `{1,3}` | 4 | `[1,0,3,2]` | `[0,2,1,3]` |
| `{0}` | 4 | **refused** | `[1,0,2,3]` |
| `{0,1}` | 4 | **refused** | `[2,0,1,3]` |
| anything | 1 | **refused** | **refused** |

A **non-contiguous** selection moves as separate items, each by one, rather
than being collapsed into a block. That is what every list control with
reorder arrows does, and the alternative — gathering the set together at the
topmost member — would silently reorder pages the operator did not name.

## Why a blocked move is a refusal rather than a no-op

`EditSession::reorder_pages` would *accept* the identity permutation and
return `Ok(())` having recorded nothing, so handing it one would be
harmless. It would also be silent, and a control the operator pressed that
produced no change and no sentence is the defect class this project is named
after. So [`move_order`] returns [`Err`] for a move that cannot happen, and
the engine is never asked a question whose answer is "nothing".

The two blocked cases are different facts and are reported as two, because
the remedy differs: *these sheets are already at the top* is a boundary the
operator can fix by picking a different one, while *there is only one page*
is about the document and cannot be fixed at all.

## What the caller does with a refusal today, and what it should do

`crate::app::dispatch` **traces** it, with the variant's name as the reason
token — `command-declined id=pages.move_up reason=at-the-edge` — and does
not word it in the status bar.

That is a scope statement rather than a judgement that it should stay so.
The surface for a worded decline is `crate::app::status::decline`, whose
`Declined` enum was being extended by concurrent undo/redo work while this
landed; adding variants to a type another session is mid-rewrite on is how
two sessions produce one broken file. The two refusals carry **distinct**
tokens precisely so that follow-up is a mapping rather than an
investigation.

The one thing to know before taking it: `Declined::still_true` re-asks the
predicate that produced a decline, and the predicate here — *are the picked
sheets at the edge?* — depends on the **Pages panel's** state, which lives
on `PdfcerApp::panels` and not on the `&OpenDoc` that function is handed. So
either the sentence retires only on the operator's next *command* (which is
`Declined::SaveFailed`'s documented answer, and leaves the sentence stale
after a mere click on a different thumbnail), or `live`'s plumbing grows a
third source. That is a real decision, not a line of wiring, which is the
other half of why it is not taken here.

## Item notes

### `fn landing`

`drop_order` returns the engine's permutation — *which page is at
position i* — and every assertion below is easier to read as the page
sequence itself, which is what an operator sees in the grid.

### `fn a_block_dragged_forward_lands_at_the_boundary_that_was_pointed_at`

The case that catches an off-by-one: dragging pages 0–1 to gap 4 in a
five-page document. Naively splicing at 4 into the three remaining
pages would put them after page 4, not before it. The lift-count
correction is what makes the answer `[2, 3, 0, 1, 4]`.

### `fn gathering_scattered_pages_at_their_own_first_page_is_not_a_no_op`

Pages 0, 4 and 8 dropped at gap 0 become adjacent at the top. A
predicate that only compared the drop against the block's endpoints
would call this a no-op, because gap 0 is the lower lip of the
selection's range — and the operator would drag, release, and watch
nothing happen.

### `fn a_block_dropped_on_itself_is_refused_at_every_boundary_it_spans`

The engine would accept the identity permutation and write an undo
entry for it, so an operator who picked a page up and put it back would
get a document marked as edited and a `Ctrl+Z` that changes nothing
they can see.

### `fn every_landing_is_a_permutation`

Swept over every operand set and every gap in a small document rather
than spot-checked: the failure this guards against is not one wrong
answer, it is a whole family of drags producing a vector the engine
declines for a reason no operator could act on.

### `fn the_moved_pages_arrive_adjacent_and_in_document_order`

A drag says *"put these here"*, and "these" is a set the operator built
by clicking. Re-ordering them relative to each other would be the panel
inventing an intention; leaving gaps between them would make the drag
do half of what it looks like it does.

### `fn with_nothing_picked_the_operand_is_the_current_page`

The rule `PanelsState::selected_pages` states in words — *"Empty is a
defined answer, not a missing one"* — as a mechanism. A build that
returned an empty operand list here would make every ribbon Pages
control do nothing until the operator discovered the panel, which is the
exact "live control, no effect" defect this work exists to close.

### `fn a_stale_pick_is_dropped_rather_than_refusing_the_whole_batch`

`EditSession::delete_pages` resolves **every** index before planning
anything and returns `PageOutOfRange` for the whole batch if one is bad,
so a stale pick would turn "delete these three" into "delete nothing,
silently". Reachable by chord: the panel clamps on its next frame and a
keyboard verb can arrive before that frame is drawn.

### `fn a_contiguous_run_moves_as_a_run`

The property a naive "swap each with its neighbour" loop gets wrong: run
it ascending without the ceiling and pages 1 and 2 swap with each other
twice and end up back where they started. This asserts the *magnitude*
as well as the direction — the run really is one place earlier —
because a test that only checks the direction is satisfied by a loop
that moves the run twice as far.

### `fn a_non_contiguous_pick_moves_each_item_by_one`

The alternative — gathering the set at its topmost member — would
reorder pages the operator never named, which is the same class of
error `select::PageSelection::right_click` exists to prevent from the
pointer's side.

### `fn the_top_of_the_document_refuses_a_move_up`

`reorder_pages` returns `Ok(())` for the identity, having recorded
nothing. Handing it one would be a control the operator pressed that
changed nothing and said nothing — which is the defect this project is
named after, so the refusal is the result rather than a detail.

### `fn a_partly_blocked_run_moves_the_part_that_can`

Pages 1 and 3 picked, moved up: page 1 is pinned, page 3 is not. The
alternative — refusing the whole gesture because one member is at the
edge — would make a large selection increasingly hard to move, which is
the opposite of what a reorder control is for.

### `fn every_order_is_a_permutation_and_moves_by_exactly_one`

The one property `EditSession::reorder_pages` checks and refuses over,
and therefore the one whose failure would be a verb that declines with
nothing an operator could do about it. Asserted exhaustively over every
non-empty subset of a five-page document, in both directions — 62 cases,
which is cheap and is the difference between "the examples above work"
and "the rule is sound".
