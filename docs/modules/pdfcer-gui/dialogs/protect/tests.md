# `pdfcer-gui/dialogs/protect/tests`

## Item notes

### `fn restricted_permissions_dialog`

This helper exists because
[`the_form_opens_seeded_from_the_document_not_from_a_default`] **cannot be
falsified on the plain fixture**. An unprotected document grants every bit,
so `ticks == standing.initial_ticks()` holds just as well when the seeding
expression is replaced by a hard-coded *eight ticks* — which is the exact
defect that test exists to catch.

⇒ **A test that checks a relation rather than a magnitude is satisfied by
any absurdity in the right direction**, and two values that are equal for a
reason unrelated to the code under test are the same trap. So the assertion
is made over a document written with **`Print` alone**, where a hard-coded
default and the file's own answer are different lists and cannot be
confused. `granted` is what the caller wants written; the file comes back
also granting `AccessibilityExtract`, which the engine sets on every write
(see [`crate::protect::always_granted`]).

### `fn the_form_opens_seeded_from_the_document_not_from_a_default`

The build brief's strongest requirement on this surface, asserted at the
seam where it could be lost: *"a permissions dialog that opens with
everything ticked, on a document that forbids printing, has told him a
falsehood before he touches anything."*

**It is asserted on a document that FORBIDS something**, and it has to be:
on the plain fixture the answer genuinely IS eight ticks, so a hard-coded
eight ticks and the file's own answer are the same list and no assertion
here can tell them apart. See [`restricted_permissions_dialog`].

### `fn a_signed_document_draws_no_form_at_all`

Not a greyed form and not a button that refuses on press — the phase is
`Refused` before anything is drawn, so [`ProtectDialog::body`] takes the
branch that states the refusal and offers nothing.

### `fn the_greyed_confirm_names_the_outstanding_condition`

`OPERATOR_REQUESTS.md` O77's sweep found seven greyed controls with no hover
explanation. Several different conditions gate this one button and they
appear at different times, so *"fill in the form"* would be vague exactly
when it matters.

### `fn changing_the_destination_retires_the_acknowledgement`

`crate::dialogs::redact::choose_destination`'s rule, and it stops the one
sequence that would otherwise leave a live button over a withdrawn consent:
tick, think better of it, choose *a new file*, change your mind, and arrive
back at *replace* with the button already enabled.

### `fn changing_the_job_re_seeds_the_ticks_from_the_file`

Without it, editing the list under one job and then selecting another leaves
the boxes wherever the previous editing put them — which on
[`Job::ChangePassword`] would be a permission set the operator chose for a
job that does not write one.

### `fn the_accessibility_bit_is_never_offered_as_a_choice`

`pdfcer-core` sets bit 10 on every file it writes (rule W19), so a tick-box
the operator could clear would come back ticked in the result. The row is a
statement instead, and [`ProtectDialog::granted`] forces the bit in
regardless of what the tick says — so the list passed to the engine is what
the written file will actually say.

The second half is the one that would rot silently: a future edit that
made the checkbox editable would still pass the first assertion.

### `fn the_debug_impl_does_not_carry_a_password`

[`crate::secret`]'s header names the cost: a `{:?}` on anything carrying a
password writes it into the trace file `tools/ui-verify` keeps as evidence.
Five fields here hold one, as `String` rather than `Secret`, because
`egui::TextEdit` binds to a `String` — so the type cannot do the protecting
and [`ProtectDialog`]'s hand-written `Debug` must.

A derived `Debug` would pass every other test in this file.

### `fn each_failure_says_something_different`

The exhaustive `match` in [`failure_line`] is what makes a new
[`PrepareFailure`] variant a compile error rather than a silent
fall-through; this asserts the sentences are actually different, which the
compiler cannot.
