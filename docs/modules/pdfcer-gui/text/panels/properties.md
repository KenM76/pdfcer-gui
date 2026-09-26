# `text::panels::properties` — the Properties panel

`RIBBON_IA.md` §5.8 commissions two surfaces for a selection's
properties, and is explicit about which is built first:

> The division of labour: the **tab** carries what a user changes *while
> working* — colour, width, style, align, delete. The **panel** carries
> everything, including the read-only facts (winding rule, node count,
> embedded-font status, exact geometry) that belong beside the Objects
> panel's inventory rather than in a ribbon band.
>
> Build order: **panel first, tab second.** The panel is the harder half
> and the tab's contents are a subset of it, so building the tab first
> would mean writing the property editors twice.

This is that panel's copy — **the read-only half of it**, which is all of
it at stage S3.

## What is deliberately absent, and why it is absent rather than greyed

§5.8 also says the panel is *"where the **editable geometry** lives — X,
Y, W, H as typed values"*, and calls that the surface through which
`/Rect` move-and-resize becomes reachable without a drag. **None of that
is here.**

Not because typed geometry is hard, but because there is nothing to edit:
[`crate::app::actions::Action`] carries zoom and page navigation and
nothing else, and the panel that would host the editors has no selection
to host them for. Four spinners bound to nothing would render, accept
typing, and discard it — which is not a placeholder in the harmless sense
but a control that silently loses an operator's work.

`RIBBON_IA.md` P3 states the rule this follows: *"An unavailable
capability renders nothing, not a disabled stub. Greying is reserved for
**temporarily** unavailable — no document open, document encrypted, undo
stack empty — and is always explained on hover."* "The selection model
does not exist" is not temporary unavailability; it is absence.

So the geometry is stated as **facts**, in the same field list as
everything else, and becomes editable when there is something to edit.

## The panel is the disclosure surface

Every `ObjectNote` an object carries is spelled out here in full, at the
foot of the field list. That placement is the disclosure rule's, not a
layout preference: inference reporting belongs **off-canvas** — *"a
status line, a results panel, a report after the command, a properties
field"* — and the page view must carry no badge, tint, dashed outline or
"provisional" layer at all.

The one-line test the rule offers: *would a screenshot of the editing
canvas differ from a screenshot of the same document saved and reopened?*
Nothing in this panel can make it differ, because nothing in this panel
draws on the page.

## Field wording lives next door

The *values* — kind names, paint dispositions, winding rules, colours,
font labels, note sentences — are all [`super::objects`]'s, and are
reached from here rather than re-worded. That is the same
single-description discipline
[`crate::panels::objects::summary`] exists to enforce, applied one layer
up: a path's fill colour must not be described one way in an Objects row
and another way in a Properties field.

This module owns only the **labels** — the left-hand column — and the
panel's own chrome.
