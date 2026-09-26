# `canvas::moving` — dragging a selection, and the four things that make it honest

## What this module is for

A press inside the selection, a drag, a release: the object moves. That is
one sentence and four separate obligations, and this module exists because
each of them is a place the gesture can go quietly wrong.

1. **One gesture is ONE command.** A multi-select moves through
   `EditSession::move_objects`, which takes a *slice*, resolves and
   type-checks every index before planning anything, and refuses the whole
   call rather than moving the prefix that happened to qualify. Emitting one
   `move_object` per selected entry would be N undo entries for one drag and
   — worse — N content-stream re-splices, each planned against byte offsets
   the previous one already invalidated. `docs/core-api/02` states the rule
   in a box: *"Never loop the singular verbs over a selection."*
2. **The delta is PAGE space, never screen pixels.** See
   [`page_delta`] and the whole of [`crate::canvas::mapping`]'s header. A
   drag measured on screen and handed to a page-space verb compiles, runs,
   and merely scales with magnification — the same silent class as the
   hit-tolerance defect that module was built to make unavailable.
3. **The preview must describe something that will actually happen.**
   `D:\Dev\FeatureRequests\pdfce_FeatureRequests\README.md` rule 4 welcomes
   a pre-commit affordance — *"a snap indicator, a hover highlight, a
   rubber-band, a selection handle — these are the cursor; they describe
   what is about to happen"* — and forbids marking content that has already
   been applied. A ghost outline is squarely in the first category, **as
   long as the move it describes is one the engine will accept.** So the
   ghost is drawn only when [`eligible`] has already said yes; a ghost over
   a text object at the Part rung, where no `move_*` verb applies, would be
   what `overlay`'s own note calls *"a lie with a low alpha"*.
4. **The rung decides the verb, and a rung with no verb declines out loud.**
   Object → `move_objects`; Part → `move_subpath`; Node → `move_node`. The
   Part rung of a *text* object has no move verb at all (a show operator is
   not a subpath), so it refuses and traces, exactly as Delete refuses at
   the Part rung today.

## Why the selection needs no invalidation across a move

Because a move **does not renumber**. This was an open question that
blocked the whole feature, was asked as `request_stable_object_identity.md`,
and came back measured rather than asserted — the proof is
`crates/pdfcer-core/tests/object_identity_across_edits.rs`, which decomposes,
edits, and decomposes *again*:

| family | mechanism | renumbers? |
|---|---|---|
| `move_object` · `move_objects` · `move_subpath` · `move_node` · `move_nodes` · `move_handle` | rewrites operator **operands** in place | **NO** |
| `delete_object` · `delete_objects` · `delete_subpath` · `delete_node` · `delete_text_run` | excises byte **spans** | **YES** |

A move changes numbers *inside* existing operators. No operator is added or
removed, so a second decomposition walks the same operators in the same
order and yields the same objects at the same indices — asserted directly by
that test, and asserted to be non-vacuous (the moved object demonstrably
moved).

So [`crate::canvas::selection::Selection`] — `{ page, object, subpath, node }`
— survives a move **unchanged**, with no durable token and no invalidation
pass. What *does* change is the geometry, and that is already handled by the
machinery invariant 3 built for a delete: the action bumps
`OpenDoc::edit_epoch`, `SelectionState::needs_resolve` sees the key move,
and the outlines are recomputed from the fresh decomposition on the next
frame. [`tests::a_move_never_alters_the_selection`] pins both halves.

## What is deliberately NOT here: resize

`EditSession` has the entire `move_*` family and **no scale or resize verb
of any kind**. The eight grips are drawn, and a drag on one is *consumed*
(so it cannot fall through to a marquee and silently replace the selection
the operator was aiming at), and it commits nothing — see
[`crate::canvas::handles`]. Wiring a ghost to a resize grip would be an
affordance for something that cannot happen, which is the no-placeholders
invariant, and it is a separate change for the day the verb exists.

## The split between the pure rules and the wiring

[`eligible`], [`action`] and [`page_delta`] are pure functions of plain
data, so every rule above is testable with no window, no document and no
decomposition — the same discipline that makes
[`crate::canvas::selection::SelectionState::click`] a pure function of a
[`ClickHit`](crate::canvas::selection::ClickHit). [`drag`] is the one
function that touches the live provider, and it does nothing except gather
those inputs, call the pure functions in order, and trace what happened.

## Item notes

### `fn entered_entry`

Refuses an entry that belongs to a different page rather than addressing
page A's index space with page B's number — the same class of error the
[`TargetId`](crate::canvas::target::TargetId) newtype exists to prevent,
and one comparison to rule out.

### `fn context`

Returns `None` when there is no object model for the page at all, which is
distinct from "the model says no": nothing can be verified, so nothing may
be promised, and [`drag`] turns it into [`Refusal::NoObjectModel`].

The Object-rung scan asks [`ObjectModelProvider::part_kind`] once per
selected entry, which is a `Vec::get` and a match. It runs on every frame
of an in-flight drag, and that is affordable for the reason the whole
preview is affordable: the decomposition is already built and cached (the
selection could not have outlines to drag without it), so this walks a
slice rather than a content stream.

### `fn run_move`

Answers the engine's refusal of the whole set, or `None` when it moves.
That is what makes a multi-chunk drag refuse *whole*: obligation 3 in this module's header says no ghost may be drawn
for a move that will not happen, and a set is a move that will not happen
as soon as one member of it cannot go.

# Asked of the set rather than of the entered chunk

The entered chunk is `entries[0]`, and a selection of four chunks whose
first one happens to be movable would otherwise draw a ghost over all four
and then refuse at the commit — the operator watching an outline slide
across the sheet and snap back, which is the exact failure O188 is about.

# Asked at the Part rung only

At the Node rung the entries carry a `subpath` too, naming the *enclosing*
part of each selected anchor, so
[`SelectionState::selected_parts_on`] would answer about containers rather
than about the operator's selection. Nothing downstream reads `run_move` at
that rung, and the condition is written here once rather than being
re-derived where it is read.

The cost is a few comparisons inside the engine per selected run, over a
decomposition the provider has already built.

### `fn node_point`

[`ObjectModelProvider::object_node_points`] is the whole-object list
precisely so a caller does not have to re-derive which subpath an
object-scoped index falls in — that offset arithmetic lives in one place,
in the provider, and duplicating it here is how the number pdfcer shows
starts disagreeing with the number the operator can act on.

### `fn decline`

One trace shape for every refusal, so a harness reads `canvas-move-declined`
and finds the cause on the same line rather than inferring it from an
absence — the same contract `canvas-delete-declined` already honours.
