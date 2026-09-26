# `panels::properties::dimension` — the selected ce dimension's own
properties

## The gap this closes, quoted from the people who found it

`docs/ui_specs/tool-options-dock-and-ce-dimension-properties.md` §C.11,
item 2, written before any of this shell existed:

> **A selection-driven property surface for an ALREADY-PLACED ce
> dimension.** … There is **no** panel today that shows a selected ce
> dimension's own properties — not its group, not its radius/diameter
> toggle, nothing. An operator who placed a Radius ce dimension and later
> wants Diameter has **no way to change it** without deleting and
> redrawing … This is the concrete gap Ken's request is actually naming
> when he says *"the ce dimensions I add need to be editable as well"*.

and `pdfcer`'s `FEATURES.md`, which carries the same row as
`core [x] cli [x] gui [ ]` for **both** shells:

> ce-dimension style AND tolerance in the GUI — **one panel covering
> both**, showing which values are inherited and which are overridden and
> letting the override be set. The model and the CLI already do all of it;
> only the disclosure surface is missing.

## Why here, and not on the Format tab

The ui-spec settles it (§C.12) and the reasoning is not about taste:

- **A ribbon band is about seventy points tall.** Eleven property editors,
  each with a checkbox and a provenance sentence, do not fit in one and
  would not be readable if they did.
- **Per-ce-dimension editing must work with no tool armed.** Tool Options is
  for an *armed tool's* controls; this is a *selection's* properties, which
  is one step further down — properties of the selected object, not of the
  thing that would create one.
- **`RIBBON_IA.md` §5.8 wants both surfaces**, with the tab carrying *"what
  a user changes while working"* and the panel carrying *everything*. The
  panel is the harder half and the tab's contents are a subset of it, so
  the specified build order is panel first — which is exactly what
  `manifest/format.rs`'s header already records.

## And why it is a section of Properties rather than a panel of its own

§C.12 answered this too, and it flagged the consequence honestly: the
Properties panel's own premise — *"nothing else competed for the word
Properties"* — **stops being true** the moment a ce-dimension selection
becomes a second claimant on it. The fix it recommends is to broaden the
panel's stated purpose rather than to invent a ninth panel or rename the
tab, and that is what has been done: [`super::body`] now shows *the
document's object properties, OR the properties of whatever is selected on
the canvas*.

Transient, "what I am looking at right now" content goes **first**, above
the persistent object form — the same top-first-bottom-persistent ordering
the Objects/Properties split already establishes.

## Rule 4: everything here is off-canvas, and the canvas is untouched

The selection outline is the cursor, which the rule permits by name. Nothing
in this section tints, badges or flags the ce dimension it is describing;
the provenance of a value — *"using the group's setting"* — is a sentence in
a panel, and the drawing renders exactly as it will save. That is the whole
point of the surface: the inference (which tier supplied this) is disclosed
**off** the page, not marked **on** it.

## What the operator cannot do here, and why each is stated on screen

| | why |
|---|---|
| ~~change the group~~ | **closed 2026-08-19.** It was *"no engine verb; filed"* — filed on the 18th, shipped on the 19th, and it is a picker now. What it gained with the verb is a **disclosure**: `set_dimension_group` re-measures, so the number changes |
| change the scale | **by refusal** — `StyleOverrides` has no scale field, asserted structurally, because a member measuring at a different scale from its group would print a number nothing on the page discloses |
| drag the extension lines | the gap and overshoot are standard-derived, not per-ce-dimension fields; new core work, named in §C.11 item 3 |

The scale row is **not** disclosed in words, deliberately: an absent control
for a thing that is *correctly* group-scoped needs no apology, and the
Set-scale window is where an operator looks for a scale anyway. The group row
needed one only while the capability was missing, and now needs a different
one — about what the move does rather than about it being impossible.
