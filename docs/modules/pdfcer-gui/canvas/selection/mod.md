# `canvas::selection` — the selection STATE, and the invariant it exists to hold

## The two halves of this module, and the seam between them

[`identity`] holds what a selection **is**: [`Selection`],
[`SelectionLevel`], [`ClickHit`] and [`EscapeOutcome`] — four `Copy` types
which between them cannot name a place on the screen. That file carries the
*"selection is an identity, not a position"* argument in full, because it is
an argument about the shape of a **type** and is answered by reading four
field declarations.

This file holds what **accumulates** them: [`SelectionState`], the ladder it
walks, the `(page, epoch)`-keyed re-resolve, and every rule about what a
click, a marquee or an Escape means. Those are answered by reading
behaviour, which is why they are worth their own file and their own tests.

All four identity types are re-exported here, so
`crate::canvas::selection::SelectionLevel` remains the path every caller
uses and the seam is invisible from outside the module.

## The invariant, stated first because everything here is shaped by it

`GUI_ROADMAP.md` Phase 1, from the operator's own words:

> *"if I select a node or something for a tool, I should be able to pan
> and zoom out without losing my first selection."*
>
> **Navigation is not an edit. Panning, zooming, changing fit mode,
> rotating the view, switching page-display mode and changing ribbon tab
> must never alter the selection.**

The roadmap names **three** ways the natural implementation loses it, each
of which looks reasonable in isolation. This module closes all three, and
each closure is a structural property rather than a promise:

**1. Selection stored in screen coordinates.** Zoom changes the mapping, so
the stored point stops naming the thing it named. ⇒ [`Selection`] holds **no
coordinate of any kind**. It is `page + object + subpath + node`, four
integers, none of which a zoom can touch, and there is no constructor that
takes a `Pos2`. This is the one closure that is a property of a *type*
rather than of a method, which is why it lives in [`identity`] — see that
file's header for the argument in full.

**2. Selection cleared by a click that was really a drag.** A gesture begins
with a press; if press-on-empty clears, every drag that starts on blank
paper destroys the selection. ⇒ Nothing in this module is called on a press.
The clear is driven by [`SelectionState::click`], which
[`crate::canvas::gesture`] raises only for a **completed click with no
drag**.

**3. Selection invalidated by re-decomposition.** The provider rebuilds on
page change and on edit; a rebuild triggered by zoom, or by a page change
that is not a page change in the operator's sense, must not drop it. ⇒
[`SelectionState::resolve`] **re-resolves against the new decomposition**
instead of discarding, and — the part that is easy to get wrong — it only
validates entries **on the page the provider serves**. An entry for another
page is left completely alone.

Row 3's second half is the one that makes the acceptance criterion pass:
*"select a node, zoom out three rungs, pan across the sheet, switch to
Continuous, come back, switch ribbon tab — the node is still selected and
still the entered level."* Going to another page builds a provider for
that page, and a `resolve` that pruned everything it could not find would
wipe the selection on the way past. Coming back would find nothing.

## Why the level is state and not derived

[`SelectionLevel`] could be inferred from whether `subpath`/`node` are
`Some`. It is stored instead, because *"inside this object, nothing picked
yet"* is a real state — reached by entering an object at a point where no
subpath was close enough — and an inferred level would collapse it into
"not inside anything at all". The operator would then find Escape taking
two presses on one path and one on another, for no reason they could see.

## What this module deliberately does NOT do

It never draws, never touches egui, never reads a pointer, and never
reaches a document. It is a state machine over four integers and a
provider trait, which is precisely why every invariant above can be
asserted in a unit test rather than hoped for in a running window.

## Item notes

### `fn outline_rect`

Falling back to the object's box when a part has no bounds is
deliberate — the alternative is drawing nothing for a selection that
exists, and a correct action with no feedback is indistinguishable
from a broken one.

### `fn click_at_object_rung`

# The second click on a text block narrows to the chunk under it

`OPERATOR_REQUESTS.md` **O215** ask 1, in his words: *"sometimes it moves
the chunk and sometimes it takes the whole block."* Selection has always
descended to a chunk — [`Self::select_part`] and the run menu address one
— but the ladder's own descent gesture is a double-click, and on text
that gesture is spent opening the caret. So the rung had no entrance a
hand could find, and which unit a drag picked up was decided on **press**
by [`crate::canvas::presspick::covers`] against a rectangle nothing drew.

The narrowing is offered on a **plain click**, and exactly three
conditions have to hold at once:

| condition | what it stops |
|---|---|
| `hit.chunk` | a path's subpath, and a text block whose boxes are switched off or which holds one line — narrowing to a unit with no box drawn around it is the same invisible aim, from the other side |
| this object is **already** the whole selection | a click on a different block, which selects it whole |
| `hit.part` is `Some` | a click in the white inside the block's box, which keeps the block |

*"Already the whole selection"* is not enough on its own, and the
hazard is the reason [`crate::canvas::presspick::changed_selection`]
exists: the press of this very click may be what selected the block, and
the state it leaves is identical. The click path folds that answer into
`hit.chunk`, so the first click of a gesture selects the block and puts
the boxes up, and the second — with somewhere visible to aim — takes one
chunk.

Everything below the Object rung is [`Self::click_inside`]'s, which
already re-picks a part on every click, so once here the gesture is
repeatable by a route that was always there.

### `fn click_inside`

Three outcomes, in precedence order: re-pick at the current rung; fall
back one rung and re-pick there; or leave the object entirely and
behave like an ordinary Object-rung click. The middle case is what
stops an operator being stranded at a rung whose targets they keep
missing — at the Node rung, a click that misses every anchor but lands
on a part ascends to that part rather than doing nothing.

### `fn descend`

A double-click on a **different** object enters that object rather
than descending inside the current one: PDF path objects do not nest,
so carrying a part or node index across would address an index in a
different object's space.

### `fn normalise`

1. **Entries are ordered and unique.** Document order, so the outlines
   paint in a stable sequence rather than re-stacking on every
   shift-click; unique, so a batched edit is handed a clean operand
   list.
2. **A rung above `Object` means exactly one object is entered.** A
   rung is a place *inside one thing*, and [`Self::entered_object`]
   derives that from the first entry. Anything that would leave the
   two disagreeing collapses to the Object rung instead — recovering
   is better than asserting, because the state is reachable from a
   marquee arriving while inside an object and the honest response is
   to step out.

### `struct SelectionState`

# Where this lives, and why that is the whole of its document scoping

It is a field of `crate::app::state::OpenDoc` — the open document itself.
That is not filing: it is the mechanism.

A selection is document-scoped state, so closing a document must forget it,
and `OpenDoc::new`'s own doc comment is the guarantee: *"opening a document
constructs a whole new `OpenDoc`, so a cached texture or a page index can
never refer to a page from a previous file."* A selection held inside that
structure inherits it by construction, on every frame, at no cost, with
nothing to compare. `panels::DocKey` and the decomposition cache are scoped
the same way for the same reason.

The alternative — living in `egui::Memory`, which outlives documents, and
*detecting* the change against a token built from the `Arc<EditSession>`'s
allocation address and the page count — cannot be made correct: an address
is not an identity, so a reused allocation with a matching page count
carries a stale selection into a new file, and holding an `Arc` or a `Weak`
to make it a real identity disables editing outright (`Arc::get_mut` fails
while any other strong **or weak** reference exists).

**A page change is still not a document change**, and never was — that is
invariant 3, and it is [`Self::resolve`]'s business, not this note's.

# Why it caches canvas-space outlines

Drawing the selection every frame needs each entry's bounds, and bounds
come from a decomposition. `decompose_page` resolves every `/Contents`
stream, inflates it, concatenates, tokenizes and walks the whole token
stream resolving fonts as it goes, with **no cache anywhere in
`pdfcer-core`** — so asking for it per frame is not an option.

Canvas-space bounds are the right thing to cache because they are
**zoom-independent**: canvas space is the page's device space at scale
1.0, so a zoom or a pan changes where the outline is *drawn* and not what
it *is*. The cache is therefore keyed on `(page, edit epoch)` and survives
every navigation — which is the invariant again, from the drawing side.

### `fn is_empty`

Both, deliberately: every caller of this is asking *"is there something
for a verb to act on?"*, and answering only about content would leave a
selected stamp reading as no selection at all. That is what would make
the contextual Format tab hide itself over a selected annotation.

### `fn select_annot`

Clearing the content selection is not a courtesy: it is the mutual
exclusion this type owns. A build that left both set would draw two
kinds of outline at once and leave `format.delete` with two plausible
meanings, one of which removes page content the operator did not point
at.

The rung resets too, for [`Self::clear`]'s reason — a `Node`-rung state
left behind an annotation selection would put the next content click
straight into a rung the operator never entered.

### `fn clear_annot`

Separate from [`Self::clear`] so a caller that is only retiring the
annotation half — entering a mode that may not author markup, say —
does not also destroy a content selection that mode still permits.

### `fn entered_object`

Derived rather than stored, and safe to derive because every path that
sets a level above [`SelectionLevel::Object`] also collapses the
entries onto one object — a rung is a place *inside one thing*.

### `fn outlines`

Paired rather than a bare `Vec<Rect>` because the overlay needs to
know which entry each box belongs to, and because [`Self::resolve`]
drops entries the provider no longer knows — which breaks positional
correspondence with [`Self::entries`].

### `fn outline_union`

The union rather than the first entry's box, because a multi-select
is one thing to act on: eight grips around one member of a set of
five would say the gesture applies to that member alone.

### `fn object_indices_on`

Ascending and de-duplicated because `EditSession::delete_objects`
resolves **every** index before planning anything, so a duplicate or a
stale entry refuses the whole call rather than deleting the prefix
that happened to resolve. Handing it a clean list is the difference
between "delete refused" and "delete did half of what I asked".

# TARGETS INSIDE A FORM XOBJECT ARE NOT IN THIS LIST

A selection can hold two kinds of thing —
[`TargetId::Object`](crate::canvas::target::TargetId::Object), an index
into the page's own paint order, and
[`TargetId::Leaf`](crate::canvas::target::TargetId::Leaf), an index
into the objects painted from inside a form XObject. **Only the first
is an edit operand**, because every paint-order verb writes to the
page's content stream and a leaf's token range indexes the form's.
In range, wrong buffer, silent corruption — the engine's own reason for
keeping the two lists apart, restated at the one funnel in this shell
that feeds them to verbs.

So a leaf is dropped here, and that is deliberate rather than a
tolerated gap. It does mean **an empty return is not the same as an
empty selection**: a caller that reports "nothing selected" on an empty
list will contradict an outline the operator can see. Ask
[`Self::leaf_indices_on`] before saying so — `canvas::moving` does, and
declines with `Refusal::InsideForm` instead.

### `fn leaf_indices_on`

**This exists so a refusal can be worded.** Its one job is to let a
caller tell *"you selected nothing"* from *"you selected something this
verb cannot reach"*, which are the two states an operator most needs
kept apart: the first is their mistake and the second is the program's
limit. `RESUME.md` records four separate occasions where a limit
reported as an absence cost weeks.

Not an operand list. Nothing in `EditSession` takes one of these
numbers, by design — see [`crate::canvas::target::TargetId`].

### `fn targets_on`

For a **readout** — the status line's *"what is selected"* — which must
describe what the operator can see rather than what a verb can act on.
Never hand a member of this to an edit verb; go through
[`crate::canvas::target::TargetId::page_object_index`], which is the
only thing that can say no.

### `fn deletable_objects_on`

# ⚠ This is not the Delete rule any more

`crate::canvas::deleting::subject` is, and it has arms for all three
rungs. Both claimants — the canvas Delete/Backspace keys and the
ribbon's `format.delete` — ask it, so a delete reaches the rung the
operator is actually on: a subpath at the Part rung, an anchor at the
Node rung, the whole object only at the Object rung.

What this method still answers is *"which whole objects are selected
such that `EditSession::delete_objects` could take them"*, which is a
narrower question and the right one for a caller that means the
whole-object verb specifically.

# Why the distinction is destructive rather than pedantic

At the Part or Node rung the selection names a subpath or an anchor
*inside* one object, while `EditSession::delete_objects` removes
**whole objects**. Deleting the enclosing object because the operator
asked to delete one line of it is the class of error that cannot be
excused by "they can undo it": one measured CAD export holds an entire
drawing view as a single path object with 1,194 subpaths, so the
difference between the two readings is one line and the whole view.

# Returns

Ascending and de-duplicated — [`Self::object_indices_on`]'s contract,
which is what `EditSession::delete_objects` needs in order to succeed
rather than refuse the whole batch. Empty means *"nothing this verb may
remove"*, which deliberately does **not** distinguish "nothing selected"
from "selected at a rung with no delete verb": a caller that must tell
those apart asks [`Self::level`], and the canvas does exactly that in
order to trace the difference.

### `fn click`

See invariant 2 in the module docs: a press that turns out to be a
drag must leave the selection completely alone, so this is only ever
reached from [`crate::canvas::gesture::GestureOutcome::Click`], which
is raised on release and only when no drag happened.

# The rules, and why each one

- **Plain click, hit, at the Object rung** — replace the selection.
- **Plain click, miss** — clear. Clicking empty paper deselects; that
  is what every editor does, and the alternative strands an operator
  with no way back to "nothing selected" except a key they have not
  discovered.
- **Shift+click, hit** — toggle that entry's membership, leaving the
  rest alone. Toggle rather than add, so shift-clicking a selected
  object is its own undo.
- **Shift+click, miss** — unchanged. There is nothing to toggle, and
  clearing here would make an over-shoot destroy a set the operator
  spent five clicks building.
- **Double-click, hit** — descend one rung into the object under the
  pointer (the operator's stated model: *"double-click to get to the
  next level down"*). A double-click at the Node rung changes nothing:
  there is nothing below a point.
- **Plain click while inside an object, hitting nothing in it** —
  leave the rung and apply the ordinary Object-rung rule. Clicking
  away is how every editor exits a group; staying inside until Escape
  strands an operator who has forgotten they descended, which is the
  failure a depth model must avoid above all.

### `fn click_direct`

# Why it is a separate entry point from the ladder

[`Self::click`] implements a *ladder*: a click selects an object, a
double-click descends to its part, another descends to a node. That
model is fine and it is what `move_node` and `move_subpath` are
addressed through, but as the **only** way to reach an anchor it leaves
nothing on screen at any stage saying a deeper rung exists. The
operator:

> *"How do I get to see the end points of an object and select them to
> drag and move? This doesn't work either."*

⇒ **The tool is the rung**, which is what every vector editor does.
With the Node tool armed there is no state to descend through, so there
is no way to be somewhere you did not choose.

It is a *separate function* rather than a flag inside `click` because
the two make different decisions at every branch — this one never
ascends, never needs `entered_object`, and treats a click on a different
object as "show me that one's anchors" rather than as "leave here". A
boolean threaded through `click_at_object_rung` and `click_inside` would
have made both harder to read and neither easier to test.

# What it does

- **Click on an anchor** → that anchor alone is selected, at the Node
  rung, ready to drag.
- **Click on a shape but not on an anchor** → the object is entered at
  the Part rung with its nearest subpath, so **every anchor of that
  subpath appears**. That is the step the ladder had no gesture for and
  it is the one the operator was missing.
- **Shift-click an anchor** → adds it, or removes it if it was already
  in the set. `move_nodes` carries the whole set as one command.
- **Click empty paper** → clears, which is the universal convention and
  the one [`Self::marquee`] already follows.

### `fn marquee_remove`

The operator: *"I can't unselect things once I have selected them for
redaction."* [`Self::marquee`] can only replace or extend, so without a
third answer the only way to drop one of several picked objects is to
shift-click it precisely — which on a CAD sheet of overlapping strokes
is often not practical.

An empty `hits` is a no-op rather than a clear, and the asymmetry with
[`Self::marquee`] is deliberate. A band that encloses nothing means
"replace the selection with nothing" when it is a plain band — that is
how every editor cancels a selection — but "remove nothing from the
selection" when it is a subtracting one. Clearing there would make a
mis-aimed Ctrl-band destroy the very selection the operator was trying
to refine, which is the opposite of what they asked for.

The level is left alone. Subtracting is a change to WHICH objects are
picked, never to how deep the selection has descended, and resetting it
to `Object` would silently throw away an operator's descent into a form.

### `fn marquee`

Always resolves to the **Object** rung, and ascends if the operator
was inside one. A rubber-band names a region of the page, and a region
contains objects; there is no sensible reading of "every subpath of
some other object that this box happens to cover".

That argument is about a band over the **page**, and one case is
outside it: a band drawn while the operator is inside a text block whose
chunk boxes are on the canvas names a region over *visible rectangles of
the one thing being worked on*. [`crate::canvas::marquee::on_release`]
claims that band before this is reached and calls
[`Self::select_parts`]; everything else still arrives here and still
ascends.

Plain replaces, `Shift` adds. An empty plain marquee therefore clears,
which is the Inkscape convention and is **not** the failure invariant
2 is about: that one is about a *press*, and this runs on release,
after a real enclosure test. Panning is the middle button and never
reaches here at all.

### `fn select_placed`

# The complaint this closes

The operator: *"if I add an image I Expect to click on it to resize but
dragging doesn't resize."*

The resize itself is not the problem: an image which **is** selected
resizes from a corner grip (`resize-commit grip=SouthEast sx=0.6810
sy=0.5899`) and moves from a body drag. An image that arrives
**unselected** puts the first press on unselected paper, where
`gesture::meaning` reads it as a marquee — so the operator gets a rubber
band instead of a resize.

# Why this is a convention rather than a convenience

Every one of the eight applications surveyed for this shell leaves a
newly placed or pasted object **selected**, with its handles up. It is
what makes "place it, then get it right" a single continuous act instead
of a place, a hunt and a click. Nothing does otherwise.

# Always the Object rung, and always replacing

The **Object** rung because the operator has just created a whole thing,
not entered one — the subpath and node rungs are places you descend to
deliberately, and arriving inside a freshly placed image would be a
state nobody asked for.

**Replacing** rather than adding, even though a placement is arguably an
addition: what was selected before is what the operator was working on
*before* they placed this, and a two-object selection whose members were
chosen minutes apart is not a set anybody intended to transform
together.

### `fn select_only`

The shared body of [`Self::select_placed`] and of the Objects panel's
row click, and it exists as one function because the two are the same
act: *"this, and only this, is now the thing being worked on."*

`why` names the origin and reaches only the trace. It is a
`&'static str` rather than an enum because nothing branches on it —
the moment something does, it should become one.

# Why the Objects panel writes the SELECTION and not a focus

A panel-local focus is a second notion of *"the thing I am working on"*
that the canvas knows nothing about, and it produces the operator's
*"when I have an object selected like text the Tool tab doesn't switch
to giving me the editable stuff for that object"*: three parallel
answers to one question — the armed tool, the panel focus and the canvas
selection — with no bridge between them and none of them authoritative.

One notion, written from both ends. A row click selects on the canvas; a
canvas click is what the panel describes. Neither can disagree with the
other, because there is nothing left to disagree with.

### `fn select_part`

# Why this exists as its own verb

`OPERATOR_REQUESTS.md` O188(A). The ladder's own entrance to the Part
rung is [`Self::click_direct`], reached only by arming the Points tool
with a chord *before* clicking, and nothing on any surface names it —
so the rung needs a second entrance that a menu can offer.
[`crate::canvas::runmenu`] carries the measurement of every other
gesture and why each lands one rung up.

A second entrance needs a way to *say* "stand here, on this part", and
[`Self::select_only`] cannot: it hard-sets [`SelectionLevel::Object`],
correctly, because that is what its callers mean.

⇒ The alternative was a `level` argument on `select_only`, and it was
refused for the reason this project refuses most boolean parameters: a
call site would then read `select_only(page, object, SOMETHING, why)`
with the rung as a value rather than as the thing the function is for,
and *"the right verb at the wrong rung"* is exactly the defect class a
second entrance to a rung can introduce.

# What it guarantees

- **Exactly one entry.** Descending is always a narrowing; a multi-part
  selection at the Part rung has no meaning any verb consumes
  (`delete_text_run` and `move_subpath` each take one part), and the
  Part-rung verb table in `panels::objects::provider` is written per
  part.
- **`node: None`.** The Node rung is one further down and this verb
  does not reach it. `Selection::node`'s own doc states the invariant
  that `Some` implies `subpath.is_some()`; the converse is not implied
  and must not be invented here.
- **`normalise` is still called**, exactly as every other mutator in
  this file does, so the one-place-that-sorts-and-dedupes rule holds
  even for a selection that cannot need sorting. A mutator that skips
  it because it *happens* not to need it today is the seam the next
  caller falls through.

**The caller is responsible for `part` being in range.** This type
holds no document and cannot check. `canvas::runmenu::resolve` re-asks
the provider immediately before calling — see its header for why a
condition that answered *for a frame* is not enough for a press that
happens in one.

### `fn select_parts`

# It replaces, and the combining happens above it

`parts` is the whole answer, not an addition to one: a rubber-band at
this rung can add, replace or subtract, and
[`crate::canvas::marquee::on_release`] has already worked out which
before it calls this. Two places deciding what a band does to what was
already selected is how a Shift-band and a plain one come to disagree
about the same rect.

An empty list clears rather than leaving an entry-less Part rung
standing: a rung with nothing in it puts the next click's first
selection into a rung the operator never entered, which is the trap
[`Self::clear`] exists to avoid.

⚠ Every entry names the **same object**. A Part-rung verb addresses
parts within one object, so a set spanning two is not one command —
[`Self::selected_parts_on`] carries the long form of that argument.

### `fn selected_nodes_on`

# Why this exists as its own accessor

Because a multi-node selection has been *representable* since the Node
rung landed — [`Self::pick_within`] adds a Shift-clicked anchor as its
own entry, and [`Self::entries`] holds them all — and **nothing read it
that way**. `canvas::moving::subject` asked [`Self::entered_object`],
which is the FIRST entry, so an operator could Shift-click four anchors,
watch four highlight, drag, and move one.

A capability that the data model supports and no consumer reads is the
hardest kind of gap to see: nothing is missing, nothing fails, and the
unit tests of both halves pass. Giving it a name with a doc comment is
what makes the next consumer ask the right question.

# Why it filters on the object as well as the page

`move_nodes` addresses anchors **within one object**, so a set spanning
two objects is not one command. It cannot arise today — the Node rung is
entered inside a single object and `click_inside` ascends the moment a
click leaves it — but a caller that assumed otherwise would build an
operand list the engine would refuse, and the refusal would arrive after
the operator had watched an outline slide.

### `fn selected_parts_on`

The Part-rung twin of [`Self::selected_nodes_on`], and it exists for
the same reason: [`Self::pick_within`] has always pushed a
Shift-clicked part in as its own entry, and normalisation only
collapses a deeper rung when the entries name different *objects* or
*pages*, so several parts of one object survive. Nothing read them that
way, so an operator could Shift-click four chunks, watch four outline,
drag, and move one.

# Why it filters on the object as well as the page

Every Part-rung verb addresses parts **within one object**, so a set
spanning two objects is not one command. `normalise` already prevents
that state, and filtering here means a caller cannot build an operand
list out of a state this type does not produce.

⚠ **Ask this at the Part rung only.** At the Node rung the entries
carry a `subpath` as well, naming the enclosing part of each selected
anchor, so this would answer about containers rather than about the
operator's selection.

### `fn escape`

**One press, one rung** — decision 025's L1. The old shell shipped
Escape as "clear everything", so an operator who descended two rungs
to reach a line and pressed Escape once found themselves back at the
page. Collapsing the ladder in one press is exactly as wrong as
requiring three presses to clear a single selection.

### `fn resolve`

Called every frame; does real work only when `(page, epoch)` has
moved, because that is the only time the answer can have changed. A
zoom, a pan, a fit-mode change, a ribbon-tab change and a window
resize all leave both halves of that key untouched, so they cost one
comparison and change nothing — which is the invariant, enforced by
the shape of the code rather than by a rule somebody has to remember.

# Only the resolved page is validated, and that is the load-bearing part

An entry on **another** page is left exactly as it was. The provider
serves one page (`panels::objects::provider`, "Single-page by
design"), so it has nothing to say about the others, and a `resolve`
that pruned everything it could not find would wipe the selection the
moment the operator paged away — and find nothing on the way back.
That is the acceptance criterion, and it is this `if`.

# What is dropped, and silently

An entry whose object the provider no longer knows, i.e. one an edit
removed. Dropping it is not a fact the operator needs disclosed: they
deleted it. Keeping it would leave a selection naming a hole, and the
next Delete would refuse the whole batch because
`EditSession::delete_objects` resolves every index before planning.
# `None` means "this page has no object model", not "nothing is selected"

A page whose content streams will not decode has no decomposition, and
the honest response is to draw no outlines while keeping the
selection — the two states are different, and conflating them would
make an undecodable page silently deselect. The `(page, epoch)` key is
still recorded, so the failed decomposition is **not retried every
frame**: the failure is deterministic (same bytes, same code), which is
the same argument `PanelsState::provider_built` and
`settle_and_rasterize`'s render-error hold both make.

### `fn resolve_annot`

Called every frame from `canvas::interact`'s step 7, beside
[`Self::resolve`]; does real work only when `(page, epoch)` has moved,
which for an annotation means *an edit happened*.

# What goes stale without it

[`AnnotSelection::outline`] and [`AnnotSelection::oriented`] are both
cached at click time, so every verb that changes an annotation's `/Rect`
— move, resize, rotate, a typed Apply in the properties panel, an undo
of any of them — leaves the painter stroking a box the mark has left. A
quarter turn takes the `/Rect` from 473.7 × 249.6 to 256.6 × 477.4 while
the outline stays at the first, with its grips on it, until the operator
clicks away and back.

⇒ **The grips are what make this more than cosmetic.** They are laid
out on the same box, so after an edit the eight squares and the rotate
handle are somewhere the mark is not — and a press on the mark itself
can miss the body test entirely.

# Not found is left alone, deliberately

An annotation the walk cannot find has been deleted, and this function
does **not** drop the selection for it. Deselection on delete is
`panels::properties::annotdelete`'s job and it has an operator-facing
disclosure attached; a silent drop here would race it and produce two
different behaviours for one event depending on which ran first.

# Cost

One `/Annots` walk of the current page, bounded by
`pdfcer_core::annot::MAX_ANNOTS_PER_PAGE`, on the frames after an edit
only. No decomposition, no content stream, no raster.

### `fn needs_resolve`

The canvas asks **before** building a decomposition, because building
one is the expensive half: `decompose_page` inflates, concatenates,
tokenizes and walks every content stream on the page with no cache
anywhere in `pdfcer-core`. A zoom, a pan, a fit change and a ribbon-tab
change all leave both halves of the key untouched, so on the
overwhelming majority of frames this is `false` and nothing is built
at all.
