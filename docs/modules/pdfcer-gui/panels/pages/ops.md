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
