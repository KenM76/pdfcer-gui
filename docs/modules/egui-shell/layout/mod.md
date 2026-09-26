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

### `enum LayoutError`

Reading never fails — see this module's header — so this is a
write-side error only, and it is a real `Result` because a failed save
is something the operator must be told about: they are about to close
an application believing their arrangement is safe.

### `fn to_ron_pretty`

# Errors

[`LayoutError::Serialize`] if the document cannot be rendered,
which in practice means a non-finite `f32` reached the writer —
[`crate::dock::DockLayout::normalize`] removes those, and
`a_normalized_layout_always_serializes` is the test that says the
two agree.

### `fn save_to_path`

# Errors

[`LayoutError::Serialize`] or [`LayoutError::Io`].

### `fn from_ron`

**Never fails.** `fallback` is the application's built-in default
arrangement, used when the text cannot be parsed at all; pass the
same value the application would use on a fresh profile.

`catalog` is what makes a stale panel id detectable. Pass
[`crate::dock::AnyPanel`] only in tooling that has no registry — in
an application it would disable the check that turns a mount for a
compiled-out capability into a disclosed skip rather than an empty
compartment.

### `fn load_from_path`

**Never fails.** A missing file yields the fallback and a
[`LayoutSkipReason::FileMissing`], which
[`LoadReport::is_noteworthy`] deliberately does not count as worth
telling anybody about: a first run is not a failure.

### `enum Scope`

A layout inside a named workspace reports its problems against the
**workspace**, not against a column index, because *"workspace
`Review`: `signatures` is not a panel this build offers"* is what an
operator can act on, whereas *"the left dock, column 0, compartment 1"*
is ambiguous between the live arrangement and four saved ones.

### `fn sanitize`

This is [`crate::dock::DockLayout::normalize`] with a voice. The two
must agree — a load that repaired something `normalize` would not, or
vice versa, would mean an arrangement that changes shape between being
loaded and being drawn — and
`sanitizing_leaves_nothing_for_normalize_to_do` is the test that holds
them together.
