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
