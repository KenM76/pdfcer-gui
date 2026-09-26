# `canvas::objring` — Tab walks the objects on the page

`OPERATOR_REQUESTS.md` O204, the object half:

> *"The tab should tab through whatever space I have clicked on (example if
> I have an object selected on the canvase it should tab through to the next
> object as expected …)"*

## Contract

[`stops`] is pure over a provider and a filter, and is what the tests drive.
[`advance`] spends a press [`crate::canvas::tabnav`] took off egui and moves
the canvas selection one stop along that list.

## What the ring contains, and why it is not simply "the page's objects"

Every CAD exporter this project has seen wraps a drawing's whole visible
body in one page-sized form XObject, so a ring over the page's own objects
alone would have one stop on a real sheet and it would be the whole sheet.
The ring is therefore **scoped to whatever the selection is standing in**:

* a leaf is selected — the leaves sharing its containment chain, which is
  the set of things drawn beside it inside the same form;
* otherwise — the page's own objects, in paint order, minus any container
  [`CanvasTargetProvider::container_is_worth_selecting`] rejects;
* and when that leaves nothing, the contents of the outermost forms, which
  is what a click anywhere on such a sheet selects anyway.

That is *"tab through whatever space I have clicked on"*, one level further
in wherever the page itself has nothing to offer.

## Why it wraps within the page

O204 decision 3. The field ring crosses pages because a form is one thing to
fill in; an object ring that changed page would also have to scroll, and
that is a second gesture nobody asked for.

## Item notes

### `const PAGE_SIZED_FORM`

The operator's CAD case in miniature, and the reason this module
exists: a ring over the page's own paint order would have exactly one
stop on this sheet and it would be the wrapper.

### `fn the_fixture_is_one_page_sized_wrapper_over_three_squares`

Its own test so that a fixture that stopped having a form fails here,
with a sentence about the fixture, instead of turning the rest of this
module into a confusing report about tab order.

### `fn a_page_sized_wrapper_is_skipped_and_the_ring_is_its_contents`

A ring of one stop that is the entire drawing is indistinguishable
from Tab doing nothing, which is the report this whole row started
from. The fallback is what keeps the gesture meaning something on the
only kind of file the operator actually opens.

### `fn a_filter_that_excludes_the_contents_leaves_no_ring`

The three squares are paths. Asking for text only must not produce a
ring of paths by some other route — the second and third branches of
`stops` both apply `allowed`, and this is what proves it.

### `fn every_stop_indexes_into_this_providers_own_lists`

`object_class` guards on the page index and answers `None` off it,
which `allowed` deliberately reads as *let it through* — so the guard
that matters here is the one in `advance`'s caller, and this pins the
one thing `stops` itself can promise: it never invents a target index
that is not in this provider's own lists.

### `fn advance`

Does nothing on the overwhelming majority of frames: `tabnav::take` answers
`None` unless the hook claimed a press, which it does only while a canvas
surface holds egui's keyboard focus.
