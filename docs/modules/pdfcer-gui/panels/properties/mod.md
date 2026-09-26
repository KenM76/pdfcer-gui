# `panels::properties` — the detail of what is selected, and nothing else

## Every section here is scoped to a SELECTION

The file's `/Info` fields and the seven facts about the file itself are
[`crate::panels::docprops`], a panel of its own, reached by
`file.document_properties` on File ▸ Document and mounted in all three
modes. **Every section in THIS panel is scoped to a selection**, and that is
the property to preserve: the one thing this panel is for is answering
*what is the thing I picked*, which is what makes it the detail half of the
Objects/Properties master–detail column `OPERATOR_REQUESTS.md` O123 asks
for.

⇒ **A section added here must read the selection.** If it does not, it
belongs in `docprops` or in a panel of its own; a permanent block at the
foot of this one puts something on screen that is true of no selection, and
it takes an operator report to find, because every test in this file stays
green.

An egui finding kept here because it is about the toolkit rather than
about any one section: a
`CollapsingHeader` whose open state must follow a condition **and** stay
overridable by the operator cannot use `default_open` (consulted only
through `load_with_default_open`, so it is dead after frame one) and cannot
use `open(Some(_))` (it forces the state every frame and moves click
handling into an `else` branch, so the header stops responding at all). The
shape that works is `set_open` + `store`, run **only on the frame the
condition flips**. Checked against the vendored egui 0.35 source rather than
remembered.

## Built from the specification

**New.** Nothing like it exists in the shell this one replaces — a
properties panel of any kind is not among what is carried across — so this
is `RIBBON_IA.md` §5.8 built from the specification.

## Why the panel is built before the tab

§5.8, verbatim:

> Build order: **panel first, tab second.** The panel is the harder half
> and the tab's contents are a subset of it, so building the tab first
> would mean writing the property editors twice.

and, on the division of labour between the two surfaces:

> the **tab** carries what a user changes *while working* — colour,
> width, style, align, delete. The **panel** carries everything,
> including the read-only facts (winding rule, node count,
> embedded-font status, exact geometry) that belong beside the Objects
> panel's inventory rather than in a ribbon band.

Those four parenthesised facts are this module's brief, and all four are
below.

## One description, two renderings

Every value here comes from [`super::objects::summary::describe_object`]
and is worded by [`crate::text::panels::objects`] — the *same* record and
the *same* functions the Objects panel's row label uses. That is the
single-source-of-truth requirement made structural: a path's fill colour
cannot be described one way in a tree row and a different way in
Properties, because there is only one description.

What differs between the two is *shape*, not *content*: a row is one line
and joins its facts with separators; a panel is a list and labels them.
[`crate::text::panels::properties`] owns the labels — the left-hand
column — and nothing else.

## This panel is where rule 4's disclosure lands

Every [`super::objects::summary::ObjectNote`] the object carries is
spelled out in full at the foot of the list, under its own heading. That
placement is the disclosure rule's:

> **Disclosure lives off-canvas**: a status line, a results panel, a
> report after the command, a properties field. … **No badge, tint, red
> flag, dashed outline or "provisional" layer drawn into the page view.**

In the old shell, "these bounds are approximate" drove a **dashed outline
on the page**. Under the rule as it now stands that is content marking,
and content marking is forbidden — it is *"a second rendering path for
the same content, and two paths drift"*. Here the same fact is a
sentence, in a panel, and the canvas is untouched.

The heading is *"Worth knowing about this object"* rather than
*"Warnings"*, and the sentences are drawn at ordinary weight: every one
of them is a fact about the **document**, and warning styling would make
a property of the file read as a pdfcer failure.

## What it describes, given that there is no selection

`super::PanelsState::focus` — the object whose row the operator last
clicked in the Objects panel. That is **not a selection**, and the
difference is spelled out where the field is declared. The consequence
here is that the panel's empty state names the Objects panel by name: it
is the only route in, and an operator has no way to guess that.

## What is deliberately not built

§5.8 also commissions **editable X/Y/W/H** here, and calls the panel the
surface through which `/Rect` move-and-resize becomes reachable without a
drag. None of it is here, and the reason is not that typed geometry is
hard: there is nothing to edit. [`crate::app::actions::Action`] carries
zoom and page navigation, and this module may not add to it. Four
spinners bound to nothing would render, accept typing, and discard it —
not a harmless placeholder but a control that silently loses an
operator's work.

So the geometry is stated as facts in the same list as everything else,
and [`crate::text::panels::properties::properties_read_only_note`] says
so once at the top. `RIBBON_IA.md` P3: an unavailable capability renders
nothing; greying is for *temporarily* unavailable, and "the selection
model does not exist" is absence, not temporary unavailability.

## The embedded-font field is a name join, and it discloses that

A text object records the `/BaseFont` in effect; the document's font
inventory records a program per font **dictionary**. Joining them by name
is the only join available — the object model does not carry the font
dictionary's object id — and a name is not a key. One document can
declare two font dictionaries with the same `/BaseFont` (two independent
subsets of one face, which the survey behind the Fonts panel found in
87 % of embedding files), and they need not agree about embedding.

So the field has **three** answers, not two: yes, no, and *"pdfcer could
not tell — the Fonts panel lists each one separately"*. Picking one when
the name is ambiguous would be an inference presented as a fact, and
unlike most inferences this one is invisible: a confidently wrong "Yes"
looks exactly like a right one.
