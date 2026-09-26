# `app::actions::bookmarks` — the verbs whose subject is one entry in the
document's outline

A sub-enum beside `PageAction` and `DimensionAction`, filed under the rule
`super`'s declaration of `action` states: the family of variants that
**grows** is the one that becomes a sub-enum.

## What makes these a family rather than a size-driven cut

Every variant here **addresses its operand by `ObjId`** and by nothing
else, and that is a property no other family in the enum has for the same
reason. The reason is the one the engine reported from its own CLI:

> *"the indices shift after every add … I got this wrong myself while
> driving the command and nested something two levels deeper than intended,
> and the output looked entirely plausible."*

An outline is a tree that every edit to it renumbers. A position in the
walk — "the fourth row", "the second child of the first" — names a
different bookmark after any add, any delete, and any undo of either.
`OutlineItem::id` exists precisely so a GUI does not have to hold one, and
its own doc comment says so: *"identity is what a GUI needs and the tree
cannot otherwise supply."*

So the shared property is not *"they are all about bookmarks"*, which would
be a subject label. It is that **every one of them is resolvable after the
frame that raised it**, which is the one thing the action funnel requires of
an operand and the one thing a tree position cannot promise.

## `/Count` is two different quantities, and the sign carries open/closed

§12.3.3 is where implementations of this feature go wrong. The engine's
table:

| | root `/Outlines` (Table 152) | an item (Table 153) |
|---|---|---|
| counts | all visible items **including** the top level | visible **descendants**, excluding itself |
| sign | **cannot** be negative | **positive = open, negative = closed** |

A **closed** item contributes exactly **1** to its ancestors' counts,
however large its subtree is. Three consequences, and this module is built
around all three:

1. **Nothing here diffs a count to describe an edit.** Adding a bookmark
   under a collapsed ancestor leaves the document's total unchanged, so a
   surface reporting *"added N"* from a root-count diff reports **zero for
   a correct save**. [`BookmarkAction::Add`] adds one bookmark and the panel
   says one bookmark; there is no number to get wrong.
2. **A delete's count comes from the engine, not from the tree we drew.**
   See [`BookmarkAction::Delete`].
3. **`open` is the only reason a disclosure about visibility can be
   written at all.** `pdfcer_core::outline::OutlineItem::open` is the shell's
   read of that sign, and §12.3.3 defines no `/Open` key, so the sign is the
   only carrier there is.

## What is deliberately absent

A verb that deletes the whole outline. `EditError::OutlineRootIsNotAnItem`
refuses the root by name, because deleting it is *"a different act that gets
its own verb when it is wanted"*, and this shell does not want it yet.

Where a capability the engine does not have would need a control, this
module grows **no variant** and the panel draws **no handle** — R9: a
capability that does not exist renders nothing, not a greyed promise.

## The `ObjId` rule binds the destination too

[`BookmarkAction::Move`] addresses its **destination** by `ObjId` as well as
its operand, because `OutlinePlacement` is built from anchors rather than
positions, for the reason its own doc comment gives about this exact
surface: *"A shell that reads a panel, lets the operator drag a row, and
then calls with the index it read has a race with its own undo stack."*

## Item notes

### `fn paste`

`OPERATOR_REQUESTS.md` **O59** item 3.

# The disclosure, and why a zero drops the clause entirely

`OutlinePasteOutcome::destinations_dropped` counts bookmarks that arrived
**without** their destination, because it named a page this document does
not have. The engine drops rather than clamps, and that is the right choice:
clamping would send the operator to *some* page, confidently and wrongly,
where a bookmark that plainly does nothing at least shows what happened.

A zero says nothing at all. `app::status` has one slot for consequences and
a sentence reading *"0 destinations were dropped"* would evict a real one to
report an absence — which is `rename`'s argument below, applied to the arm
that does have something to say when there is something.

# It reports what happened; the panel predicted it

`panels::bookmarks::clip::paste_row` warns before the press, from
`OutlineClip::deepest_page()` against the page count. The two are not
duplicates and neither is sufficient alone: a prediction is a guess nobody
confirmed, and a report arrives too late to choose differently. The operator
gets the choice *and* the outcome.

### `fn rename`

# Why the disclosure list is empty, deliberately

`vector_edit` surfaces whatever this returns to `app::status`, and the rule
that module's header states is that a disclosure is *"the part they cannot
see"*. A rename has no such part. `set_outline_title` writes `/Title` on one
dictionary and touches nothing else — its own doc comment says *"nothing in
the `/First`/`/Last`/`/Next`/`/Prev`/`/Count` machinery depends on it"* — so
the entire effect of this call is a row in the panel beside the operator's
pointer reading the words they just typed.

Emitting *"Bookmark renamed."* would put a sentence in the one slot
`app::status` has for consequences, describing something with no
consequences, and it would evict the previous edit's real disclosure to do
it. `super::forms::rename` reports one because a form field's rename
**does** have an invisible part: renaming a parent renames its descendants,
and the count of them is not on screen. Nothing analogous exists here.

# What happens on a refusal

`vector_edit` traces it and leaves the document alone, which is the whole
response this shell gives to any engine refusal today. Three are reachable:
`DocumentEncrypted`, the certification gate, and `NotADictionary` if the id
no longer resolves to an outline item — which is what an id from a stale
draft becomes after an undo. The panel does not pre-empt any of them; see
`crate::panels::bookmarks::edit`'s header for why the encryption case is
not gated at the widget.

### `fn delete`

# The count is the disclosure, and it comes from the engine

`delete_outline_item` returns `usize` — the number of items actually
removed, the clicked one included. That number is the answer to the question
this verb raises and cannot answer any other way: **the subtree went too**,
and on a collapsed parent the operator could not see how large it was.

**Disclose off-canvas, never on the page**, in its plainest form. The panel
already stated the expected size before the press, from the tree it had
drawn; this states what the engine actually removed.

**The two numbers are allowed to differ, and that is the reason both are
said.** `read_outline` gives up part-way on a cycle, on excessive depth, or
on exhausting its item budget — the panel draws a truncation notice when it
does — so the shell's pre-press count is a count of *what pdfcer could
read*, and the engine's is a count of *what it removed*. On any ordinary
document they agree. On a damaged one the after-the-fact number is the true
one, and an operator who saw "3" promised and "47" reported has been told
something real about their file rather than being quietly lied to by the
only number they were shown.

# Why one is not spelled as none

Deleting a leaf removes exactly one item, and *"Bookmark deleted, including
its 0 bookmarks beneath it"* is the shape of sentence that makes a program
look like it is reading from a template. The catalog branches on the count;
see `crate::text::panels::bookmark_deleted`.

# What it cannot be asked to do

The **outline root** is refused by name — `EditError::OutlineRootIsNotAnItem`
— because the root is not an item, carries no `/Title`, and deleting it means
deleting the whole outline, *"a different act that gets its own verb when it
is wanted."* The panel cannot raise that refusal: `read_outline` reports the
root's *children* as its top-level items, so no `ObjId` the panel can offer
is the root's. The refusal is therefore unreachable from this surface rather
than routed around, which is the better of the two outcomes and is recorded
here so nobody adds a guard for a case that cannot occur.

### `fn move_to`

# Three disclosures, from three different sources, and each is needed

| Sentence | Source | Answers |
|---|---|---|
| [`crate::text::panels::bookmarks::bookmark_moved`] | `OutlineMove::visible_items` and `::reparented`, from the **engine** | *how many rows moved, and did it change owner?* |
| [`crate::text::panels::bookmarks::bookmark_move_took_hidden`] | the **tree the panel drew**, read before the call | *how many bookmarks were in the branch that nobody could see?* |
| [`crate::text::panels::bookmarks::bookmark_move_into_collapsed`] | the tree read **after** the call | *why has the bookmark not appeared where I put it?* |

The first two are two different quantities and it is §12.3.3 that makes
them so. `OutlineMove::visible_items` is *"the item plus its **visible**
descendants"*, which for a **collapsed** chapter of forty sections is
**one** — Table 153's sign convention gives a closed node a contribution of
exactly 1 to its ancestors however large it is. The engine's own doc is
explicit that a shell must report that number and must not recompute it:

> *"A shell can say 'moved 1 bookmark (7 nested)' only if the core tells
> it; recomputing it shell-side would be a second implementation of the sign
> convention."*

So the engine's number is reported verbatim, and the branch size is a
**separate sentence from a separate source**, offered only when the item was
collapsed — which is exactly when the two disagree. This is the same posture
[`delete`] takes about its own before-and-after counts, with the difference
that there the two numbers answer one question and here they answer two.

# Why the collapsed-destination check is made AFTER the call

Because it is a fact about the document the move produced, and only the move
knows where the bookmark went. `OutlineMove::to_parent` names it — and it is
carried on the report precisely so a shell does not have to work it out —
but whether that parent is *open* is a `/Count` sign the panel must read
back, because the move itself may have changed it: `move_outline_item`
leaves a parent that already had children exactly as the operator set it,
and **opens one that was a leaf**.

The immediate parent is enough, and a walk to the root would be a walk
nothing could reach. A drop lands on a row, a row is drawn only when every
ancestor above it is open, so a collapsed grandparent implies a destination
that was never on screen to be dropped on.

`to_parent` may be the **outline root**, for a move to the top level.
`read_outline` reports the root's *children* as its top-level items and
never the root itself, so the lookup answers `None` and no sentence is
drawn — which is correct: the top level is always visible.

# What happens on a refusal

A sentence, through `app::status::decline`, recorded from **inside** the
closure — [`crate::app::status::decline::record_resize_not_rebuildable`]'s
placement and its stated reason: whether the engine will refuse is not
knowable before the call. `vector_edit`'s `Err` arm traces, and words an
un-categorised decline naming neither this verb nor a remedy — so **a
refusal must be a sentence** means *this* sentence, recorded here.

The panel forecasts and refuses the one case an operator can act on — a drop
into the bookmark's own subtree — before raising this action at all, so what
reaches here is the residue: `/Encrypt`, the certification gate, and an id
that stopped resolving between the frame and the apply. See
[`crate::text::panels::bookmarks::bookmark_move_declined_engine`] for the
table and for why a catch-all is honest when none of the causes has a remedy.

### `fn set_open`

# Why the disclosure list is empty, and it is the same ruling as
[`rename`]'s

`vector_edit` surfaces whatever this returns to `app::status`, and that
module's rule is that a disclosure is *"the part they cannot see"*. The
whole effect of this verb is rows appearing or disappearing in the panel the
operator's pointer is in. There is no invisible part, and emitting
*"Bookmark expanded."* would put a sentence in the one slot `app::status`
has for consequences — evicting the previous edit's real disclosure — to
describe something already on screen.

The fact that **is** surprising is disclosed, and it is disclosed
**before** the press, on the triangle's hover text: this writes into the
document. See
[`crate::text::panels::bookmarks::bookmark_expand_tooltip`], which carries
the argument. A consequence an operator can still decide against belongs in
front of the control, not behind it.

# One label for both directions

`vector_edit`'s label is a string literal by construction —
`tools/gates/check-trace-names.py` reads it out of the call site — so the
two directions cannot take two labels without splitting this into two
functions that differ by a boolean. They are one verb with one operand and
one refusal set; which way it went is a **key** on the panel's own trace
line (`bookmark-disclosure open=`), which is where a driven check reads it.

# What it cannot be asked to do

A **leaf** — Table 153 makes `/Count` *"required if the item has any
descendants"*, so an item without them has no open-or-closed state.
`set_outline_open` answers `Ok(false)` rather than refusing, and the panel
draws no triangle on one, so this is unreachable from that surface rather
than routed around.

The **outline root** is refused by name (`OutlineRootIsNotAnItem`) because
its `/Count` is the *other* quantity — it counts visible items at every
level and *"cannot be negative"* — so it has no state to set. As with
[`delete`], no `ObjId` the panel can offer is the root's, since
`read_outline` reports the root's children as its top-level items.

### `fn the_three_verbs_are_distinguishable`

`PartialEq` is derived and `super::super::tests` compares whole
`Action`s, so a variant that failed to distinguish itself would make
those comparisons pass for the wrong reason — the exact failure mode
the engine reported on this same Pass, where *"all three of our
sabotage checks survived the first test suite"* because the fixtures
could not tell the answers apart.

### `fn a_rename_is_identified_by_both_its_item_and_its_title`

Both halves matter and only one of them is obvious. The queue may hold
more than one action from a single frame, and `PdfcerApp::apply` applies
them in order; a variant that compared equal on only one of its two
fields would let a de-duplicating caller — or a test asserting "the
queue holds what I expected" — accept the wrong one.

The fixture deliberately makes the two answers different in each
direction, which is the discipline the engine's note asks for: *"when
you assert that A and B differ, check your fixture can tell them
apart."*

### `fn an_objid_generation_distinguishes_two_deletes`

`ObjId` is `(num, generation)`, and a delete addressed to `7 0 R` must
not compare equal to one addressed to `7 1 R`. This is cheap to assert
and it pins the thing that would make the whole "address by id, never
by position" argument in the module header hollow: an id that only
half-identifies is a position with extra steps.

### `enum BookmarkAction`

See the module header for what makes them a family: every one of them names
its operand by `ObjId`, because an outline is a tree that every edit to it
renumbers.
**`PartialEq` and not `Eq`**, and the bound cannot be restored.
`pdfcer_core::outline::OutlineClip`, which [`BookmarkAction::Paste`]
carries, is `PartialEq` only — a bookmark's colour is three `f64`s and
floats have no total equality — and an enum holding one cannot be `Eq`.
Nothing needs it: `Eq` over `PartialEq` buys a `HashMap` key, and no action
is ever one.

### `fn apply`

The dispatch half of this module, reached from `PdfcerApp::apply`'s single
[`super::action::Action::Bookmark`] arm. It is a free function taking
`&mut OpenDoc` rather than a method, exactly like [`super::dimensions::apply`]
and [`super::pages::apply`], because the caller is the one place that owns
the borrow and the arm should be one line.

**Every arm goes through [`super::apply::vector_edit`]** — the
cancel–mutate–bump–invalidate protocol — and none of them may hand-roll it.
Its doc comment carries the argument: four hand-written copies of a
four-step protocol are four chances to omit a step, and the two steps most
easily omitted (the epoch bump and the structural resync) fail *silently*,
leaving an edit that happened in the document and did not happen on screen.

The `page` argument passed to `vector_edit` is **`0` for all three**, and
that is honest rather than lazy: an outline is document-level, no page is
being edited, and the parameter exists only so the diagnostic trace can say
which sheet a geometry edit touched. [`super::dimensions::apply`] passes `0`
for its group verbs for the identical reason. The one exception is
[`BookmarkAction::Add`], which passes the destination page — not because a
page is being changed, but because the page is the operand that decides what
the bookmark points at, and a trace that could not say which one would be
unable to check the commonest thing to get wrong.
