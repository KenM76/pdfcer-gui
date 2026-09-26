# `egui-shell/layout/mod`

## Item notes

### `fn a_rich_layout_round_trips_through_text_unchanged`

The property everything else in this module depends on. Asserted
against a *rich* arrangement — two columns, a tabbed stack, a
non-default width, both sides — because a round trip over a
one-panel layout is satisfied by a serializer that drops almost
everything.

### `fn a_file_from_another_version_still_loads`

Without the first, every field ever added is a day on which
everybody's layout resets. Without the second, a file touched by a
newer build is unreadable by the one the operator rolled back to.

### `fn a_panel_this_build_does_not_offer_loses_its_tab_and_nothing_else`

The `SHELL_FRAMEWORK.md` §7 case: a capability compiled out
registers no panel, so its saved mount is dropped — with the
operator's arrangement of everything else intact, and with no
`#[cfg]` anywhere in this crate.

### `fn an_arrangement_left_with_nothing_falls_back_to_the_default`

A build with none of the saved panels — every capability compiled
out, or an application that renamed all of its ids — must not
present a dock with nothing in it, which is indistinguishable from
a broken application. Every individual drop has already been
disclosed, so the fallback adds no new reason.

### `fn sanitizing_leaves_nothing_for_normalize_to_do`

They are two implementations of one set of invariants — one with a
voice, one without — and an arrangement that changed shape between
being loaded and being drawn would be a defect no test of either
alone could find.
