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

## The disclosure is DATA, not a heuristic

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

## Four properties can never report `Factory`, and that is not a bug

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

## Item notes

### `const DENOMINATORS`

Powers of two only. `pdfcer-core` accepts any `u32`, and a drawing
dimensioned to the nearest 1/7 inch does not exist — offering the free
integer would be a control whose useful values are five of four billion.

### `struct Row`

# Why this is a two-step builder rather than one function

Because the editor's type differs per row and its closure has to borrow the
**unwrapped** value, which only exists after the checkbox has decided
whether there is one. A single function taking both would need the seed
closure and the editor closure in the same call, which is where clippy's
argument budget and the reader's patience both run out at row eleven.

The two-step shape also puts the invariant in the type: `edit` is the only
way to reach the value, so a row cannot be drawn with a checkbox and no
editor.

### `fn edit`

**Absent, not greyed, when inherited.** R9 reserves greying for
*temporarily* unavailable, and an inherited property is not unavailable
— it has a value, supplied by a tier above. A greyed spinner would
invite a drag and then refuse it. The provenance sentence already drawn
beside the checkbox says what the value is instead.

### `fn row`

`seed` is called **only** when the checkbox is ticked, and returns the
resolved value — what was in force a moment ago. Seeding from the resolved
value rather than from the factory default is what makes ticking a box a
visual no-op: the number does not jump when it becomes editable, so the
operator's first drag starts from where they were.

### `const DRAWN`

Exists **only** for the test below, and that is worth the lines. The
engine's `StyleProvenance::each()` returns a fixed-size `[_; 11]` precisely
so a consumer gets a compile error rather than a short list when a twelfth
property lands — but this module does not call `each()`, it reads the fields
by name, so it would silently keep drawing eleven rows for ever.

This closes that: the test compares this list against `each()`'s names.

### `fn no_property_of_the_cascade_is_left_without_a_row`

The gap this closes is specific and would otherwise be silent. The
engine's `StyleProvenance::each()` is a fixed-size array so that a
consumer iterating it fails to compile when a property is added — and
this module reads the provenance **fields by name** rather than
iterating, which is the right shape for a panel that draws eleven
different editors and the wrong shape for noticing a twelfth.

So the array is compared against `each()`'s names here, and adding a
property to `pdfcer-core` without a row in this file fails this test with
the property's own name in the message.

### `fn ticking_an_override_starts_from_the_value_that_was_showing`

The property is *the number does not jump*. Seeding from
`StyleDefaults::FACTORY` instead would take a ce dimension inheriting a
3 pt group text height and snap it to 10 pt the instant the operator
asked to edit it — a change nobody requested, applied by a checkbox.

### `fn the_four_concrete_properties_never_claim_a_factory_source`

Asserted against the engine rather than assumed, because this panel
renders whatever provenance it is given: if `unit` ever did report
`Factory`, the row would show *"using pdfcer's default"* for a property
whose group always has a value — a sentence the engine calls *"a lie an
operator could act on"*.
