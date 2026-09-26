# `canvas::target` — the seam a hit-testable content model plugs into

The canvas selects *things*. It does not know what a thing is, how it was
decomposed, or what coordinate frame its geometry was authored in. All it
needs is: **what is under this point, what is inside this rect, and where
is the thing I already have?** That question set is
[`CanvasTargetProvider`], and everything in `canvas/` is written against
it rather than against `pdfcer-core`.

## Why a trait rather than a direct call into the provider

Three reasons, in order of how much they cost if ignored.

1. **The selection layer becomes headlessly testable.** Every invariant
   this stage is accountable for — *selection survives navigation* above
   all — is a property of the selection layer's *logic*, not of PDF
   decomposition. A test that had to build a `Document` to prove that
   zooming does not clear a selection would be a slow test of the wrong
   thing. [`StubTargets`] lets those tests state a page's contents in
   three lines.
2. **The old shell already drew this line, and the provider was salvaged
   expecting it.** `panels::objects::provider`'s header, §2 of "What
   changed at salvage": *"The `CanvasTargetProvider` trait impl became
   inherent methods. The trait lives in `canvas/` and does not exist yet.
   The three methods keep their names and their exact semantics …
   Re-attaching the trait at S4 is a one-line `impl` block over methods
   that already have the right signatures."* This module is that
   re-attachment, and it is exactly that: [`impl CanvasTargetProvider for
   ObjectModelProvider`] delegates and adds nothing.
3. **`GUI_ROADMAP.md` Phase 4** (continuous page display) changes *which
   pages* a provider answers for. A canvas written against the concrete
   single-page provider would have that assumption spread through it; a
   canvas written against a trait that takes `page_index` on every query
   already asks the right question.

## Every geometric argument here is CANVAS space

Points, rects and tolerances crossing this trait are in canvas space —
Y-**down**, origin at the page's top-left, `/Rotate` already resolved. The
provider owns the hop into PDF user space (Y-**up**), because it owns the
page transform and inverts the *renderer's own* map to get there, so the
selection geometry and the raster agree by construction. See
[`crate::canvas::mapping`] for the full three-frame table and why
conflating any two of them is silent.

## The tolerance is a parameter, never a provider constant

Stated on [`CanvasTargetProvider::hit_test`] and worth stating here too:
the only honest source for a hit tolerance is the live zoom, and the live
zoom belongs to the frame, not to the model. A provider that baked its own
tolerance would be a provider whose catch radius shrank as the operator
zoomed out — which is the defect
[`crate::canvas::mapping::SELECT_SCREEN_TOLERANCE_PX`] exists to close.

## Item notes

### `fn hit_test_all`

The required half of the point query, and the input to click-through
cycling: an object entirely covered by another can only ever be
selected by stepping past the cover, and a topmost-only query gives no
click any way to do that.

Empty for a miss, and empty for a query about a page this provider
does not serve. Those two are deliberately the same answer — a caller
that must distinguish them is asking the wrong object; the *canvas*
knows which page it drew.

### `fn hit_test`

`tolerance` is the canvas-space slack the click may miss an object's
edge by, and it is a **parameter, not a constant** — see the module
docs. Callers hand it
[`crate::canvas::mapping::PageMapping::tolerance`], which is the one
place the screen radius is divided by the zoom.

**A provided method, not a required one.** Defined as the head of
[`Self::hit_test_all`], which is what makes *"what does a plain click
select?"* and *"what does cycling start from?"* structurally the same
answer rather than a convention two implementations have to keep.

### `fn object_class`

This exists so that [`crate::canvas::input::probe`] can skip
candidates whose class is switched off *without* knowing anything
about how objects are stored. The alternative — handing `probe` the
decomposition and letting it match on `VectorObject` — would put a
second kind classifier in the codebase, which
`crate::panels::objects::summary` exists to prevent.

A provided method returning `None`, so the test doubles in this
module and elsewhere do not all have to implement it. `None` means
*"I cannot say"*, and the filter's contract for that is to let the
candidate through: a provider that does not classify must not become
a provider whose objects are all unselectable.

### `fn containing_form`

A provided method answering `None`, like [`Self::object_class`], so a
double that has no forms is unaffected. `None` means *"nothing to
substitute"*, which is the honest answer for a page with no forms and
for a provider that does not model them.

### `fn container_is_worth_selecting`

# The question the Smart-Selector forgot to ask

`canvas::smart::Scope::resolve` maps a leaf to its containing form so
that a first click selects the container and a double-click descends —
`OPERATOR_REQUESTS.md` O70, and the right model for a title block or a
stamp.

It is the **wrong** model for the commonest form in the world.
Every CAD exporter this project has seen wraps a drawing's whole visible
body in one page-sized form XObject, and a `/BBox` is a clipping extent
(§8.10.1) rather than a claim about ink — so that wrapper contains
everything, wins every click, and "select the container first" becomes
"select the whole drawing, every time".

⇒ Which is the operator's **headline complaint**, verbatim, restored by
the feature built to improve selection:

> *"There are obviously more than one item on the page, but when I click
> on one of the objects all I get is the page selected."*

So a container is worth resolving to only when selecting it says
something selecting the leaf does not. A container that holds
**everything on the page** says nothing: it IS the page, under another
name.

# What it does NOT change

**Entering** such a form still works, and must. A double-click descends
into it, the Objects panel lists it, and the canvas menu's *"select the
containing form"* reaches it. Reachable on purpose was always the
design; winning by default is what was wrong, both times.

Defaults to `true` — a provider that cannot measure says yes, which is
the behaviour before this existed.

### `fn hit_test_rect`

`forms` is a parameter as of 2026-09-11, and for the same reason one
rung up: the engine's [`pdfcer_core::vector::hit_test_rect_deep`] makes
it explicit *"a deliberate act at the call site rather than a
surprise"*, and a shell that pinned it to one value inside one
implementation would be re-taking a decision the engine had just handed
to the caller. The three callers in this crate do not all want the same
answer, which is the practical half of the same argument:

| caller | passes | because |
|---|---|---|
| the rubber band | `Include` | a leaf is not an edit operand here; the container is |
| Select All | `Include` | it is a census, and the form is one of the things on the page |
| *"are there images on this page?"* | `Exclude` | it asks about **ink**, and a `/BBox` is an extent declaration |

The live provider's `hit_test_rect` carries the long form of the first
row, including why this shell does **not** take the engine's `Exclude`
default.

### `fn bounds`

**`None` rather than a panic is the contract**, and it is what makes
re-resolution possible: a selection can outlive an edit that removed
what it named, and the correct response is to drop the entry silently,
not to crash the frame that is trying to draw.

### `fn page_objects_model`

One query for both kinds, because the alternative is a kind match at
every call site and the failure when two of them drift is that
descending works for a drawing and not for a label. The dispatch lives
in the provider ([`ObjectModelProvider::part_hits`]).

Empty for an object with no part rung at all (an image), which is why
the ladder caps itself at the Object rung for images by construction
rather than by a check.
The page's decomposed geometry, for a consumer that needs the model
itself rather than a hit test over it.

**The one consumer is the two-line measure tool**, whose pick is
`pdfcer_core::vector::linepick::pick_line_in_page` — a query this trait
deliberately does not wrap. Wrapping it would put a *second* line-pick
rule in the shell beside the engine's, and the whole point of the
two-line dimension is that the shell and `pdfcer dimension-add`
resolve the same click to the same line.

`None` is a real answer, not a failure: a provider that has no
`PageObjects` for `page_index` (the test double has none at all) simply
cannot be asked, and the caller's correct response is to take no pick
rather than to substitute one.

### `fn part_bounds`

The *part's* box, never the object's. An object-sized rectangle drawn
around a part tells the operator they selected the whole thing again
— which is the misunderstanding entering the object exists to
resolve, and on a measured CAD export that rectangle spans the entire
drawing.

### `fn nearest_node`

Object-scoped, not part-scoped, and that is load-bearing rather than a
convention: it is the space `vector::anchor_count` reports and the
space `pdfcer node-move --node N` addresses. A second numbering
would make the number pdfcer shows disagree with the number the
operator can act on.

### `const COVERS_EVERYTHING`

0.9 — a container over nine tenths of everything on the sheet **is** the
sheet. See [`CanvasTargetProvider::container_is_worth_selecting`] for the
defect this number exists to prevent, and for why it errs generous.

### `impl CanvasTargetProvider`

`panels::objects::provider` carried these methods across salvage as
inherent methods with their signatures and semantics unchanged, precisely
so this block would be a delegation and nothing else. It is: no
arithmetic, no tolerance rule, no hit ordering and no index convention is
decided here. Every one of the delegated methods is already under test in
that module against a real decomposition.

[`Self::hit_test`] is deliberately **not** overridden. The provider has an
inherent `hit_test` with the identical derivation (the head of
`hit_test_all`), and its own doc comment says the comment carries the
guarantee *"until the trait comes back"*. The trait is back, so the
guarantee is structural again and the inherent one is the redundant copy
— overriding here would reinstate two derivations of one answer.

The three ladder methods take `page_index` even though the provider is
single-page and its inherent methods do not, so the guard is applied on
this side. A canvas that descended into a part of a page the provider does
not serve would be addressing paint-order indices in the wrong page's
index space, which is the same class of error the `TargetId` newtype
exists to prevent — and the guard costs one comparison.

### `fn container_is_worth_selecting`

# The measurement, and why it is not "is it page-sized"

The obvious predicate — compare the form's `/BBox` with the page's media
box — needs a page rect this provider does not hold, and it answers the
wrong question anyway. A form can be *smaller* than the page and still
contain every mark on it, which is common: an exporter that wraps the
drawing body but not the margin produces exactly that, and selecting
that wrapper is just as useless.

So the comparison is against **what is actually on the sheet** — the
union of every page object's bounds. If the container covers essentially
all of it, the container IS the page's content, and selecting it tells
the operator nothing they did not already know.

[`COVERS_EVERYTHING`] is deliberately generous. What it guards against
is severe and constant — every click on a CAD drawing — and the cost of
being slightly too generous is that one unusually large title block
stops being offered as a container on the *first* click, while staying
reachable by double-click, by the Objects panel and by the canvas menu.
A mild inconvenience against a headline defect.

### `fn object_class`

Delegates to `panels::objects::summary::object_kind`, which is **the**
kind classifier in this crate, and then maps its answer with
`PickClass::of_object`. Two hops rather than one match, deliberately:
the hop through `object_kind` is what keeps this from becoming a
second, drifting copy of the same decision.

### `fn the_stub_marquee_reaches_inside_a_form_and_honours_the_policy`

# What this is really testing, and why it is not the engine's job


The engine proves its own deep marquee against real content streams.
What is proved here is narrower and is the part the engine cannot see:
**that this crate's test double answers the same SHAPE of question the
live provider answers**, so a selection-layer test written against it is
evidence about the shipped program.

# Both values of `forms`, one rect, deliberately

An `Include`-only assertion passes against a stub that accepts the
parameter and never reads it, which is precisely the shape a hurried
threading of this change would have — the same trap
[`the_stub_marquee_requires_full_enclosure`] names for `mode`, one
argument later.
