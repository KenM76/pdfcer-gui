# `pdfcer-gui/app/lifecycle/tests`

## Item notes

### `fn opening_a_document_forgets_the_panels_focus_and_expansion`

Expansion sets and the Properties focus are paint-order indices that live
on `PdfcerApp`, so they genuinely do outlive a document. Documents are
opened in exactly one place, so forgetting is one statement at the one
moment it is true — rather than a document identity compared every frame.

Without it, opening a second document leaves the Objects panel with
rows expanded for a page that no longer exists and the Properties
panel describing whatever object lands at that index in the new
file.

### `fn read_mode_opens_a_document_continuous_and_the_others_paged`

`MODES_AND_PANELS.md`'s table and the operator's decision, asserted through
the **open path** rather than through
`PageDisplay::default_for_mode` — which is already tested in its own
module. What this adds is that `open_path` actually consults it: the
rule existing and the rule being applied are two different facts, and
the second is the one an operator experiences.

Driven with no remembered choice for the fixture (nothing has ever set
one for a path under the engine fixtures directory), so what is measured
is the mode default and not a leftover.

### `fn a_freshly_opened_document_is_not_mid_navigation`

`tracked_page` starting anywhere but at `view.page_index` would make
the canvas scroll a continuous strip on the first frame after an open,
which the operator did not ask for and which would fight a saved scroll
position the moment there is one.

### `fn the_new_command_makes_a_blank_document_from_nothing`

Driven through the real token lookup rather than by calling the arm,
exactly as `the_close_command_empties_the_shell` is, so a command that
stopped being registered fails here instead of silently taking the
`command-unimplemented` path. A test that calls the arm directly cannot
see that failure at all: the verb works and the control that raises it
does nothing.

The starting state is `Empty`, which is the state New exists for: an
operator who has just launched pdfcer with no argument.

### `fn new_replaces_the_open_document_and_forgets_its_panel_state`

Why New and open share [`PdfcerApp::adopt`] rather than each doing the
forgetting themselves. A New
that left the panels' paint-order indices behind would show the Objects
panel expanded over rows of a four-page drawing that is no longer open,
on a document that has one blank page — and every test of `open_path`
would still pass, because `open_path` would still be doing it correctly.

The page count moving from four to one is what makes "replaced" a
measurement rather than an assumption.

### `fn each_new_document_is_numbered_from_one`

`crate::text::files::untitled`'s own test pins that the *function*
numbers; this pins that the **application** advances the ordinal, which
is a different fact and the one that breaks if the increment is dropped
or placed after the name is built. Without it both documents would be
`Untitled 1.pdf`, the forms cache would key two different documents the
same way, and the trace of a driven run could not tell a second New
from a New that did nothing.

### `fn a_created_document_is_not_remembered_but_an_opened_one_is`

Both halves, because the interesting failure is not "New was skipped"
but "the guard was written the wrong way round and now nothing is ever
remembered". A Recent menu offering `Untitled 1.pdf` is a row that
cannot be opened, on a surface whose whole promise is *this worked
before*.

`PdfcerApp::new()` under `cfg(test)` builds a `RecentFiles` that points
nowhere and writes nothing, so this reads the in-memory list and leaves
the operator's own recent file untouched.

### `fn only_a_document_with_a_file_has_somewhere_to_store_its_preferences`

The predicate that decides whether a document has anywhere to keep its
per-file preferences. Asserted as a pair rather than
one at a time, because a version that answered `None` for everything
would satisfy every assertion about created documents in this file and
would silently stop persisting page-display and guide choices for real
ones — a regression with no visible symptom until the next session.

### `fn a_new_document_takes_the_modes_default_arrangement`

The sibling of `read_mode_opens_a_document_continuous_and_the_others_paged`,
and it asserts something that test cannot: a created document reaches
the *second* source every time, because `stored_under` answers `None`
and there is nothing to recall. New therefore inherits the mode the
operator is in rather than changing it — see `new_document`'s own note
on why it does not switch to Edit.

### `fn a_failed_open_forgets_the_panels_state_as_well`

Whatever was showing is gone either way, and stale expansion state
over a document that could not be read is the worse of the two states
to leave behind: the panel would look populated while the shell says
the file is damaged.
