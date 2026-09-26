# `panels::properties::dimension::overrides` — eleven properties, eleven
checkboxes, and the tier each value came from

## What this is

`StyleOverrides` — the **bottom** tier of the cascade — made editable, which
is the half `FEATURES.md` records as `core [x] cli [x] gui [ ]`:

> ce-dimension style AND tolerance in the GUI — **one panel covering both**,
> showing which values are inherited and which are overridden and letting
> the override be set.

Eleven `Option`s, and **each `Option` IS the operator's checkbox**. The
operator asked for it in those words on 2026-08-12: *"groups of dimensions
should have a default dimensioning and tolerance style that can be set for
the group, but these should have a checkbox to override and set
differently."*

## ★ The disclosure is DATA, not a heuristic

`style_provenance(group, &record.style)` answers, per property, which tier
supplied the value in force: `Factory`, `Group` or `Dimension`. This module
renders that; it never computes it. Amendment B §B.2:

> This satisfies §C.11.1's two-state convention (*inherited-and-greyed* vs
> *overridden-and-editable*) with data rather than with a UI-local
> heuristic. The panel needs to render it; it does not need to compute it.

Two consequences a reader should not have to rediscover:

- **`Factory` and `Group` are shown as different sentences**, because they
  are different answers to *"where do I go to change this for everything?"*
  — and collapsing them into one "inherited" would hide which tier to edit.
- **Both answer *"will a group edit move this?"* with yes.**
  `StyleSource::follows_group()` is `true` for `Factory`, which is the easy
  thing to get wrong, and the note beside each row takes the predicate from
  the engine rather than matching on the variant here.

## ★ Four properties can never report `Factory`, and that is not a bug

`unit`, `fraction`, `decimal_marker` and `standard` are **concrete**, not
`Option`s — the first three inside `Group::format`, the last as
`Group::standard` — so the group always has a value for them
and their provenance is `Group` or `Dimension` and nothing else — that is
`style_provenance`'s `two` closure. Saying "factory" for them *"would be a lie an operator
could act on"* — the engine's words. Nothing here special-cases them,
because nothing here derives provenance; they simply never render the
factory sentence.

## Why there is no scale row

By refusal, and the refusal is structural: `StyleOverrides` has **no scale
field**, asserted by `scale_is_never_overridable_per_ce_dimension`. A ce
dimension quietly measuring at a
different scale from its group would print a number that is wrong in a way
nothing on the page discloses, which rule 4 makes a refusal rather than a
feature. Scale is set on the group, in the Set-scale window, and this
paragraph exists so nobody adds the row.
