# `panels::dimension_groups::style` — a group's appearance defaults, and
the count that stops them being a surprise

## What this draws

The **middle tier** of `pdfcer-core`'s three-tier style cascade
(factory → group → ce dimension). Seven properties, each an `Option` on
`GroupStyle`, and **the `Option` is the operator's checkbox**: clear means
*this group has not spoken, so pdfcer's own default applies*, ticked means
*this group says this*.

Two of the seven — `tolerance` and `tolerance_places` — are not drawn here
and the reason is at [`show`].

## The count beside every control, and why it is not the engine's

The operator's own words, quoted in
`docs/ui_specs/tool-options-dock-and-ce-dimension-properties.md` §C.11.1:

> *"cannot change one and be surprised 40 others changed or didn't."*

`EditSession::set_group_style` returns a count, and it is **the wrong
number to show**. `docs/core-api/03-capabilities.md` §1.6 trap (a) says so
outright: the return is the number of members *regenerated*, which is every
wired member including the ones that override the very property being
changed — because regenerating an overrider is byte-identical and free in
the diff, so the engine does not bother to exclude them.

The number that will visibly **move** is the members whose
`StyleProvenance` for that property reports `follows_group() == true`, and
it has to be computed **before** the edit if it is to be shown before the
edit. [`will_move`] is that computation, and it is called every frame for
every drawn property so the sentence under a control is always about the
model as it stands.

## `Factory` counts as following the group, and this is the easy thing to
get wrong

`StyleSource::follows_group()` is `true` for **both** `Factory` and
`Group`. A property nobody has set yet *will* move when the group sets one
— the group simply has not spoken. A panel that derives the predicate by
hand and tests only for `Group` greys out, or under-counts, exactly the rows
that are about to change.

`pdfcer-core` pins that with a test named for the trap
(`factory_sourced_properties_still_follow_a_group_edit`), and this module
never re-derives the predicate: it calls `StyleSource::follows_group`.
