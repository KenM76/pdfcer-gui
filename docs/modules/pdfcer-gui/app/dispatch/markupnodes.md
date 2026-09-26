# `app::dispatch::markupnodes` — the three commands about a markup shape's
**points**

## The seam, and it is a subject rather than a size

What these ids share is not the `markup.` prefix — `markup.cloud` arms a
pen, `markup.stamp` arms a different one, `markup.comments` opens a panel,
and none of those is here. What they share is their **operand: a point on a
shape.**

| id | the point it is about |
|---|---|
| `markup.finish` | the run of points the operator has been clicking out — *stop here* |
| `markup.add_node` | a place on one edge of a drawn shape — *put a corner there* |
| `markup.remove_node` | one existing corner — *take it away* |

`markup.finish` belongs here rather than beside the other `markup.*` arms:
it is the ribbon half of the vertex tools' ending, which is a statement
about points, and an operator who has just used it is one keystroke from
wanting the two below it.

⇒ Together they form the whole answer to one operator sentence, which is
this project's usual test for a module:

> *"I also can't edit or delete nodes of a markup shape once it is drawn."*

## The two node commands do NOT require the Points tool armed

The chord route does — `Ctrl` and `Ctrl+Shift` over a node, with
`view.tool_node` armed — because `Ctrl` already means *take this out of the
selection* everywhere on this canvas, so arming the tool whose subject is
points is what says the operator meant it. A **menu row is unambiguous by
construction**: they read *"Add a point here"* and chose it. Requiring an
armed tool as well would be carrying a rule past the reason that produced
it. [`crate::canvas::annotnodes::menu`]'s header carries the full argument;
there is deliberately no tool check anywhere below.

## What is NOT decided here

**Whether the edit is allowed.** That is the engine's, asked through
`EditSession::reshape_annotation_preview` inside
[`crate::canvas::annotnodes::menu::action_for`], which is also where the
parked pick — *which* corner, *which* edge — is read. This module reads one
published capability, calls that function once, and pushes what comes back.
A second copy of the engine's subtype matrix here is exactly what
`canvas::annotnodes`' header refuses by name.

## The capability gate is one sentence, asked once

`author_markup`, for every id here, and they must decline **alike**: a mode that
may not place a shape has no business finishing one, adding a corner to one
or taking one away. Three different refusals for one capability would read
as arbitrary. Each traces separately all the same, because *"the mode says
no"* and *"there was nothing to act on"* are different facts with different
answers and a reader of a trace from a machine they cannot see should not
have to guess which nothing happened.

## Item notes

### `fn claims`

Named `claims` rather than `handles` because
`shell::commands::reach::guards::EVALUATED_GUARDS` is a set of **function
names** read out of `dispatch.rs`'s syntax tree and asserted equal to the
set the reachability checker evaluates. A guard function whose name is not
in that set is a place commands can hide from the check that exists to find
them, so a new module reuses an evaluated name rather than inventing one.

A predicate paired with [`dispatch`] over the same ids, which is two
statements of one set — the shape this crate usually refuses. It is accepted
here only because the two sit adjacent in one small file. **Nothing
mechanical welds them**: the `match` below ends in a `_ => {}`, so an id
added here and not there is a control that does nothing and says nothing.
Add to both, in the same edit.
