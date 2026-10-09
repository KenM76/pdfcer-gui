# `app::dispatch::arrange` — the eight commands whose subject is a mark's
or a page object's DEPTH

Page objects selected at the Object rung, in a mode that edits content, go to
[`crate::app::actions::restack`]; otherwise a selected markup is moved in
`/Annots`. The chords are bound to the `markup.*` ids and reach both, so one
key means *move what I selected*. An `edit.*` id with no page objects
selected traces `command-declined reason=no-page-objects-selected`; the
commands are disabled then, so only a stale press reaches it.

Bring to front, Bring forward, Send backward, Send to back. Every drawing
program has them; this one had the engine verb, a test for it, three written
disclosures about it — and no way for the operator to reach any of it.

## The capability was present, tested, disclosed, and unreachable


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

## Why this is a module and not four arms in [`super`]

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

## What is deliberately NOT decided here

**The permutation.** This module resolves *which mark* and *which end* and
raises an action; the array itself is read at apply time by
[`crate::app::actions::reorder::arrange`], whose header carries the whole
argument. An order computed here would describe the `/Annots` the page had
before every action queued ahead of it was applied, and the engine refuses a
stale permutation by name rather than applying it approximately.

## Item notes

### `fn restackable_objects`

The one test the dispatcher, the `selection.objects_restackable` condition
and the canvas menu share: the selection's first page, page objects only. A
part of a placed drawing has no restack verb and yields `None`.

### `fn destination`

**`markup.` and not `arrange.`**, and the reason is a registry invariant
rather than taste. `shell::commands::tests::every_handler_token_is_in_its_
tabs_block` asserts that a command's handler token sits inside the hundred
belonging to its id's prefix — `markup.` is 500-599 — and it panics by name
for a prefix it does not know. A new `arrange.` prefix would therefore have
meant editing that table, in a file three other tracks are editing today, to
express a fact that is already true: **these are Markup-tab commands.**
*Arrange* is the name of the group they sit in, not of a tab.

### `fn the_predicate_and_the_mapping_are_one_statement`

The property [`claims`]' doc rests on: the predicate and the mapping are
one statement, so an id that answers `true` here and `None` there is
unrepresentable rather than merely unlikely.

### `fn no_two_commands_mean_the_same_end`

A copy-paste that gave two ids one destination would be silent: both
presses would produce a legal permutation, and only an operator watching
the wrong mark come forward would notice.

### `fn bring_to_front_means_the_end_of_the_array`

The one place the label and the array end are related, and the place a
reader thinking of `/Annots` as a list will get it backwards. `Front`
is the **last** entry, because §12.5.6 paints in array order.

### `fn claims`

Named `claims` rather than `handles` for [`super::markupnodes::claims`]'
stated reason: `shell::commands::reach::guards::EVALUATED_GUARDS` is a set of
**function names** read out of `dispatch.rs`'s syntax tree, `claims` is
already in it, and the register's own note blesses two guards sharing a name.
One line in `guard_claiming` and no change to that list.

Paired with [`destination`] rather than with a second `match` in
[`dispatch`], which is `dispatch::routes`' improvement on the
membership-test shape: the two statements that could grow apart are **one
statement**, so an id this predicate claims and that function cannot place is
unrepresentable.

### `fn dispatch`

# Three gates, and only one of them can be met by an operator who did
nothing wrong

| gate | reachable from the ribbon? | how it reports |
|---|---|---|
| the mode may not author markup | **no** — the Markup tab is not shown in Read | trace only |
| no markup is selected | **no** — the group is not drawn (`selection.markup_restylable`) | trace only |
| the mark is **locked** | **yes** — the group is drawn and the mark is selected | a sentence on the status row |

The first two are belt to the ribbon's braces: a customized manifest or a
chord reaches any command from any state, so they are written rather than
assumed — the same *"push the chord blind, gate the effect in dispatch"* rule
every arm in `super` follows. Neither owes the operator a sentence, because
neither can happen to one.

The **lock** can, and does: `selection.markup_restylable` deliberately
excludes the lock (§12.5.3 bit 8 is a fact about one annotation, not about
the build or the mode), so the four controls are live on a locked mark and
pressing one has to say why it did nothing.
