# `app::dispatch::arrange` — the four commands whose subject is a mark's
DEPTH

Bring to front, Bring forward, Send backward, Send to back. Every drawing
program has them; this one had the engine verb, a test for it, three written
disclosures about it — and no way for the operator to reach any of it.

## ★★★ The capability was present, tested, disclosed, and unreachable


> `/Annots` order is **paint order** for every annotation, so moving a widget
> past a `/Link` or a markup changes which is drawn on top where they
> overlap.

That sentence is written as a *warning to somebody arranging a tab order*.
Read the other way round it is a **feature specification**, and the only
surface that could reach it was the form-field tab-order panel — a place
nobody looking to put a revision cloud on top of a highlight would ever open.

⇒ The shape is the one this project keeps finding, one rung along from
*"the capability was not in the binary"*: **the capability was in the binary,
reachable from exactly one surface, and that surface was about something
else.**

## ★★ Why this is a module and not four arms in [`super`]

Two reasons, and the second is the load-bearing one.

1. **Room.** `super` stood at 1,477 of R2's 1,500 lines when this was
   written — 23 to spend — and four arms with the argument each of them
   carries is not 23 lines. The gate's own header says the answer to a file
   approaching the limit is to find the seam, not to shorten the prose.
2. **They share one preamble and one refusal**, exactly as
   [`super::markupnodes`]' three do: the same capability, the same
   selection, the same lock. Four sibling arms would have been the same six
   lines of gate written four times, which is four places for the next
   change to be made in three of them.

## ★ What is deliberately NOT decided here

**The permutation.** This module resolves *which mark* and *which end* and
raises an action; the array itself is read at apply time by
[`crate::app::actions::reorder::arrange`], whose header carries the whole
argument. An order computed here would describe the `/Annots` the page had
before every action queued ahead of it was applied, and the engine refuses a
stale permutation by name rather than applying it approximately.
